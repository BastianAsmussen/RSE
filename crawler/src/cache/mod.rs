use anyhow::Result;
use redis::{Client, Commands, Connection};
use reqwest::Url;
use tracing::warn;

pub fn init() -> Result<Client> {
    let url = std::env::var("CACHE_URL").expect("`CACHE_URL` must be set!");
    let client = Client::open(url)?;

    Ok(client)
}

pub fn next_website(conn: &mut Connection) -> Result<Option<Url>> {
    let urls: Vec<String> = conn.get("to_crawl:*")?;
    let Some(choice) = urls.first() else {
        let wikipedia = Url::parse("https://www.wikipedia.org/")?;
        warn!("No websites in crawler cache, using {wikipedia}...");

        return Ok(Some(wikipedia));
    };

    conn.del(format!("to_crawl:{choice}"))?;

    Ok(Some(choice.parse()?))
}

pub fn add_website(conn: &mut Connection, url: &Url) -> Result<()> {
    conn.set(format!("to_crawl:{url}"), 0)?;

    Ok(())
}
