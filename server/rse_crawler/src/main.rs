mod utils;

use log::{error, info, warn};

use std::time::{Instant};

use crate::utils::html::index_pages;

use spider::tokio;
use spider::website::Website;
use utils::db_manager::{get_database_url, get_pool};
use utils::seed_urls::get_seed_urls;

#[tokio::main]
async fn main() {
    env_logger::init();

    // Fetch seed URLs.
    info!("Fetching seed URLs...");

    let Some(seed_urls) = get_seed_urls() else {
        panic!("Failed to fetch seed URLs!");
    };

    info!("Loaded {} seed URLs!", seed_urls.len());

    // Establish database connection.
    info!("Connecting to the database...");

    let Ok(url) = get_database_url() else {
        panic!("Missing environment variables!");
    };
    let Ok(pool) = get_pool(&url).await else {
        panic!("Failed to connect to the database!");
    };

    info!("Successfully connected to the database!");

    // Crawl seed URLs.
    info!("Crawling seed URLs...");

    for url in seed_urls {
        info!("Crawling {url}...");

        let mut website: Website = Website::new(&url);
        website
            .with_respect_robots_txt(true)
            .with_subdomains(true)
            .with_tld(false)
            .with_delay(0)
            .with_request_timeout(None)
            .with_http2_prior_knowledge(false)
            .with_user_agent(Some("RSE Crawler"))
            .with_budget(Some(spider::hashbrown::HashMap::from([
                ("*", 300),
                ("/licenses", 10),
            ])))
            .with_external_domains(Some(
                Vec::from(
                    ["https://creativecommons.org/licenses/by/3.0/"]
                        .map(std::string::ToString::to_string),
                )
                .into_iter(),
            ))
            .with_headers(None)
            .with_blacklist_url(Some(Vec::from([
                "https://choosealicense.com/licenses/".into()
            ])))
            .with_proxies(None);

        let start = Instant::now();
        website.scrape().await;
        let duration = start.elapsed();

        let Some(pages) = website.get_pages() else {
            warn!("No pages found for {url}!");

            continue;
        };

        info!(
            "Scraping {url} took {:?}, found {:?} total pages.",
            duration, pages
        );

        let Ok(mut conn) = pool.get_conn().await else {
            error!("Failed to get a connection to the database!");

            continue;
        };

        info!("Indexing pages...");

        let metadata = index_pages(&mut conn, pages).await;

        info!("Indexed {} pages!", metadata.len());
    }
}
