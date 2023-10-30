use crate::error::Error;
use crate::spiders::Spider;
use crate::util;
use crate::util::robots::RobotFile;
use async_trait::async_trait;
use log::info;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::RwLock;
use url::Url;

/// A web crawler.
///
/// # Fields
///
/// * `http_client`: The HTTP client to use.
/// * `robot_files`: The `robots.txt` files for each domain.
#[derive(Debug)]
pub struct Web {
    http_client: Client,
    robot_files: RwLock<HashMap<String, RobotFile>>,
}

impl Web {
    /// Creates a new web crawler.
    ///
    /// # Returns
    ///
    /// A new web crawler.
    #[allow(clippy::expect_used)]
    pub fn new() -> Self {
        let http_timeout = Duration::from_secs(10);
        let http_client = Client::builder()
            .timeout(http_timeout)
            .build()
            .expect("Failed to build HTTP client!");

        Self {
            http_client,
            robot_files: RwLock::new(HashMap::new()),
        }
    }
}

/// A web page.
///
/// # Fields
///
/// * `url`: The URL of the page.
/// * `html`: The HTML of the page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebItem {
    pub url: Url,
    pub html: String,
}

#[async_trait]
impl Spider for Web {
    type Item = WebItem;

    fn name(&self) -> String {
        String::from("Web")
    }

    #[allow(clippy::expect_used)]
    fn seed_urls(&self) -> Vec<String> {
        util::env::seed_url::fetch().expect("Failed to fetch seed URLs!")
    }

    #[allow(clippy::expect_used)]
    async fn scrape(&self, url: String) -> Result<(Vec<Self::Item>, Vec<String>), Error> {
        // Make sure to respect the robots.txt file.
        let url = Url::from_str(&url)?;
        let robots_url = format!(
            "{}://{}/robots.txt",
            url.scheme(),
            url.host()
                .ok_or_else(|| Error::Reqwest("Failed to get host!".to_string()))?
        );
        let domain = url.domain().unwrap_or_default().to_string();

        let mut robot_files = self.robot_files.write().await;
        let robots = if let Some(robots) = robot_files.get(&domain) {
            info!("Using cached robots.txt file for domain: {}", domain);

            Some(robots.clone())
        } else if let Ok(response) = self.http_client.get(&robots_url).send().await {
            let status = response.status();

            if !status.is_success() {
                return Err(Error::Reqwest(format!(
                    "Failed to fetch robots.txt file: {}",
                    status.canonical_reason().unwrap_or("Unknown")
                )));
            }

            let contents = response.text().await?;
            let robots = util::robots::parse(&contents);

            robot_files.insert(domain.clone(), robots.clone());
            drop(robot_files);

            Some(robots)
        } else {
            None
        };

        // Check if the URL is crawlable, or not.
        if let Some(robots) = robots {
            if !robots.is_crawlable(&url) {
                return Err(Error::Reqwest(format!("URL is not crawlable: {url}")));
            }
        }

        // Actually crawl the page.
        let response = self.http_client.get(url.clone()).send().await?;
        let status = response.status();

        if !status.is_success() {
            return Err(Error::Reqwest(format!(
                "Failed to fetch {}: {}",
                url,
                status.canonical_reason().unwrap_or("Unknown")
            )));
        }

        let body = response.text().await?;
        let document = scraper::Html::parse_document(&body);

        let mut urls = Vec::new();
        for element in document
            .select(&scraper::Selector::parse("a").expect("Failed to parse anchor selector!"))
        {
            let href = element.value().attr("href").unwrap_or("");

            if href.starts_with("http") {
                urls.push(href.to_string());
            }
        }

        Ok((
            vec![WebItem {
                url,
                html: document.html(),
            }],
            urls,
        ))
    }

    async fn process(&self, item: Self::Item) -> Result<(), Error> {
        Ok(())
    }
}
