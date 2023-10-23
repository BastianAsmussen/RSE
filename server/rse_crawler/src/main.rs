mod crawler;
mod utils;

use log::{error, info};
use db::model::NewPage;

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

    // Map the seed URLs to `NewPage`s.
    let seed_pages = seed_urls
        .iter()
        .map(|url| NewPage {
            url: url.to_string(),
            title: None,
            description: None,
        })
        .collect::<Vec<NewPage>>();

    // Create a new frontier.
    info!("Creating a new frontier...");
    let Ok(mut frontier) = crawler::frontier::Frontier::new(&seed_pages, 100).await else {
        error!("Failed to create a new frontier, exiting...");

        return;
    };

    // Get the next URL to be crawled.
    info!("Getting the next URL to be crawled...");
    let next_page = frontier.get_next_page();
}
