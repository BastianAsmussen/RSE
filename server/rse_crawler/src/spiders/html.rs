use crate::error::Error;
use crate::utils;
use async_trait::async_trait;
use db::model::NewMetadata;
use log::info;
use regex::Regex;
use reqwest::{Client, Url};
use scraper::Html;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use rust_stemmers::Stemmer;

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
/// * `keywords` - The keywords of the website.
/// * `forward_links` - The forward links of the website.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebItem {
    url: Url,
    metadata: HashMap<String, String>,
    keywords: HashMap<String, i32>,
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
        let document = Html::parse_document(&html);

        let items = vec![WebItem {
            url: url.clone(),
            metadata: get_metadata(&document)?,
            keywords: get_keywords(&document)?,
            forward_links: urls.clone(),
        }];

        Ok((items, urls))
    }

    #[allow(clippy::expect_used)]
    async fn process(&self, item: Self::Item) -> Result<(), super::Error> {
        info!("Processing URL: {}", item.url);

        let mut conn = db::get_connection().await?;

        info!("Inserting page...");
        let page = db::create_page(&mut conn, &item.url).await?;

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

        info!("Inserting {} keywords...", item.keywords.len());

        item.keywords
            .iter()
            .max_by(|a, b| a.0.len().cmp(&b.0.len()))
            .map_or_else(
                || {
                    info!("No keywords found!");
                },
                |keyword| {
                    info!("Longest Keyword:");
                    info!("- Word:\t{}", keyword.0);
                    info!("- Length:\t{}", keyword.0.len());
                    info!("- Frequency:\t{}", keyword.1);
                },
            );

        let keywords = item
            .keywords
            .iter()
            .map(|keyword| db::model::NewKeyword {
                page_id: page.id,

                word: keyword.0.to_string(),
                frequency: *keyword.1,
            })
            .collect::<Vec<db::model::NewKeyword>>();
        db::create_keywords(&mut conn, &keywords).await?;

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
/// * `document` - The HTML document.
///
/// # Returns
///
/// * `Ok(HashMap<String, String>)` - The metadata.
/// * `Err(Error)` - The error.
#[allow(clippy::expect_used)]
fn get_metadata(document: &Html) -> Result<HashMap<String, String>, super::Error> {
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

    // Get the title of the page.
    let selector = scraper::Selector::parse("title")?;
    let title = document.select(&selector).next();
    if let Some(title) = title {
        let title = title.text().collect::<Vec<_>>().join(" ");

        meta_map.insert("title".to_string(), title);
    }

    // Get the language of the page.
    let selector = scraper::Selector::parse("html")?;
    let language = document.select(&selector).next();
    if let Some(language) = language {
        let value = language.value().attr("lang").unwrap_or("en");

        meta_map.insert("language".to_string(), value.to_string());
    };

    Ok(meta_map)
}

/// Extracts the keywords from the HTML.
///
/// # Arguments
///
/// * `document` - The HTML document.
///
/// # Returns
///
/// * `HashMap<String, i32>` - The keywords.
#[allow(clippy::expect_used)]
fn get_keywords(document: &Html) -> Result<HashMap<String, i32>, super::Error> {
    // Get the language of the page or default to English.
    let language = document.root_element().value().attr("lang").unwrap_or("en");

    // What counts as a keyword? Any word that is not a stop word, and is a purely alphabetical word, or purely numeric word.
    // Grab all the words in the page, filter out tags, and other garbage, and count how many times they appear.
    let selector = scraper::Selector::parse("body")?;
    let Some(body) = document.select(&selector).next() else {
        return Err(super::Error::Scraper("No body found!".to_string()));
    };

    let words = body
        .text()
        .flat_map(|words| words.split_whitespace().collect::<Vec<_>>())
        .map(str::to_lowercase)
        .map(|word| {
            let mut word = word;

            word.retain(|character| character.is_alphabetic() || character.is_numeric());
            word
        })
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();

    let stop_words = get_stop_words();
    let stemmer = Stemmer::create(determine_stemmer_algorithm(language));

    let keywords = words
        .iter()
        .filter(|word| !stop_words.contains(word))
        .map(|word| {
            let stemmed_word = stemmer.stem(word);

            stemmed_word.to_string()
        })
        .fold(HashMap::new(), |mut map, word| {
            if let std::collections::hash_map::Entry::Vacant(e) = map.entry(word.clone()) {
                e.insert(1);
            } else {
                let frequency = map.get_mut(&word).expect("Failed to get keyword frequency!");

                *frequency += 1;
            }

            map
        });

    Ok(keywords)
}

/// Gets the stop words.
///
/// # Returns
///
/// * `Vec<String>` - The stop words.
///
/// # Panics
///
/// * If the stop words file could not be read.
#[allow(clippy::expect_used)]
fn get_stop_words() -> Vec<String> {
    let stop_words =
        std::fs::read_to_string("stop_words.txt").expect("Failed to read stop words file!");

    stop_words
        .split('\n')
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
}

/// Determines the stemmer algorithm to use.
///
/// # Arguments
///
/// * `language` - The language of the page.
///
/// # Returns
///
/// * `Algorithm` - The stemmer algorithm.
///
/// # Notes
///
/// * If the language is not supported, the English stemmer algorithm is used.
fn determine_stemmer_algorithm(language: &str) -> rust_stemmers::Algorithm {
    // Determine the language of the page and choose the stemmer accordingly.
    match language {
        "ar" => rust_stemmers::Algorithm::Arabic,
        "da" => rust_stemmers::Algorithm::Danish,
        "nl" => rust_stemmers::Algorithm::Dutch,
        "fi" => rust_stemmers::Algorithm::Finnish,
        "fr" => rust_stemmers::Algorithm::French,
        "de" => rust_stemmers::Algorithm::German,
        "el" => rust_stemmers::Algorithm::Greek,
        "hu" => rust_stemmers::Algorithm::Hungarian,
        "it" => rust_stemmers::Algorithm::Italian,
        "no" => rust_stemmers::Algorithm::Norwegian,
        "pt" => rust_stemmers::Algorithm::Portuguese,
        "ro" => rust_stemmers::Algorithm::Romanian,
        "ru" => rust_stemmers::Algorithm::Russian,
        "es" => rust_stemmers::Algorithm::Spanish,
        "sv" => rust_stemmers::Algorithm::Swedish,
        "ta" => rust_stemmers::Algorithm::Tamil,
        "tr" => rust_stemmers::Algorithm::Turkish,
        _ => rust_stemmers::Algorithm::English,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_keywords() {
        let html = r#"
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta charset="UTF-8">
                    <meta name="description" content="This is a description.">
                    <meta name="keywords" content="this, is, a, keyword, list">
                    <meta name="author" content="John Doe">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>LinkedIn</title>
                </head>
                <body>
                    <h1>LinkedIn</h1>
                    <p>500 million+ members | Manage your professional identity. Build and engage with your professional network. Access knowledge, insights and opportunities.</p>
                </body>
            </html>
        "#;

        let document = Html::parse_document(html);

        let keywords = get_keywords(&document).expect("Failed to get keywords!");
        let length = keywords.len();

        assert_eq!(length, 14);

        assert_eq!(keywords.get("ident"), Some(&1));
        assert_eq!(keywords.get("knowledg"), Some(&1));
        assert_eq!(keywords.get("linkedin"), Some(&1));
        assert_eq!(keywords.get("500"), Some(&1));
        assert_eq!(keywords.get("access"), Some(&1));
        assert_eq!(keywords.get("network"), Some(&1));
        assert_eq!(keywords.get("insight"), Some(&1));
        assert_eq!(keywords.get("opportun"), Some(&1));
        assert_eq!(keywords.get("million"), Some(&1));
        assert_eq!(keywords.get("profession"), Some(&2));
        assert_eq!(keywords.get("manag"), Some(&1));
        assert_eq!(keywords.get("engag"), Some(&1));
        assert_eq!(keywords.get("member"), Some(&1));
        assert_eq!(keywords.get("build"), Some(&1));
    }
}
