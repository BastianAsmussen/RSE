pub mod web;

use crate::error::Error;
use async_trait::async_trait;

/// A generic spider.
///
/// # Type Parameters
///
/// * `Item`: The type of item the spider scrapes.
///
/// # Methods
///
/// * `name`: Returns the name of the spider.
/// * `seed_urls`: Returns the URLs the spider starts scraping from.
/// * `scrape`: Scrapes a URL.
/// * `process`: Processes an item.
#[async_trait]
pub trait Spider: Send + Sync {
    type Item;

    fn name(&self) -> String;
    fn seed_urls(&self) -> Vec<String>;
    async fn scrape(&self, url: String) -> Result<(Vec<Self::Item>, Vec<String>), Error>;
    async fn process(&self, item: Self::Item) -> Result<(), Error>;
}
