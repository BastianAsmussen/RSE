use crate::robots;
use crate::robots::RobotFile;
use crate::spiders::Spider;
use async_trait::async_trait;
use database::model::NewKeyword;
use error::Error;
use html5ever::tree_builder::TreeSink;
use log::{debug, error, info};
use reqwest::header::HeaderValue;
use reqwest::Client;
use scraper::Html;
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
/// * `boundaries`: The boundaries for the words.
#[derive(Debug)]
pub struct Web {
    http_client: Client,
    robot_files: RwLock<HashMap<String, RobotFile>>,
    word_boundaries: (usize, usize, usize, usize),
}

impl Web {
    /// Creates a new web crawler.
    ///
    /// # Arguments
    ///
    /// * `http_timeout`: The HTTP timeout.
    /// * `user_agent`: The user agent to use.
    /// * `word_boundaries`: The boundaries for the words, in order: minimum word frequency, maximum word frequency, minimum word length, maximum word length.
    ///
    /// # Returns
    ///
    /// A new web crawler.
    #[allow(clippy::expect_used)]
    pub fn new(
        http_timeout: &Duration,
        user_agent: &HeaderValue,
        word_boundaries: (usize, usize, usize, usize),
    ) -> Self {
        let http_client = Client::builder()
            .timeout(*http_timeout)
            .user_agent(user_agent)
            .build()
            .expect("Failed to build HTTP client!");

        Self {
            http_client,
            robot_files: RwLock::new(HashMap::new()),
            word_boundaries,
        }
    }
}

/// A web page.
///
/// # Fields
///
/// * `url`: The URL of the page.
/// * `forward_links`: The outbound URLs of the page.
/// * `raw_html`: The raw HTML of the page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub url: Url,
    pub forward_links: Vec<Url>,
    pub raw_html: String,
}

impl Page {
    /// Gets the title of a page.
    ///
    /// # Arguments
    ///
    /// * `html`: The HTML document to get the title from.
    ///
    /// # Returns
    ///
    /// * `Option<String>`: The title of the page.
    ///
    /// # Panics
    ///
    /// * If the title selector fails to parse.
    #[allow(clippy::expect_used)]
    fn get_title(html: &str) -> Option<String> {
        Html::parse_document(html)
            .select(&scraper::Selector::parse("title").expect("Failed to parse title selector!"))
            .next()
            .map(|element| element.inner_html().trim().to_string())
    }

    /// Gets the description of a page.
    ///
    /// # Arguments
    ///
    /// * `html`: The HTML document to get the description from.
    ///
    /// # Returns
    ///
    /// * `Option<String>`: The description of the page.
    ///
    /// # Panics
    ///
    /// * If the description selector fails to parse.
    #[allow(clippy::expect_used)]
    fn get_description(html: &str) -> Option<String> {
        Html::parse_document(html)
            .select(
                &scraper::Selector::parse("meta[name=description]")
                    .expect("Failed to parse description selector!"),
            )
            .next()
            .map(|element| element.inner_html().trim().to_string())
    }

    /// Gets the language of a page.
    ///
    /// # Arguments
    ///
    /// * `document`: The HTML document to get the language from.
    ///
    /// # Returns
    ///
    /// * `Option<String>`: The language of the page.
    ///
    /// # Panics
    ///
    /// * If the HTML selector fails to parse.
    #[allow(clippy::expect_used)]
    fn get_language(html: &str) -> Option<String> {
        Html::parse_document(html)
            .select(&scraper::Selector::parse("html").expect("Failed to parse HTML selector!"))
            .next()
            .and_then(|element| element.value().attr("lang"))
            .map(std::string::ToString::to_string)
    }

    /// Gets the keywords of a page.
    ///
    /// # Arguments
    ///
    /// * `html`: The HTML document to get the keywords from.
    ///
    /// # Returns
    ///
    /// * `Option<Vec<String>>`: The keywords of the page.
    ///
    /// # Panics
    ///
    /// * If the keywords selector fails to parse.
    #[allow(clippy::expect_used)]
    fn get_keywords(html: &str) -> Option<Vec<String>> {
        Html::parse_document(html)
            .select(
                &scraper::Selector::parse("meta[name=keywords]")
                    .expect("Failed to parse keywords selector!"),
            )
            .next()
            .and_then(|element| element.value().attr("content"))
            .map(|keywords| {
                keywords
                    .split(',')
                    .map(|keyword| keyword.trim().to_string())
                    .collect()
            })
    }

