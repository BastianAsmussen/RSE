mod crawler;
mod indexer;
mod utils;

use log::{error, info};
use std::collections::HashMap;

use crate::utils::seed_urls;
use crate::utils::timer::{format_time, Timer};
use spider::tokio;

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut timer = Timer::default();

    // Fetch seed URLs.
    info!("Fetching seed URLs...");
    let (time, Some(seed_urls)) = timer.time(|| async { seed_urls::fetch() }).await else {
        panic!("Failed to fetch seed URLs, exiting...");
    };
    info!(
        "Loaded {} seed URLs in {}!",
        seed_urls.len(),
        format_time(&time)
    );

    // Crawl seed URLs.
    info!("Crawling seed URLs...");

    let mut websites = Vec::new();

    let crawl = async {
        let chunks = seed_urls
            .chunks(utils::get_worker_threads())
            .map(|chunk| chunk.join("\n"));
        for chunk in chunks {
            let chunk_websites = match tokio::spawn(async move {
                let mut websites = Vec::new();
                for url in chunk.split('\n') {
                    let website = match crawler::crawl(url).await {
                        Ok(website) => website,
                        Err(e) => {
                            error!("Failed to crawl seed URL {url}! (Error: {e})");

                            continue;
                        }
                    };

                    info!("Crawled seed URL {url}!");

                    websites.push(website.clone());

                    // Index crawled websites.
                    info!("Indexing crawled websites...");

                    let mut indexed_pages = HashMap::new();
                    let pages = indexer::scrape(&website);
                    for url in pages.keys() {
                        info!("Indexed page {url}!");

                        indexed_pages.insert(url.to_string(), website.clone());

                        // TODO: Store pages in database.
                        info!("{:#?}", pages.get(url).unwrap());
                    }

                    info!(
                        "Indexed {} pages in {}!",
                        indexed_pages.len(),
                        format_time(&time)
                    );
                }

                websites
            })
            .await
            {
                Ok(websites) => websites,
                Err(e) => {
                    error!("Failed to crawl seed URLs (Error: {e})");

                    continue;
                }
            };

            websites.extend(chunk_websites);
        }
    };

    let (time, ()) = timer.time(|| crawl).await;
    info!(
        "Crawled {} seed URLs in {}!",
        websites.len(),
        format_time(&time)
    );
}
