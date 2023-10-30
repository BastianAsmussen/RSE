use crate::spiders::Spider;
use futures::StreamExt;
use log::{error, info};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Barrier};

/// A web crawler.
///
/// # Fields
///
/// * `delay`: The delay between requests.
/// * `crawling_workers`: The number of concurrent crawlers.
/// * `processing_workers`: The number of concurrent processors.
#[derive(Debug)]
pub struct Crawler {
    delay: Duration,
    crawling_workers: usize,
    processing_workers: usize,
}

impl Crawler {
    pub fn new(delay: Duration, crawling_workers: usize, processing_workers: usize) -> Self {
        Self {
            delay,
            crawling_workers,
            processing_workers,
        }
    }

    /// Runs the crawler.
    ///
    /// # Arguments
    ///
    /// * `spider`: The spider to use.
    pub async fn run<T: Send + 'static>(&self, spider: Arc<dyn Spider<Item = T>>) {
        let mut visited_urls = HashSet::<String>::new();

        let crawling_workers = self.crawling_workers;
        let crawling_queue_capacity = crawling_workers * 400;

        let processing_workers = self.processing_workers;
        let processing_queue_capacity = processing_workers * 10;

        let active_spiders = Arc::new(AtomicUsize::new(0));

        let (urls_to_visit_tx, urls_to_visit_rx) = mpsc::channel(crawling_queue_capacity);
        let (items_tx, items_rx) = mpsc::channel(processing_queue_capacity);
        let (new_urls_tx, mut new_urls_rx) = mpsc::channel(crawling_queue_capacity);

        let barrier = Arc::new(Barrier::new(3)); // 3 tasks: 1 for the control loop, 1 for the processors, 1 for the scrapers.

        for url in spider.seed_urls() {
            visited_urls.insert(url.clone());

            let _ = urls_to_visit_tx.send(url).await;
        }

        self.launch_processors(
            processing_workers,
            spider.clone(),
            items_rx,
            barrier.clone(),
        );

        self.launch_scrapers(
            crawling_workers,
            spider.clone(),
            urls_to_visit_rx,
            new_urls_tx.clone(),
            items_tx,
            active_spiders.clone(),
            self.delay,
            barrier.clone(),
        );

        loop {
            if let Ok((visited_url, new_urls)) = new_urls_rx.try_recv() {
                visited_urls.insert(visited_url);

                for url in new_urls {
                    if !visited_urls.contains(&url) {
                        visited_urls.insert(url.clone());

                        info!("Queued URL: {}", url);

                        let _ = urls_to_visit_tx.send(url).await;
                    }
                }
            }

            if new_urls_tx.capacity() == crawling_queue_capacity // If the new_urls_tx is empty,
                && urls_to_visit_tx.capacity() == crawling_queue_capacity // and the urls_to_visit_tx is empty,
                && active_spiders.load(Ordering::SeqCst) == 0
            // and there are no active spiders.
            {
                // We're finished!
                break;
            }

            tokio::time::sleep(Duration::from_millis(5)).await;
        }

        info!("Control loop finished! Waiting for streams to complete...");

        // Drop the transmitter in order to close the stream.
        drop(urls_to_visit_tx);
        // Wait for the streams to complete.
        barrier.wait().await;
    }

    /// Launches the processors.
    ///
    /// # Arguments
    ///
    /// * `workers`: The number of concurrent processors.
    /// * `spider`: The spider to use.
    /// * `items`: The items to process.
    /// * `barrier`: The barrier to wait on.
    fn launch_processors<T: Send + 'static>(
        &self,
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

    /// Launches the scrapers.
    ///
    /// # Arguments
    ///
    /// * `workers`: The number of concurrent scrapers.
    /// * `spider`: The spider to use.
    /// * `urls_to_visit`: The URLs to visit.
    /// * `new_urls_tx`: The transmitter to send new URLs to.
    /// * `items_tx`: The transmitter to send items to.
    /// * `active_spiders`: The number of active spiders.
    /// * `delay`: The delay between requests.
    /// * `barrier`: The barrier to wait on.
    fn launch_scrapers<T: Send + 'static>(
        &self,
        workers: usize,
        spider: Arc<dyn Spider<Item = T>>,
        urls_to_visit: mpsc::Receiver<String>,
        new_urls_tx: mpsc::Sender<(String, Vec<String>)>,
        items_tx: mpsc::Sender<T>,
        active_spiders: Arc<AtomicUsize>,
        delay: Duration,
        barrier: Arc<Barrier>,
    ) {
        tokio::spawn(async move {
            tokio_stream::wrappers::ReceiverStream::new(urls_to_visit)
                .for_each_concurrent(workers, |queued_url| async {
                    active_spiders.fetch_add(1, Ordering::SeqCst);
                    let mut urls = Vec::new();
                    let res = spider
                        .scrape(queued_url.clone())
                        .await
                        .map_err(|err| {
                            error!("Failed to scrape {}: {err}", queued_url);

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
                    tokio::time::sleep(delay).await;
                    active_spiders.fetch_sub(1, Ordering::SeqCst);
                })
                .await;

            drop(items_tx);
            barrier.wait().await;
        });
    }
}
