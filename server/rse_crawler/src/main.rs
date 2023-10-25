mod crawler;
mod error;
mod spiders;
mod utils;

use log::{error, info};
use std::sync::Arc;

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

    // Create a crawler.
    let crawler = crawler::Crawler::default();

    // Create a spider.
    let spider = spiders::html::WebSpider::default();

    // Run the crawler.
    info!("Running the crawler...");

    let (time, ()) = timer
        .time(|| async {
            crawler.run(Arc::new(spider)).await;
        })
        .await;

    info!(
        "Crawled {} URLs in {}!",
        seed_urls.len(),
        format_time(&time)
    );
}
