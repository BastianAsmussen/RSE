use log::warn;

/// A frontier is a queue of URLs to be crawled.
///
/// # Fields
///
/// * `to_be_crawled` - The URLs to be crawled.
/// * `crawled` - The URLs that have been crawled.
#[derive(Debug)]
pub struct Frontier {
    to_be_crawled: Vec<(String, usize)>,
    crawled: Vec<String>,
}

impl Frontier {
    /// Create a new frontier with seed URLs.
    ///
    /// # Arguments
    ///
    /// * `seed_urls` - The seed URLs to be crawled.
    pub fn new(seed_urls: &[String]) -> Self {
        Self {
            to_be_crawled: seed_urls.iter().map(|url| (url.to_string(), 0)).collect(),
            crawled: Vec::new(),
        }
    }

    /// Get the URLs to be crawled.
    ///
    /// # Returns
    ///
    /// * `&Vec<(String, usize)>` - The URLs to be crawled and their depths.
    pub fn get_to_be_crawled(&self) -> &Vec<(String, usize)> {
        &self.to_be_crawled
    }

    /// Get the URLs that have been crawled.
    ///
    /// # Returns
    ///
    /// * `&Vec<String>` - The URLs that have been crawled.
    pub fn get_crawled(&self) -> &Vec<String> {
        &self.crawled
    }

    /// Get the next URL to be crawled.
    ///
    /// # Returns
    ///
    /// * `Some((String, usize))` - The next URL to be crawled and its depth.
    /// * `None` - If there is no URL to be crawled.
    pub fn get_next_url(&mut self) -> Option<(String, usize)> {
        self.to_be_crawled.pop()
    }

    /// Push a URL to the frontier.
    ///
    /// # Arguments
    ///
    /// * `(url, depth)` - The URL to be pushed and its depth.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the URL was successfully pushed.
    /// * `Err(Box<dyn std::error::Error>)` - If the URL was not pushed.
    ///
    /// # Errors
    ///
    /// * If the URL is already in the frontier.
    pub fn push(&mut self, url: (&str, usize)) -> Result<(), Box<dyn std::error::Error>> {
        let url = (url.0.to_string(), url.1);

        if self.to_be_crawled.contains(&url) {
            return Err(format!("The URL \"{}\" is already in the frontier!", url.0).into());
        }

        self.to_be_crawled.push(url);

        Ok(())
    }

    /// Inform the frontier that a URL has been crawled.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL that has been crawled.
    pub fn inform_crawled(&mut self, url: &str) {
        self.crawled.push(url.to_string());
    }

    /// Check if a URL has been crawled.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to check.
    ///
    /// # Returns
    ///
    /// * `bool` - If the URL has been crawled.
    pub fn has_crawled(&self, url: &str) -> bool {
        self.crawled.contains(&url.to_string())
    }

    /// Check if a URL can be crawled, i.e. if the `robots.txt` file allows it.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to check.
    ///
    /// # Returns
    ///
    /// * `bool` - If the URL can be crawled.
    ///
    /// # Notes
    ///
    /// * The `robots.txt` file is located at the root of the domain.
    pub async fn can_crawl(&self, url: &str) -> Result<bool, Box<dyn std::error::Error>> {
        // Robots.txt files are located at the root of the domain.
        // For example, the robots.txt file for https://www.google.com/ is located at https://www.google.com/robots.txt.
        let domain = url.split('/').take(3).collect::<Vec<&str>>().join("/");

        let robots_url = format!("{domain}/robots.txt");

        let Ok(response) = reqwest::get(&robots_url).await else {
            warn!("Failed to fetch the robots.txt file at \"{}\"!", robots_url);

            return Ok(true);
        };
        let Ok(robots) = response.text().await else {
            warn!("Failed to read the robots.txt file at \"{}\"!", robots_url);

            return Ok(true);
        };

        let url = url.replace(&domain, "");

        Ok(!robots
            .lines()
            .filter(|line| line.starts_with("Disallow: "))
            .map(|line| line.replace("Disallow: ", ""))
            .any(|disallowed| url.starts_with(&disallowed)))
    }
}