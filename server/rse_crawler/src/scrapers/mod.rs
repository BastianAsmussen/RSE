pub mod web;

use async_trait::async_trait;
use common::errors::Error;
use std::collections::HashMap;
use url::Url;

/// A generic scraper.
///
/// # Type Parameters
///
/// * `Item`: The type of item the scraper scrapes.
///
/// # Methods
///
/// * `seed_urls`: Returns the URLs the scraper starts scraping from.
/// * `scrape`: Scrapes a URL.
/// * `process`: Processes an item.
#[async_trait]
pub trait Scraper: Send + Sync {
    type Item;

    fn seed_urls(&self) -> HashMap<Url, u32>;
    async fn scrape(
        &self,
        url: Url,
        depth: u32,
    ) -> Result<(Vec<Self::Item>, HashMap<Url, u32>), Error>;
    async fn process(&self, item: Self::Item) -> Result<(), Error>;
}
