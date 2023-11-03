use crate::robots::RobotsFile;
use crate::scrapers::Scraper;
use async_trait::async_trait;
use common::database::model::NewKeyword;
use common::errors::Error;
use common::{database, utils};
use html5ever::tree_builder::TreeSink;
use log::{debug, error, info, warn};
use reqwest::Client;
use rust_stemmers::Algorithm;
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
/// * `word_boundaries` - The boundaries of the words.
#[derive(Debug)]
pub struct Web {
    http_client: Client,
    max_depth: Option<u32>,
    robots_cache: RwLock<HashMap<String, RobotsFile>>,
    word_boundaries: (usize, usize, usize, usize),
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
            word_boundaries: utils::env::scraper::get_word_boundaries(),
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
                html: body,
                links: Some(links.clone()),
            }],
            links
                .into_iter()
                .map(|url| (url, depth + 1))
                .collect::<HashMap<_, _>>(),
        ))
    }

    #[allow(clippy::expect_used)]
    async fn process(&self, item: Self::Item) -> Result<(), Error> {
        info!("Processing \"{}\"...", item.url);

        let title = Website::get_title(&item.html);
        let description = Website::get_description(&item.html);
        let language = Website::get_language(&item.html);
        let keywords = Website::get_keywords(&item.html);
        let words = Website::get_words(&item.html, language.as_deref(), self.word_boundaries)?;
        let link_count = item.links.as_ref().map(Vec::len).unwrap_or_default();

        debug!("=> Title: {title:?}");
        debug!("=> Description: {description:?}");
        debug!("=> Language: {language:?}");
        debug!("=> Keywords: {keywords:?}");
        debug!("=> Words: {}", words.len());
        debug!("=> Links: {link_count}");

        let mut conn = database::get_connection().await?;

        info!("=> Creating page with URL: {}", item.url);
        let page = database::create_page(
            &mut conn,
            &item.url,
            title.as_deref(),
            description.as_deref(),
        )
        .await?;

        let mut forward_links = HashMap::new();
        for link in item.links.unwrap_or_else(|| {
            warn!("=> No links found for \"{}\"!", item.url);

            Vec::new()
        }) {
            if link == item.url {
                warn!("=> Skipping forward link to self for \"{}\"...", link);

                continue;
            }

            let count = forward_links.entry(link).or_insert(0);
            *count += 1;
        }
        info!(
            "=> Creating {} forward links for \"{}\"...",
            forward_links.len(),
            item.url
        );
        database::create_forward_links(&mut conn, &item.url, &forward_links).await?;

        let keywords = words
            .into_iter()
            .map(|(word, frequency)| NewKeyword {
                page_id: page.id,
                word,
                frequency: i32::try_from(frequency).expect("=> Failed to convert frequency!"),
            })
            .collect::<Vec<_>>();
        info!(
            "=> Creating {} keywords for page with URL: {}",
            keywords.len(),
            item.url
        );
        database::create_keywords(&mut conn, &keywords).await?;

        Ok(())
    }
}

/// A scraped website.
///
/// # Fields
///
/// * `url` - The URL of the website.
/// * `html` - The HTML of the website.
/// * `links` - The links on the website, if any.
pub struct Website {
    pub url: Url,
    pub html: String,
    pub links: Option<Vec<Url>>,
}

impl Website {
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
            .select(&Selector::parse("title").expect("Failed to parse title selector!"))
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
                &Selector::parse("meta[name=description]")
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
            .select(&Selector::parse("html").expect("Failed to parse HTML selector!"))
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
                &Selector::parse("meta[name=keywords]")
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
        let selector = Selector::parse("script, style").expect("Failed to parse selector!");
        let node_ids = document
            .select(&selector)
            .map(|x| x.id())
            .collect::<Vec<_>>();
        for node_id in node_ids {
            document.remove_from_parent(&node_id);
        }

        // Get the text from the body.
        let selector = Selector::parse("body").expect("Failed to parse body selector!");
        let element = document
            .select(&selector)
            .next()
            .expect("Failed to get body!");
        let text = &element.text().collect::<Vec<_>>().join(" ");

        // Get the language of the page, or default to English.
        let language = language.unwrap_or("en");
        let language = match language {
            "ar" => Algorithm::Arabic,
            "da" => Algorithm::Danish,
            "nl" => Algorithm::Dutch,
            "fi" => Algorithm::Finnish,
            "fr" => Algorithm::French,
            "de" => Algorithm::German,
            "hu" => Algorithm::Hungarian,
            "it" => Algorithm::Italian,
            "no" => Algorithm::Norwegian,
            "pt" => Algorithm::Portuguese,
            "ro" => Algorithm::Romanian,
            "ru" => Algorithm::Russian,
            "es" => Algorithm::Spanish,
            "sv" => Algorithm::Swedish,
            "tr" => Algorithm::Turkish,
            _ => Algorithm::English,
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
            Website::get_words(html, None, utils::env::scraper::get_word_boundaries())
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
