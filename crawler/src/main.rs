use std::sync::mpsc::channel;

use anyhow::Result;
use crawler::Crawler;
use tracing::{info, warn};

mod db;
mod cache;

mod crawler;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let db = db::init().await?;
    let cache = cache::init()?;

    let (sender, reciever) = channel();
    let mut conn = cache.get_connection()?;
    tokio::spawn(async move {
        loop {
            let Ok(url) = reciever.recv() else {
                continue;
            };

            match cache::add_website(&mut conn, &url) {
                Ok(()) => info!("Added {url} to crawl list."),
                Err(why) => warn!("Failed to add {url} to crawl list: {why}"),
            };
        }
    });

    info!("Connected to databases!");
    while let Some(url) = cache::next_website(&mut cache.get_connection()?)? {
        info!("crawlin...");
        let mut crawler = Crawler::new(sender.clone(), url);
        crawler.crawl().await?;
    }

    Ok(())
}
