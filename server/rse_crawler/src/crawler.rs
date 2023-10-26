use crate::spiders::Spider;
use crate::utils;
use futures::stream::StreamExt;
use log::{error, info};
use std::str::FromStr;
use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    sync::{mpsc, Barrier},
    time::sleep,
};
use url::Url;

/// The crawling queue capacity multiplier is used to determine the capacity of the crawling queue.
const CRAWLING_QUEUE_CAPACITY_MULTIPLIER: usize = 400;

/// The processing queue capacity multiplier is used to determine the capacity of the processing queue.
const PROCESSING_QUEUE_CAPACITY_MULTIPLIER: usize = 10;

/// The ordering used for atomic operations.
const ATOMIC_ORDERING: Ordering = Ordering::SeqCst;

/// The sleep duration between each iteration of the control loop.
const SLEEP_DURATION: Duration = Duration::from_millis(5);

/// Represents a crawler that crawls a website.
///
/// # Fields
///
/// * `delay` - The delay between each request.
/// * `crawling_workers` - The number of crawling workers.
/// * `processing_workers` - The number of processing workers.
#[derive(Debug)]
pub struct Crawler {
    delay: Duration,
    crawling_workers: usize,
    processing_workers: usize,
}

impl Default for Crawler {
    fn default() -> Self {
        Self {
            delay: utils::env::crawler::get_delay(),
            crawling_workers: utils::env::workers::get_crawler_workers(),
            processing_workers: utils::env::workers::get_processing_workers(),
        }
    }
}

impl Crawler {
    /// Creates a new crawler.
    ///
    /// # Arguments
    ///
    /// * `delay` - The delay between each request.
    /// * `crawling_workers` - The number of crawling workers.
    /// * `processing_workers` - The number of processing workers.
    pub fn new(delay: Duration, crawling_workers: usize, processing_workers: usize) -> Self {
        Self {
            delay,
            crawling_workers,
            processing_workers,
        }
    }

    pub async fn run<T: Send + 'static>(&self, spider: Arc<dyn Spider<Item = T>>) {
        let mut visited_urls = HashSet::<Url>::new();

        // Define crawling/processing queue capacity, which is twice the number of crawling/processing workers.
        let crawling_workers = self.crawling_workers;
        let crawling_queue_capacity = crawling_workers * CRAWLING_QUEUE_CAPACITY_MULTIPLIER;

        let processing_workers = self.processing_workers;
        let processing_queue_capacity = processing_workers * PROCESSING_QUEUE_CAPACITY_MULTIPLIER;

        let active_spiders = Arc::new(AtomicUsize::new(0));

        // Define mpsc channels.
        let (urls_to_visit_tx, urls_to_visit_rx) = mpsc::channel(crawling_queue_capacity);
        let (items_tx, items_rx) = mpsc::channel(processing_queue_capacity);

        let (new_urls_tx, mut new_urls_rx) = mpsc::channel(crawling_queue_capacity);

        // Create a barrier to wait for all workers to finish.
        let barrier = Arc::new(Barrier::new(3)); // 1 crawling worker + 1 processing worker + 1 main thread = 3.

        // Send seed URLs to the crawling queue.
        for url in spider.seed_urls() {
            let url = match Url::from_str(url.as_str()) {
                Ok(url) => url,
                Err(err) => {
                    error!("{err}");

                    continue;
                }
            };

            visited_urls.insert(url.clone());

            let _ = urls_to_visit_tx.send(url).await;
        }

        // Spawn processing workers.
        Self::launch_processors(
            processing_workers,
            spider.clone(),
            items_rx,
            barrier.clone(),
        );

        // Spawn crawling workers.
        Self::launch_scrapers(
            crawling_workers,
            spider.clone(),
            urls_to_visit_rx,
            new_urls_tx.clone(),
            items_tx,
            active_spiders.clone(),
            self.delay,
            barrier.clone(),
        );

        // Start the control loop.
        loop {
            if let Ok((visited_url, new_urls)) = new_urls_rx.try_recv() {
                visited_urls.insert(visited_url);

                for url in new_urls {
                    if !visited_urls.contains(&url) {
                        visited_urls.insert(url.clone());

                        info!("Queueing: {url}");

                        let _ = urls_to_visit_tx.send(url).await;
                    }
                }
            }

            if new_urls_tx.capacity() == crawling_queue_capacity // new_urls channel is empty.
                && urls_to_visit_tx.capacity() == crawling_queue_capacity // urls_to_visit channel is empty.
                && active_spiders.load(ATOMIC_ORDERING) == 0
            {
                // No more work, so we exit the control loop.
                break;
            }

            // After each iteration, we wait for a short period of time.
            sleep(SLEEP_DURATION).await;
        }

        info!("No more URLs to crawl, exiting...");

        // we drop the transmitter in order to close the stream
        drop(urls_to_visit_tx);

        // and then we wait for the streams to complete
        barrier.wait().await;
    }

    /// Launches a number of processing workers.
    ///
    /// # Arguments
    ///
    /// * `workers` - The number of processing workers.
    /// * `spiders` - The spiders.
    /// * `items` - The items to process.
    /// * `barrier` - The barrier to wait for all workers to finish.
    fn launch_processors<T: Send + 'static>(
        workers: usize,
        spider: Arc<dyn Spider<Item = T>>,
        items: mpsc::Receiver<T>,
        barrier: Arc<Barrier>,
    ) {
        tokio::spawn(async move {
            tokio_stream::wrappers::ReceiverStream::new(items)
                .for_each_concurrent(workers, |item| async {
                    let _ = spider.process(item).await;
                })
                .await;

            barrier.wait().await;
        });
    }

    /// Launches a number of crawling workers.
    ///
    /// # Arguments
    ///
    /// * `workers` - The number of crawling workers.
    /// * `spiders` - The spiders.
    /// * `urls_to_visit` - The URLs to visit.
    /// * `new_urls_tx` - The new URLs to send.
    /// * `items_tx` - The items to send.
    /// * `active_spiders` - The active spiders.
    /// * `delay` - The delay between each request.
    /// * `barrier` - The barrier to wait for all workers to finish.
    fn launch_scrapers<T: Send + 'static>(
        workers: usize,
        spider: Arc<dyn Spider<Item = T>>,
        urls_to_visit: mpsc::Receiver<Url>,
        new_urls_tx: mpsc::Sender<(Url, Vec<Url>)>,
        items_tx: mpsc::Sender<T>,
        active_spiders: Arc<AtomicUsize>,
        delay: Duration,
        barrier: Arc<Barrier>,
    ) {
        tokio::spawn(async move {
            tokio_stream::wrappers::ReceiverStream::new(urls_to_visit)
                .for_each_concurrent(workers, |queued_url| async {
                    active_spiders.fetch_add(1, ATOMIC_ORDERING);

                    let mut urls = Vec::new();
                    let res = spider
                        .scrape(&queued_url)
                        .await
                        .map_err(|err| {
                            error!("{err}");

                            err
                        })
                        .ok();

                    if let Some((items, new_urls)) = res {
                        for item in items {
                            let _ = items_tx.send(item).await;
                        }

                        urls = new_urls;
                    }

                    let _ = new_urls_tx.send((queued_url, urls)).await;

                    sleep(delay).await;
                    active_spiders.fetch_sub(1, ATOMIC_ORDERING);
                })
                .await;

            drop(items_tx);
            barrier.wait().await;
        });
    }
}
