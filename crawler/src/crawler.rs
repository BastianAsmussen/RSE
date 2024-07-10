use std::sync::mpsc::Sender;

use anyhow::Result;
use reqwest::Url;
use scraper::{Html, Selector};
use tracing::info;

#[derive(Debug)]
pub struct Crawler {
    sender: Sender<Url>,
    url: Url,
    body: Option<String>,
    found_links: Option<Vec<Url>>,
}

impl Crawler {
    pub const fn new(sender: Sender<Url>, url: Url) -> Self {
        Self {
            sender,
            url,
            body: None,
            found_links: None,
        }
    }

    pub const fn body(&self) -> &Option<String> {
        &self.body
    }

    pub const fn found_links(&self) -> &Option<Vec<Url>> {
        &self.found_links
    }

    pub async fn crawl(&mut self) -> Result<()> {
        if self.is_crawlable().await.is_ok_and(|is_crawlable| !is_crawlable) {
            info!("{} isn't crawlable.", self.url);

            return Ok(());
        }

        let body = reqwest::get(self.url.as_str()).await?.text().await?;
        let found_links = self.find_links(&body);

        for link in found_links.clone() {
            self.sender.send(link)?;
        }

        self.body = Some(body);
        self.found_links = Some(found_links);

        Ok(())
    }

    async fn is_crawlable(&self) -> Result<bool> {
        let robots_url = {
            let mut url = self.url.clone();
            url.set_path("/robots.txt");
            url
        };

        let body = reqwest::get(robots_url).await?.text().await?;
        for line in body.lines() {
            let mut user_agent = String::new();
            let mut crawl_delay = None;

            let mut allowed_sites = Vec::new();
            let mut disallowed_sites = Vec::new();

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let mut parts = line.splitn(2, ':');
            let (Some(key), Some(value)) = (parts.next(), parts.next()) else {
                continue;
            };

            match key {
                "user-agent" => user_agent = value.to_lowercase(),
                "crawl-delay" => crawl_delay = value.parse::<u16>().ok(),
                "allow" => allowed_sites.push(value.to_lowercase()),
                "disallow" => disallowed_sites.push(value.to_lowercase()),
                _ => {}
            };
        }

        Ok(true)
    }

    fn find_links(&self, body: &str) -> Vec<Url> {
        let document = Html::parse_document(body);
        let anchors = Selector::parse("a").expect("Failed to create anchor tag selector!");

        let mut links = Vec::new();
        for element in document.select(&anchors) {
            // If the element has no href, skip it.
            let Some(url) = element.value().attr("href") else {
                continue;
            };

            let Ok(url) = Url::parse(url) else {
                continue;
            };

            let mut new_url = self.url.clone();
            new_url.set_path(url.path());

            // Make sure we only check HTTP(s) sites.
            match new_url.scheme() {
                "http" | "https" => links.push(new_url),
                _ => continue,
            }
        }

        links
    }
}
