mod utils;
mod data;

use std::time::{SystemTime, UNIX_EPOCH};

use log::{error, info, warn};
use std::time::Instant;
use mysql_async::params;
use mysql_async::prelude::{BatchQuery, WithParams};

use spider::tokio;
use spider::website::Website;
use utils::db_manager::{get_database_url, get_pool};

const SEED_URLS: [&str; 1] = ["https://en.wikipedia.org"];

#[tokio::main]
async fn main() {
    env_logger::init();

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

    for url in SEED_URLS {
        info!("Crawling {url}...");

        let mut website: Website = Website::new("https://en.wikipedia.org");
        website.with_respect_robots_txt(true)
            .with_subdomains(true)
            .with_tld(false)
            .with_delay(0)
            .with_request_timeout(None)
            .with_http2_prior_knowledge(false)
            .with_user_agent(Some("RSE Crawler"))
            .with_budget(Some(spider::hashbrown::HashMap::from([("*", 300), ("/licenses", 10)])))
            .with_external_domains(Some(Vec::from(["https://creativecommons.org/licenses/by/3.0/"].map(std::string::ToString::to_string)).into_iter()))
            .with_headers(None)
            .with_blacklist_url(Some(Vec::from(["https://choosealicense.com/licenses/".into()])))
            .with_proxies(None);

        let start = Instant::now();
        website.scrape().await;
        let duration = start.elapsed();

        let Some(pages) = website.get_pages() else {
            warn!("No pages were scraped for {url}!");

            continue;
        };

        info!(
            "Scraping {url} took {:?} for {:?} total pages.",
            duration,
            pages.len()
        );

        let Ok(mut conn) = pool.get_conn().await else {
            error!("Failed to get a connection to the database!");

            continue;
        };

        let result =
            r"INSERT INTO sites(url, is_accurate)
              VALUES(:url, :is_accurate);
              INSERT INTO data(site_id, last_updated, html)
              VALUES(sites.LAST_INSERT_ID(), :last_updated, :html)"
            .with(pages.clone().into_iter().map(|page| params ! {
                "url" => page.get_url(),
                "is_accurate" => 1,
                "last_updated" => SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .expect("Time went backwards!"),
                "html" => page.get_html(),
            }))
            .batch(&mut conn)
            .await;

        if let Err(err) = result {
            error!("Failed to insert sites into the database: {:?}", err);
        }
    }
}