    /// Gets the "spoken" words on a page, excluding HTML tags.
    ///
    /// # Arguments
    ///
    /// * `html`: The HTML document to get the words from.
    /// * `language`: The language of the page.
    /// * `bounds`: The bounds of the words.
    ///
    /// # Returns
    ///
    /// * `Result<HashMap<String, usize>, Error>`: The words on the page.
    ///
    /// # Errors
    ///
    /// * If the minimum length is greater than the maximum length.
    ///
    /// # Panics
    ///
    /// * If the illegal characters regex fails to compile.
    /// * If the body selector fails to parse.
    #[allow(clippy::expect_used)]
    fn get_words(
        html: &str,
        language: Option<&str>,
        boundaries: (usize, usize, usize, usize),
    ) -> Result<HashMap<String, usize>, Error> {
        let (minimum_frequency, maximum_frequency, minimum_length, maximum_length) = boundaries;

        if minimum_length > maximum_length {
            return Err(Error::InvalidBoundaries(
                "Minimum length cannot be greater than maximum length!".into(),
            ));
        }
        if minimum_frequency > maximum_frequency {
            return Err(Error::InvalidBoundaries(
                "Minimum frequency cannot be greater than maximum frequency!".into(),
            ));
        }

        let mut document = Html::parse_document(html);

        // Remove script and style tags.
        let selector =
            scraper::Selector::parse("script, style").expect("Failed to parse selector!");
        let node_ids = document
            .select(&selector)
            .map(|x| x.id())
            .collect::<Vec<_>>();
        for node_id in node_ids {
            document.remove_from_parent(&node_id);
        }

        // Get the text from the body.
        let selector = scraper::Selector::parse("body").expect("Failed to parse body selector!");
        let element = document
            .select(&selector)
            .next()
            .expect("Failed to get body!");
        let text = &element.text().collect::<Vec<_>>().join(" ");

        // Get the language of the page, or default to English.
        let language = language.unwrap_or("en");
        let language = match language {
            "ar" => rust_stemmers::Algorithm::Arabic,
            "da" => rust_stemmers::Algorithm::Danish,
            "nl" => rust_stemmers::Algorithm::Dutch,
            "fi" => rust_stemmers::Algorithm::Finnish,
            "fr" => rust_stemmers::Algorithm::French,
            "de" => rust_stemmers::Algorithm::German,
            "hu" => rust_stemmers::Algorithm::Hungarian,
            "it" => rust_stemmers::Algorithm::Italian,
            "no" => rust_stemmers::Algorithm::Norwegian,
            "pt" => rust_stemmers::Algorithm::Portuguese,
            "ro" => rust_stemmers::Algorithm::Romanian,
            "ru" => rust_stemmers::Algorithm::Russian,
            "es" => rust_stemmers::Algorithm::Spanish,
            "sv" => rust_stemmers::Algorithm::Swedish,
            "tr" => rust_stemmers::Algorithm::Turkish,
            _ => rust_stemmers::Algorithm::English,
        };

        // Get the words from the text, stem, filter and count them.
        let mut words = utils::words::extract(text, language);

        words.retain(|_, frequency| {
            *frequency >= minimum_frequency && *frequency <= maximum_frequency
        });
        words.retain(|word, _| word.len() >= minimum_length && word.len() <= maximum_length);

        Ok(words)
    }
}

#[async_trait]
impl Spider for Web {
    type Item = Page;

    fn name(&self) -> String {
        String::from("Web")
    }

    #[allow(clippy::expect_used)]
    fn seed_urls(&self) -> Vec<String> {
        utils::env::data::fetch_seed_urls().expect("Failed to fetch seed URLs!")
    }

