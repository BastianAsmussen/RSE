use anyhow::Result;
use tracing::{error, info, warn};

use spider::{configuration::Configuration, website::Website};

use tokio::time::Instant;

mod db;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let db = db::init().await?;
    info!("Connected to database!");

    let config = Configuration::new().with_respect_robots_txt(true).build();

    let mut to_crawl = vec![String::from("https://www.wikipedia.org/")];
    for url in &to_crawl {
        let mut website = Website::new(&url).with_config(config.to_owned()).build()?;
        website.crawl().await;

        to_crawl.extend(
            website
                .get_links()
                .iter()
                .map(|link| link.inner().to_string())
                .collect::<Vec<_>>(),
        );
    }

    Ok(())
}
