use crate::robots::RobotsFile;
use crate::scrapers::Scraper;
use async_trait::async_trait;
use error::Error;
use log::{debug, error, info, warn};
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::RwLock;
use url::Url;

/// A scraper for websites.
///
/// # Fields
///
/// * `http_client` - The HTTP client to use.
/// * `max_depth` - The maximum depth to crawl to, if any.
/// * `robots_cache` - The cache of `robots.txt` files.
#[derive(Debug)]
pub struct Web {
    http_client: Client,
    max_depth: Option<u32>,
    robots_cache: RwLock<HashMap<String, RobotsFile>>,
}

impl Web {
    /// Creates a new `WebScraper`.
    ///
    /// # Arguments
    ///
    /// * `http_client` - The HTTP client to use.
    /// * `max_depth` - The maximum depth to crawl to, if any.
    pub fn new(http_client: Client, max_depth: Option<u32>) -> Self {
        Self {
            http_client,
            max_depth,
            robots_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Gets the `robots.txt` file for a given URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to get the `robots.txt` file for.
    /// * `depth` - The current depth of the crawl.
    ///
    /// # Returns
    ///
    /// * `Result<RobotsFile, Error>` - The parsed `robots.txt` file.
    async fn get_robots_file(&self, url: &Url) -> Result<RobotsFile, Error> {
        let robots_url = Url::from_str(&format!(
            "{}://{}/robots.txt",
            url.scheme(),
            url.host()
                .ok_or_else(|| Error::Reqwest(format!("Failed to get host for \"{url}\"")))?
        ))?;
        let domain = url.domain().unwrap_or_default().to_string();

        if let Some(robots_file) = self.robots_cache.write()?.get(&domain) {
            info!("Using cached robots.txt file for \"{url}\"...");

            return Ok(robots_file.clone());
        }

        let response = self.http_client.get(robots_url).send().await?;
        let body = response.text().await?;

        info!("Parsing robots.txt file for \"{url}\"...");
        let robots_file = RobotsFile::parse(&body);

        self.robots_cache
            .write()?
            .insert(domain, robots_file.clone());

        Ok(robots_file)
    }

    /// Extracts all links from the given HTML body.
    ///
    /// # Arguments
    ///
    /// * `body` - The HTML body to extract links from.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Url>, Error>` - The extracted links.
    pub fn extract_links(body: &str) -> Result<Vec<Url>, Error> {
        let mut links = Vec::new();

        let document = Html::parse_document(body);
        let selector = Selector::parse("a")?;
        for element in document.select(&selector) {
            // If the element has no href, skip it.
            let Some(link) = element.value().attr("href") else {
                continue;
            };

            // If the link fails to parse, skip it.
            let Ok(url) = Url::from_str(link) else {
                continue;
            };

            // If the link has no scheme, skip it.
            if url.scheme().is_empty() {
                continue;
            }

            links.push(url);
        }

        Ok(links)
    }

    /// Checks if the given depth has been reached.
    ///
    /// # Arguments
    ///
    /// * `depth` - The current depth of the crawl.
    ///
    /// # Returns
    ///
    /// * `bool` - Whether the max depth has been reached, or not.
    const fn has_reached_max_depth(&self, depth: u32) -> bool {
        if let Some(max_depth) = self.max_depth {
            depth >= max_depth
        } else {
            false
        }
    }
}

#[async_trait]
impl Scraper for Web {
    type Item = Website;

    #[allow(clippy::expect_used)]
    fn seed_urls(&self) -> HashMap<Url, u32> {
        let seed_urls = utils::env::data::fetch_seed_urls().expect("Failed to fetch seed URLs!");

        seed_urls
            .into_iter()
            .map(|url| (url, 0))
            .collect::<HashMap<_, _>>()
    }

    /// Scrapes the given URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to scrape.
    /// * `depth` - The current depth of the crawl.
    ///
    /// # Returns
    ///
    /// * `Result<(Vec<Self::Item>, (Vec<Url>, u32)), Error>` - The scraped items and new URLs.
    async fn scrape(
        &self,
        url: Url,
        depth: u32,
    ) -> Result<(Vec<Self::Item>, HashMap<Url, u32>), Error> {
        if self.has_reached_max_depth(depth) {
            warn!("Reached max depth, skipping \"{url}\"...");

            return Ok((Vec::new(), HashMap::new()));
        }

        debug!("Current Depth: {depth}");

        info!("Getting robots.txt file for \"{url}\"...");
        match self.get_robots_file(&url).await {
            Ok(robots_file) => {
                if !robots_file.is_crawlable(&url) {
                    warn!("\"{url}\" is not crawlable, skipping...");

                    return Ok((Vec::new(), HashMap::new()));
                }
            }
            Err(err) => {
                error!(
                    "Failed to get robots.txt file for \"{url}\"! \
                        Error: {err}"
                );

                return Ok((Vec::new(), HashMap::new()));
            }
        };

        info!("Getting body of \"{url}\"...");
        let response = self.http_client.get(url.to_string()).send().await?;
        let body = response.text().await?;

        info!("Extracting links from \"{url}\"...");
        let links = Self::extract_links(&body)?;

        Ok((
            vec![Website {
                url: url.clone(),
                title: None,
                description: None,
                links: Some(links.clone()),
            }],
            links
                .into_iter()
                .map(|url| (url, depth + 1))
                .collect::<HashMap<_, _>>(),
        ))
    }

    async fn process(&self, item: Self::Item) -> Result<(), Error> {
        info!("Processing \"{}\"...", item.url);
        info!("Title: {}", item.title.unwrap_or_else(|| "Unknown".into()));
        info!(
            "Description: {}",
            item.description
                .unwrap_or_else(|| "Unknown".into())
                .replace('\n', " ")
        );
        info!("Links: {}", item.links.unwrap_or_default().len());

        Ok(())
    }
}

/// A scraped website.
///
/// # Fields
///
/// * `url` - The URL of the website.
/// * `title` - The title of the website, if any.
/// * `description` - The description of the website, if any.
/// * `links` - The links on the website, if any.
pub struct Website {
    pub url: Url,
    pub title: Option<String>,
    pub description: Option<String>,
    pub links: Option<Vec<Url>>,
}
