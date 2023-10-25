use crate::crawler::frontier::backend::Backend;
use crate::crawler::frontier::frontend::Frontier;
use crate::utils::env::crawler::get_max_crawl_depth;
use log::{info, warn};
use scraper::Html;

pub mod frontier;

/// A crawler that will crawl the web.
///
/// # Fields
///
/// * `frontier` - The frontier of the crawler.
#[derive(Debug)]
pub struct Crawler {
    frontier: Frontier,
}

impl Crawler {
    /// Create a new crawler with seed URLs.
    ///
    /// # Arguments
    ///
    /// * `seed_urls` - The seed URLs to be crawled.
    pub fn new(seed_urls: &[String]) -> Self {
        Self {
            frontier: Frontier::new(seed_urls),
        }
    }

    /// Get the frontier of the crawler.
    ///
    /// # Returns
    ///
    /// * `&Frontier` - The frontier of the crawler.
    pub fn get_frontier(&self) -> &Frontier {
        &self.frontier
    }

    /// Start the crawling process.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the crawling process was successful.
    /// * `Err(Box<dyn std::error::Error>)` - If the crawling process was unsuccessful.
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Start crawling.
        while let Some((url, depth)) = self.frontier.get_next_url() {
            let url = url.clone();

            match self.crawl((&url, depth)).await {
                Ok(()) => info!("Crawled \"{url}\"!"),
                Err(why) => warn!("Failed to crawl \"{url}\": {why}"),
            };
        }

        info!("There are no more URLs to be crawled!");

        Ok(())
    }

    pub async fn crawl(&mut self, url: (&str, usize)) -> Result<(), Box<dyn std::error::Error>> {
        let (url, depth) = url;
        let arrows = "=> ".repeat(depth);

        info!("{arrows}Crawling \"{url}\"...");

        // Check if the depth is greater than the maximum crawl depth.
        if depth > get_max_crawl_depth() {
            warn!("{arrows}Reached the maximum crawl depth!");

            self.frontier.inform_crawled(url);

            return Ok(());
        }

        // Check if the URL has already been crawled.
        if self.frontier.has_crawled(url) {
            warn!("{arrows}Already crawled \"{url}\"!");

            return Ok(());
        }

        // Check if the URL can be crawled.
        if !match self.frontier.can_crawl(url).await {
            Ok(can_crawl) => can_crawl,
            Err(why) => {
                warn!("{arrows}Failed to check if \"{url}\" can be crawled: {why}");

                return Ok(());
            }
        } {
            warn!("{arrows}\"{url}\" cannot be crawled!");

            return Ok(());
        }

        // Download the web page.
        let web_page = reqwest::get(&url.to_string()).await?.text().await?;

        let backend = Backend::new(Html::parse_document(&web_page));

        // Get the title of the web page.
        let title = match backend.get_title() {
            Ok(title) => title,
            Err(why) => {
                warn!("{arrows}Failed to get the title of \"{url}\": {why}");

                return Ok(());
            }
        };

        // Get the description of the web page.
        let description = match backend.get_description() {
            Ok(description) => description,
            Err(why) => {
                warn!("{arrows}Failed to get the description of \"{url}\": {why}");

                return Ok(());
            }
        };

        // Get the keywords of the web page.
        let keywords = match backend.get_keywords() {
            Ok(keywords) => keywords,
            Err(why) => {
                warn!("{arrows}Failed to get the keywords of \"{url}\": {why}");

                return Ok(());
            }
        };

        // Get the links of the web page.
        let links = match backend.get_links() {
            Ok(links) => {
                let Some(links) = links else {
                    warn!("{arrows}Failed to get the links of \"{url}\"!");

                    return Ok(());
                };

                links
            }
            Err(why) => {
                warn!("{arrows}Failed to get the links of \"{url}\": {why}");

                return Ok(());
            }
        };

        // Push the links to the frontier.
        for link in &links {
            match self.frontier.push((&link, depth + 1)) {
                Ok(()) => info!("{arrows}Pushed \"{link}\" to the frontier!"),
                Err(why) => warn!("{arrows}Failed to push \"{link}\" to the frontier: {why}"),
            };
        }

        // Add the URL to the crawled URLs.
        self.frontier.inform_crawled(url);

        // Push the web page to the database.
        let Ok(mut conn) = db::get_connection().await else {
            warn!("{arrows}Failed to get a connection to the database!");

            return Ok(());
        };

        let page = match db::create_page(
            &mut conn,
            url,
            title.as_deref(),
            description.as_deref(),
        )
            .await
        {
            Ok(page) => {
                info!("{arrows}Pushed \"{url}\" to the database!");

                page
            }
            Err(why) => {
                warn!("{arrows}Failed to push \"{url}\" to the database: {why}");

                return Ok(());
            }
        };

        // Push the keywords to the database.
        for keyword in &keywords {
            let keyword = (keyword.0.as_str(), keyword.1);
            match db::create_keyword(&mut conn, page.id, keyword).await {
                Ok(()) => info!("{arrows}Pushed \"{}\" to the database!", keyword.0),
                Err(why) => warn!(
                    "{arrows}Failed to push \"{}\" to the database: {why}",
                    keyword.0
                ),
            };
        }

        // Push the links to the database.
        for link in &links {
            match db::create_link(&mut conn, page.id, link).await {
                Ok(()) => info!("{arrows}Pushed \"{link}\" to the database!"),
                Err(why) => warn!("{arrows}Failed to push \"{link}\" to the database: {why}"),
            };
        }

        Ok(())
    }
}
