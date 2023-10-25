use crate::error::Error;
use crate::utils;
use async_trait::async_trait;
use log::{info};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use db::model::NewMetadata;

/// A spider that crawls a website.
///
/// # Fields
///
/// * `client` - The HTTP client.
/// * `regex` - The regular expression used to extract URLs.
/// * `expected_results` - The number of results the spider expects.
#[derive(Debug)]
pub struct WebSpider {
    http_client: Client,
    regex: Regex,
}

impl Default for WebSpider {
    fn default() -> Self {
        Self::new(
            utils::env::spider::get_http_timeout(),
            utils::env::spider::get_url_regex(),
        )
    }
}

impl WebSpider {
    /// Creates a new HTML spider.
    ///
    /// # Arguments
    ///
    /// * `http_timeout` - The HTTP timeout.
    /// * `url_regex` - The regular expression used to extract URLs.
    #[allow(clippy::expect_used)]
    pub fn new(http_timeout: Duration, url_regex: Regex) -> Self {
        let http_client = Client::builder()
            .timeout(http_timeout)
            .build()
            .expect("Failed to build HTTP client!");
        let regex = url_regex;

        Self { http_client, regex }
    }
}

/// A scraped website.
///
/// # Fields
///
/// * `url` - The URL of the website.
/// * `metadata` - The metadata of the website.
/// * `forward_links` - The forward links of the website.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebItem {
    url: String,
    metadata: HashMap<String, String>,
    forward_links: HashSet<String>,
}

#[async_trait]
impl super::Spider for WebSpider {
    type Item = WebItem;

    fn name(&self) -> String {
        "WebSpider".to_string()
    }

    #[allow(clippy::expect_used)]
    fn seed_urls(&self) -> Vec<String> {
        utils::env::seed_url::fetch().expect("Failed to fetch seed URLs!")
    }

    async fn scrape(&self, url: &str) -> Result<(Vec<Self::Item>, Vec<String>), super::Error> {
        let response = self.http_client.get(url).send().await?;

        let html = response.text().await?;

        let mut urls = Vec::new();

        for capture in self.regex.captures_iter(&html) {
            if let Some(url) = capture.get(1) {
                info!("Found URL: {}", url.as_str());

                urls.push(url.as_str().to_string());
            }
        }

        // let urls = filter_non_scrapable(urls).await?;
        let urls = normalize_urls(url, urls);

        let items = vec![WebItem {
            url: url.to_string(),
            metadata: get_metadata(&html)?,
            forward_links: urls.iter().cloned().collect(),
        }];

        Ok((items, urls))
    }

    async fn process(&self, item: Self::Item) -> Result<(), super::Error> {
        info!("Processing URL: {}", item.url);

        // Insert the item into the database.
        let mut conn = db::get_connection().await?;

        let page = db::create_page(&mut conn, &item.url).await?;
        let metadata = item.metadata.iter().map(|metadata| NewMetadata {
            page_id: page.id,

            name: metadata.0.to_string(),
            content: metadata.1.to_string(),
        }).collect::<Vec<NewMetadata>>();
        db::create_metadata(&mut conn, &metadata).await?;

        let forward_links = item.forward_links.iter().map(|forward_link| db::model::NewForwardLink {
            page_id: page.id,

            url: forward_link.to_string(),
        }).collect::<Vec<db::model::NewForwardLink>>();
        db::create_links(&mut conn, &forward_links).await?;

        Ok(())
    }
}

/// Filters out non-scrapable URLs.
///
/// # Arguments
///
/// * `urls` - The URLs.
///
/// # Returns
///
/// * `Ok(Vec<String>)` - The scrapable URLs.
/// * `Err(Error)` - The error.
async fn filter_non_scrapable(urls: Vec<String>) -> Result<Vec<String>, Error> {
    todo!("Implement filter_non_scrapable!")
}

/// Extracts the root URL from a URL.
///
/// # Example
///
/// ```
/// use rse_crawler::spiders::html::get_root_url;
///
/// let url = "https://www.example.com/path/to/page";
/// let root_url = get_root_url(url).unwrap();
///
/// assert_eq!(root_url, "https://www.example.com");
/// ```
///
/// # Arguments
///
/// * `url` - The URL.
///
/// # Returns
///
/// * `Ok(String)` - The root URL.
/// * `Err(Error)` - The error.
#[allow(clippy::expect_used)]
fn get_root_url(url: &str) -> Result<String, Error> {
    let regex = Regex::new(r"^(https?://[^/]+)")?;
    let Some(captures) = regex.captures(url) else {
        return Err(Error::Internal("Failed to get captures!".to_string()));
    };

    captures.get(1).map_or_else(
        || Err(Error::Internal("Failed to get root URL!".to_string())),
        |root_url| Ok(root_url.as_str().to_string()),
    )
}

/// Normalizes the URLs.
///
/// # Arguments
///
/// * `origin` - The origin URL.
/// * `urls` - The URLs.
///
/// # Returns
///
/// * `Vec<String>` - The normalized URLs.
#[allow(clippy::expect_used)]
fn normalize_urls(origin: &str, urls: Vec<String>) -> Vec<String> {
    let mut normalized_urls = Vec::new();

    for url in urls {
        // Check if a given URL refers to an absolute or relative path.
        let is_absolute = url.starts_with("http://") || url.starts_with("https://");

        // If the URL is absolute, then we can just add it to the list of normalized URLs.
        if is_absolute {
            normalized_urls.push(url);

            continue;
        }

        // If the URL is relative, then we need to normalize it.
        let root_url = get_root_url(origin).expect("Failed to get root URL!");

        // Filter the URLs, so that they don't end or start with a slash.
        let root_url = root_url.trim_end_matches('/');
        let url = url.trim_start_matches('/');

        // Make sure no illegal characters are in the URL.
        let illegal_characters = ["#", "?"];
        if illegal_characters.iter().any(|illegal_character| url.contains(illegal_character)) {
            continue;
        }

        normalized_urls.push(format!("{root_url}/{url}"));
    }

    normalized_urls
}

/// Extracts the metadata from the HTML.
///
/// # Arguments
///
/// * `html` - The HTML.
///
/// # Returns
///
/// * `Ok(HashMap<String, String>)` - The metadata.
/// * `Err(Error)` - The error.
fn get_metadata(html: &str) -> Result<HashMap<String, String>, super::Error> {
    let document = scraper::Html::parse_document(html);

    let selector = scraper::Selector::parse("meta")?;
    let meta = document.select(&selector);
    let mut meta_map = HashMap::new();

    for element in meta {
        let Some(name) = element.value().attr("name") else {
            continue;
        };
        let Some(content) = element.value().attr("content") else {
            continue;
        };

        meta_map.insert(name.to_string(), content.to_string());
    }

    Ok(meta_map)
}
