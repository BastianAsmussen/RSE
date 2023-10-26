use crate::error::Error;
use crate::utils;
use async_trait::async_trait;
use db::model::NewMetadata;
use log::info;
use regex::Regex;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

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
    url: Url,
    metadata: HashMap<String, String>,
    forward_links: Vec<Url>,
}

#[async_trait]
impl super::Spider for WebSpider {
    type Item = WebItem;

    fn name(&self) -> String {
        "WebSpider".to_string()
    }

    #[allow(clippy::expect_used)]
    fn seed_urls(&self) -> Vec<Url> {
        let seed_urls = utils::env::seed_url::fetch().expect("Failed to fetch seed URLs!");

        seed_urls
            .iter()
            .map(|seed_url| Url::parse(seed_url).expect("Failed to parse seed URL!"))
            .collect()
    }

    async fn scrape(&self, url: &Url) -> Result<(Vec<Self::Item>, Vec<Url>), super::Error> {
        let response = self.http_client.get(url.as_str()).send().await?;

        let html = response.text().await?;

        let mut urls = Vec::new();
        for capture in self.regex.captures_iter(&html) {
            if let Some(captured_url) = capture.get(1) {
                let normalized_url = normalize_url(url, captured_url.as_str())?;

                urls.push(normalized_url.clone());

                info!("Found URL: {normalized_url}");
            }
        }

        // let urls = filter_non_scrapable(urls).await?;

        let items = vec![WebItem {
            url: url.clone(),
            metadata: get_metadata(&html)?,
            forward_links: urls.clone(),
        }];

        Ok((items, urls))
    }

    #[allow(clippy::expect_used)]
    async fn process(&self, item: Self::Item) -> Result<(), super::Error> {
        info!("Processing URL: {}", item.url);

        let mut conn = db::get_connection().await?;

        let page = db::create_page(&mut conn, &item.url).await?;

        info!("Inserted page...");
        let metadata = item
            .metadata
            .iter()
            .map(|metadata| NewMetadata {
                page_id: page.id,

                name: metadata.0.to_string(),
                content: metadata.1.to_string(),
            })
            .collect::<Vec<NewMetadata>>();

        info!("Inserting {} metadata tags...", metadata.len());
        db::create_metadata(&mut conn, &metadata).await?;

        // Count how many times each forward link appears, and add them to the HashMap.
        let forward_links =
            item.forward_links
                .into_iter()
                .fold(HashMap::new(), |mut map, forward_link| {
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        map.entry(forward_link.clone())
                    {
                        e.insert(1);
                    } else {
                        let frequency = map
                            .get_mut(&forward_link)
                            .expect("Failed to get forward link frequency!");
                        *frequency += 1;
                    }

                    map
                });

        info!("Inserting {} forward links...", forward_links.len());
        db::create_links(&mut conn, &item.url, &forward_links).await?;

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
/// * `Ok(Vec<Url>)` - The scrapable URLs.
/// * `Err(Error)` - The error.
async fn filter_non_scrapable(urls: Vec<Url>) -> Result<Vec<Url>, Error> {
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
/// let root_url = get_root_url(url);
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
/// * `Ok(Url)` - The root URL.
/// * `Err(Error)` - The error.
fn get_root_url(url: Url) -> Url {
    let mut root_url = url;

    root_url.set_path("");
    root_url.set_query(None);
    root_url.set_fragment(None);

    root_url
}

/// Normalizes the URL.
///
/// # Arguments
///
/// * `origin_url` - The origin URL.
/// * `to_normalize` - The URL to normalize.
///
/// # Returns
///
/// * `Ok(Url)` - The normalized URL.
/// * `Err(Error)` - The error.
#[allow(clippy::expect_used)]
fn normalize_url(origin_url: &Url, to_normalize: &str) -> Result<Url, Error> {
    let mut normalized_url = origin_url.join(to_normalize)?;

    // If the normalized URL has an extension, return an error.
    if normalized_url.path_segments().is_some() {
        let path_segments = normalized_url
            .path_segments()
            .expect("Failed to get path segments!");

        if path_segments
            .last()
            .expect("Failed to get last path segment!")
            .contains('.')
        {
            return Err(Error::Url(format!(
                "URL has an extension: {normalized_url}"
            )));
        }
    };

    normalized_url.set_fragment(None);
    normalized_url.set_query(None);

    Ok(normalized_url)
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
