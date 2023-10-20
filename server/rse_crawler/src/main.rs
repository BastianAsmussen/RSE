mod crawler;
mod indexer;
mod spider;
mod utils;

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

    let mut websites = Vec::new();

    for url in seed_urls {
        info!("Crawling seed URL (\"{url}\")...");

        let new_websites = match spider::crawl_url(&url, 0).await {
            Ok(new_websites) => new_websites,
            Err(why) => {
                error!("Failed to crawl seed URL (\"{url}\"): {why}!");

                continue;
            }
        };

        websites.extend(new_websites);
    }

    info!("Crawled {} URLs.", websites.len());

    /*
    // Split the seed URLs over the number of worker threads.
    let url_chunks = seed_urls
        .chunks(utils::env::crawler::get_worker_threads())
        .map(|url_chunk| url_chunk.join("\n"))
        .collect::<Vec<String>>();

    // Create a channel to send the crawled URLs over.
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<Vec<Website>>(url_chunks.len());
    for url_chunk in url_chunks {
        let sender = sender.clone();
        rayon::spawn(move || {
            let mut websites_chunk = Vec::new();

            for url in url_chunk.lines() {
                if let Err(e) = crawler::crawl_url(
                    &mut websites_chunk,
                    url,
                    utils::env::crawler::get_max_crawl_depth(),
                    0,
                ) {
                    error!("Failed to crawl URL: {}", e);
                }
            }

            if let Err(e) = sender.blocking_send(websites_chunk) {
                error!("Failed to send crawled URLs: {}", e);
            }
        });
    }

    let (time, websites) = timer
        .time(|| async {
            let mut websites = Vec::new();

            while let Some(websites_chunk) = receiver.recv().await {
                websites.extend(websites_chunk);
            }

            websites
        })
        .await;

    info!("Crawled {} URLs in {}!", websites.len(), format_time(&time));

    // Index crawled URLs.
    info!("Indexing crawled URLs...");

    let mut conn = match utils::env::database::establish_connection().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to establish database connection: {}", e);

            return;
        }
    };

    let (time, ()) = timer
        .time(|| async {
            for website in &websites {
                match indexer::create_page(&mut conn, &website.page).await {
                    Ok(()) => (),
                    Err(e) => error!("Failed to index page: {}", e),
                }

                if let Some(links) = &website.forward_links {
                    for link in links {
                        match indexer::create_forward_link(&mut conn, link).await {
                            Ok(()) => (),
                            Err(e) => error!("Failed to index link: {}", e),
                        }
                    }
                } else {
                    warn!("No forward links found for page: {}", website.page.url);
                };

                for keyword in &website.keywords {
                    match indexer::create_keyword(&mut conn, keyword).await {
                        Ok(()) => (),
                        Err(e) => error!("Failed to index keyword: {}", e),
                    }
                }
            }
        })
        .await;

    info!("Indexed {} URLs in {}!", websites.len(), format_time(&time));
     */
}
