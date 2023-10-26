pub mod html;

use crate::error::Error;
use async_trait::async_trait;
use reqwest::Url;

/// A spiders that crawls a website.
#[async_trait]
pub trait Spider: Send + Sync {
    type Item;

    fn name(&self) -> String;
    fn seed_urls(&self) -> Vec<Url>;
    async fn scrape(&self, url: &Url) -> Result<(Vec<Self::Item>, Vec<Url>), Error>;
    async fn process(&self, item: Self::Item) -> Result<(), Error>;
}