    #[allow(clippy::expect_used)]
    async fn scrape(&self, url: String) -> Result<(Vec<Self::Item>, Vec<String>), Error> {
        // Make sure to respect the robots.txt file.
        let url = Url::from_str(&url)?;
        let robots_url = format!(
            "{}://{}/robots.txt",
            url.scheme(),
            url.host()
                .ok_or_else(|| Error::Reqwest("Failed to get host!".into()))?
        );
        let domain = url.domain().unwrap_or_default().to_string();

        let mut robot_files = self.robot_files.write().await;
        let robots = if let Some(robots) = robot_files.get(&domain) {
            info!("Using cached robots.txt file for domain: {}", domain);

            Some(robots.clone())
        } else if let Ok(response) = self.http_client.get(&robots_url).send().await {
            let status = response.status();
            if !status.is_success() {
                let canonical_reason = status.canonical_reason().unwrap_or("Unknown");
                if canonical_reason != "Not Found" {
                    return Err(Error::Reqwest(format!(
                        "Failed to fetch robots.txt file: {canonical_reason}"
                    )));
                }
            }

            // If the robots.txt file is not found, then we assume that the domain is crawlable.
            let robots = match response.text().await {
                Ok(contents) => robots::parse(&contents),
                Err(err) => {
                    error!("Failed to parse robots.txt file: {err}");

                    RobotFile::default()
                }
            };

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
        let document = Html::parse_document(&body);

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
            vec![Page {
                url,
                forward_links: urls
                    .iter()
                    .filter_map(|url| Url::from_str(url).ok())
                    .collect::<Vec<_>>(),
                raw_html: document.html(),
            }],
            urls,
        ))
    }

    #[allow(clippy::expect_used)]
    async fn process(&self, item: Self::Item) -> Result<(), Error> {
        info!("Processing: {}", item.url);

        let title = Page::get_title(&item.raw_html);
        let description = Page::get_description(&item.raw_html);
        let language = Page::get_language(&item.raw_html);
        let keywords = Page::get_keywords(&item.raw_html);
        let words = Page::get_words(&item.raw_html, language.as_deref(), self.word_boundaries)?;

        debug!("Website Debug Info:");
        debug!("- URL: {}", item.url);
        debug!("- Title: {title:#?}");
        debug!("- Description: {description:#?}");
        debug!("- Language: {language:#?}");
        debug!("- Keywords: {keywords:#?}");
        debug!("- Words: {words:#?}");

        let mut conn = database::get_connection().await?;

        let page = database::create_page(
            &mut conn,
            &item.url,
            title.as_deref(),
            description.as_deref(),
        )
        .await?;

        let mut forward_links = HashMap::new();
        for url in item.forward_links {
            let count = forward_links.entry(url).or_insert(0);
            *count += 1;
        }
        database::create_links(&mut conn, &item.url, &forward_links).await?;

        let keywords = words
            .into_iter()
            .map(|(word, frequency)| NewKeyword {
                page_id: page.id,
                word,
                frequency: i32::try_from(frequency).expect("Failed to convert frequency!"),
            })
            .collect::<Vec<_>>();
        database::create_keywords(&mut conn, &keywords).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::expect_used)]
    fn test_get_words() {
        let html = r#"
            <html>
                <head>
                    <script src="script.js"></script>
                    <style src="style.css"></style>
                </head>
                <body>
                    <h1>Hello, world!</h1>
                    <p>This is a test.</p>
                    <p>This is yet another test.</p>
                    <p>This is a third test.</p>
                    <p>This is the final test.</p>
                </body>
            </html>
        "#;

        assert_eq!(
            Page::get_words(html, None, utils::env::spider::get_word_boundaries())
                .expect("Failed to get words!"),
            vec![
                ("hello".into(), 1), // "hello" is counted once.
                ("world".into(), 1), // "world" is counted once.
                ("this".into(), 4),  // "this" is counted four times.
                ("is".into(), 4),    // "is" is counted four times.
                ("test".into(), 4),  // "test" is counted four times.
                ("yet".into(), 1),   // "yet" is counted once.
                ("anoth".into(), 1), // "another" is stemmed to "anoth" and counted once.
                ("third".into(), 1), // "third" is counted once.
                ("the".into(), 1),   // "the" is counted once.
                ("final".into(), 1), // "final" is counted once.
            ]
            .into_iter()
            .collect::<HashMap<_, _>>()
        );
    }
}
