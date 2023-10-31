use url::Url;

/// A parsed `robots.txt` file.
///
/// # Fields
///
/// * `crawl_delay`: The delay specified by the `robots.txt` file.
/// * `disallow`: The disallowed URLs specified by the `robots.txt` file.
/// * `allow`: The allowed URLs specified by the `robots.txt` file.
/// * `content`: The raw contents of the `robots.txt` file.
#[derive(Debug, Clone)]
pub struct RobotFile {
    pub crawl_delay: Option<u64>,
    pub disallow: Vec<String>,
    pub allow: Vec<String>,
    pub content: String,
}

impl RobotFile {
    /// Checks if a URL is crawlable.
    ///
    /// # Arguments
    ///
    /// * `url`: The URL to check.
    ///
    /// # Returns
    ///
    /// * `bool`: Whether the URL is crawlable, or not.
    pub fn is_crawlable(&self, url: &Url) -> bool {
        let path = url.path().to_lowercase();

        if self.disallow.iter().any(|url| path.starts_with(url)) {
            return false;
        }

        if self.allow.iter().any(|url| path.starts_with(url)) {
            return true;
        }

        true
    }
}

/// Parses a `robots.txt` file.
///
/// # Arguments
///
/// * `content`: The content of the `robots.txt` file.
///
/// # Returns
///
/// The parsed `robots.txt` file.
#[allow(clippy::expect_used)]
pub fn parse(content: &str) -> RobotFile {
    let mut crawl_delay = None;

    let mut user_agent = String::new();
    let mut disallow = Vec::new();
    let mut allow = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let mut parts = line.splitn(2, ':');

        let key = parts.next().expect("Failed to get key!").to_lowercase();
        let value = parts.next().unwrap_or_default().trim();

        match key.as_str() {
            "user-agent" => {
                if user_agent.is_empty() {
                    user_agent = value.to_lowercase();
                }
            }
            "crawl-delay" => {
                if crawl_delay.is_none() {
                    crawl_delay = value.parse::<u64>().ok();
                }
            }
            "disallow" => {
                if user_agent == "*" {
                    disallow.push(value.to_lowercase());
                }
            }
            "allow" => {
                if user_agent == "*" {
                    allow.push(value.to_lowercase());
                }
            }
            _ => {}
        }
    }

    RobotFile {
        crawl_delay,
        disallow,
        allow,
        content: content.to_string(),
    }
}
