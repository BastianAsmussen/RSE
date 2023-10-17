mod crawler;
mod indexer;
mod utils;

use log::{error, info};
use std::collections::HashMap;

use crate::crawler::crawl;
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

    let chunks = seed_urls
        .chunks(utils::get_worker_threads())
        .map(|chunk| chunk.join("\n"));
    for chunk in chunks {
        let chunk_websites = match tokio::spawn(async move {
            let mut websites = Vec::new();
            for url in chunk.split('\n') {
                let website = match crawl(url).await {
                    Ok(website) => website,
                    Err(e) => {
                        error!("Failed to crawl seed URL {url}! (Error: {e})");

                        continue;
                    }
                };

                info!("Crawled seed URL {url}!");

                websites.push(website);
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

    info!(
        "Crawled {} seed URLs in {}!",
        websites.len(),
        format_time(&timer.total_time())
    );

    // Index crawled websites.
    info!("Indexing crawled websites...");

    let (time, indexed_pages) = timer.time(|| async {
            info!("Indexing crawled websites...");

            let mut indexed_pages = HashMap::new();
            for website in websites {
                let pages = indexer::scrape(&website);
                for url in pages.keys() {
                    info!("Indexed page {url}!");

                    indexed_pages.insert(url.to_string(), website.clone());

                    // TODO: Store pages in database.
                    info!("Data: {:#?}", pages.get(url).unwrap());
                }
            }

            indexed_pages
        }).await;

    info!(
        "Indexed {} pages in {}!",
        indexed_pages.len(),
        format_time(&time)
    );
}
