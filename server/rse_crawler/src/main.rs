mod crawler;
mod indexer;
mod utils;

use crate::utils::db::model::Page;
use log::{error, info};

use crate::utils::env::seed_urls;
use crate::utils::timer::{format_time, Timer};

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

    // Split the seed URLs over the number of worker threads.
    let url_chunks = seed_urls
        .chunks(utils::env::crawler::get_worker_threads())
        .map(|url_chunk| url_chunk.join("\n"))
        .collect::<Vec<String>>();

    // Create a channel to send the crawled URLs over.
    let (send, mut recv) = tokio::sync::mpsc::channel::<Vec<Page>>(url_chunks.len());
    for url_chunk in url_chunks {
        let send = send.clone();
        rayon::spawn(move || {
            let mut pages_chunk = Vec::new();

            for url in url_chunk.lines() {
                if let Err(e) = crawler::crawl_url(
                    &mut pages_chunk,
                    url,
                    utils::env::crawler::get_max_crawl_depth(),
                    0,
                ) {
                    error!("Failed to crawl URL: {}", e);
                }
            }

            if let Err(e) = send.blocking_send(pages_chunk) {
                error!("Failed to send crawled URLs: {}", e);
            }
        });
    }

    let (time, pages) = timer
        .time(|| async {
            let mut pages = Vec::new();

            while let Some(pages_chunk) = recv.recv().await {
                pages.extend(pages_chunk);
            }

            pages
        })
        .await;

    info!("Crawled {} URLs in {}!", pages.len(), format_time(&time));
}
