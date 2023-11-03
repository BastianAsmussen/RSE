use crate::scrapers::Scraper;
use futures::StreamExt;
use log::{error, info};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Barrier};
use tokio_stream::wrappers::ReceiverStream;
use url::Url;

/// The maximum number of items that can be in the queues at once. If it's exceeded, the control loop will exit.
pub const SCRAPER_QUEUE_CAPACITY_MULTIPLIER: usize = 400;

/// The maximum number of items that can be in the queues at once. If it's exceeded, the control loop will exit.
pub const PROCESSOR_QUEUE_CAPACITY_MULTIPLIER: usize = 10;

/// A crawler is responsible for orchestrating the crawling of URLs.
///
/// # Fields
///
/// * `delay`: The delay between requests.
///
/// * `scraper_queue_capacity`: The maximum number of items that can be in the scraper queue at once.
/// * `processor_queue_capacity`: The maximum number of items that can be in the processor queue at once.
#[derive(Debug)]
pub struct Crawler {
    delay: Duration,

    scraper_queue_capacity: usize,
    processor_queue_capacity: usize,
}

impl Crawler {
    /// Creates a new crawler.
    ///
    /// # Arguments
    ///
    /// * `delay` - The delay between requests.
    ///
    /// * `scrapers` - The number of scrapers running at once.
    /// * `processors` - The number of processors running at once.
    pub const fn new(delay: Duration, scrapers: usize, processors: usize) -> Self {
        Self {
            delay,

            scraper_queue_capacity: scrapers * SCRAPER_QUEUE_CAPACITY_MULTIPLIER,
            processor_queue_capacity: processors * PROCESSOR_QUEUE_CAPACITY_MULTIPLIER,
        }
    }

    /// Runs the crawler.
    ///
    /// # Arguments
    ///
    /// * `scraper`: The scraper to use.
    pub async fn run<T: Send + 'static>(&self, scraper: Arc<dyn Scraper<Item = T>>) {
        let mut visited_urls = HashSet::<Url>::new();

        let active_scrapers = Arc::new(AtomicUsize::new(0));

        let (urls_to_visit_tx, urls_to_visit_rx) = mpsc::channel(self.scraper_queue_capacity);
        let (items_tx, items_rx) = mpsc::channel(self.processor_queue_capacity);
        let (new_urls_tx, mut new_urls_rx) = mpsc::channel(self.scraper_queue_capacity);

        // Create a barrier to wait for the scrapers, processors, and new control loop to finish.
        let barrier = Arc::new(Barrier::new(3));

        // Add the seed URLs to the queue.
        for (url, depth) in scraper.seed_urls() {
            visited_urls.insert(url.clone());

            let _ = urls_to_visit_tx
                .send(HashMap::from([(url.clone(), depth)]))
                .await;
        }

        // Spawn the processors.
        self.launch_processors(scraper.clone(), items_rx, barrier.clone());

        // Spawn the scrapers.
        self.launch_scrapers(
            scraper.clone(),
            urls_to_visit_rx,
            new_urls_tx.clone(),
            items_tx,
            active_scrapers.clone(),
            barrier.clone(),
        );

        // Start the control loop.
        loop {
            let Ok((visited_url, new_urls)) = new_urls_rx.try_recv() else {
                if new_urls_tx.capacity() == self.scraper_queue_capacity
                    && urls_to_visit_tx.capacity() == self.scraper_queue_capacity
                    && active_scrapers.load(Ordering::SeqCst) == 0
                {
                    break;
                }

                tokio::time::sleep(Duration::from_millis(5)).await;

                continue;
            };

            visited_urls.insert(visited_url);

            for (url, depth) in new_urls {
                if visited_urls.contains(&url) {
                    continue;
                }

                // Retry sending the URL until it's successfully sent to the queue.
                loop {
                    if urls_to_visit_tx
                        .send(HashMap::from([(url.clone(), depth)]))
                        .await
                        .is_err()
                    {
                        // Sleep for a short duration before retrying.
                        tokio::time::sleep(Duration::from_millis(5)).await;

                        continue;
                    }

                    // URL successfully sent, break the retry loop.
                    break;
                }

                visited_urls.insert(url.clone());
                info!("Queued URL: {url}");
            }
        }

        info!("Control loop finished, waiting for streams to complete...");

        drop(urls_to_visit_tx);
        barrier.wait().await;
    }

    /// Launches the processors.
    ///
    /// # Arguments
    ///
    /// * `scraper`: The scraper to use.
    /// * `items`: The items to process.
    /// * `barrier`: The barrier to wait for.
    fn launch_processors<T: Send + 'static>(
        &self,
        scraper: Arc<dyn Scraper<Item = T>>,
        items: mpsc::Receiver<T>,
        barrier: Arc<Barrier>,
    ) {
        let processor_queue_capacity = self.processor_queue_capacity;

        tokio::spawn(async move {
            ReceiverStream::new(items)
                .for_each_concurrent(processor_queue_capacity, |item| async {
                    let _ = scraper.process(item).await;
                })
                .await;

            barrier.wait().await;
        });
    }

    /// Launches the scrapers.
    ///
    /// # Arguments
    ///
    /// * `scraper`: The scraper to use.
    /// * `urls_to_visit`: The URLs to visit.
    /// * `new_urls_tx`: The channel to send new URLs to.
    /// * `items_tx`: The channel to send items to.
    /// * `active_scrapers`: The number of active spiders.
    /// * `barrier`: The barrier to wait for.
    fn launch_scrapers<T: Send + 'static>(
        &self,
        scraper: Arc<dyn Scraper<Item = T>>,
        urls_to_visit: mpsc::Receiver<HashMap<Url, u32>>,
        new_urls_tx: mpsc::Sender<(Url, HashMap<Url, u32>)>,
        items_tx: mpsc::Sender<T>,
        active_scrapers: Arc<AtomicUsize>,
        barrier: Arc<Barrier>,
    ) {
        let scraper_queue_capacity = self.scraper_queue_capacity;
        let delay = self.delay;

        tokio::spawn(async move {
            ReceiverStream::new(urls_to_visit)
                .for_each_concurrent(scraper_queue_capacity, |queued_url| async {
                    active_scrapers.fetch_add(1, Ordering::SeqCst); // Increment the number of active scrapers.

                    let Some((url, depth)) = queued_url.into_iter().next() else {
                        active_scrapers.fetch_sub(1, Ordering::SeqCst); // Decrement the number of active scrapers.

                        return;
                    };

                    let mut urls = HashMap::new();
                    let results = scraper
                        .scrape(url.clone(), depth)
                        .await
                        .map_err(|err| {
                            error!("Failed to scrape {url}: {err}");

                            err
                        })
                        .ok();

                    if let Some((items, new_urls)) = results {
                        for item in items {
                            let _ = items_tx.send(item).await;
                        }

                        urls = new_urls;
                    }

                    let _ = new_urls_tx.send((url.clone(), urls)).await;

                    tokio::time::sleep(delay).await;
                    active_scrapers.fetch_sub(1, Ordering::SeqCst);
                })
                .await;

            drop(items_tx);
            barrier.wait().await;
        });
    }
}
