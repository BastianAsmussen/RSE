mod crawler;
mod utils;

use log::{error, info};

use crate::utils::env::seed_url;
use crate::utils::timer::{format_time, Timer};

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut timer = Timer::default();

    // Fetch seed URLs.
    info!("Fetching seed URLs...");
    let (time, seed_urls) = timer.time(|| async { seed_url::fetch() }).await;
    let seed_urls = match seed_urls {
        Ok(seed_urls) => {
            info!(
                "Fetched {} seed URLs in {}!",
                seed_urls.len(),
                format_time(&time)
            );

            seed_urls
        }
        Err(why) => {
            error!("Failed to fetch seed URLs: {why}");

            return;
        }
    };

    // Create a new crawler.
    let mut crawler = crawler::Crawler::new(&seed_urls);

    // Start the crawler.
    info!("Starting the crawler...");
    match crawler.start().await {
        Ok(()) => info!(
            "Crawled {} URLs!",
            crawler.get_frontier().get_crawled().len()
        ),
        Err(why) => error!("Crawling failed: {why}"),
    };
}
