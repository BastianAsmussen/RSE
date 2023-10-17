use spider::website::Website;
use std::io::Error;

/// Crawl a website and return the crawled website.
///
/// # Arguments
/// * `url` - The URL of the website to crawl.
///
/// # Returns
/// * The crawled website.
///
/// # Notes
/// * This function is async.
/// * It will return an `Error` if the website fails to crawl.
pub async fn crawl(url: &str) -> Result<Website, Error> {
    let mut website = Website::new(url)
        .with_respect_robots_txt(true) // Respect robots.txt so we don't get banned.
        .with_budget(Some(spider::hashbrown::HashMap::from([
            ("*", 256),
            ("/licenses", 16),
            ("example.com", 0),
        ]))) // Set the budget for each domain.
        .with_user_agent(Some("RSE Crawler")) // Set the user agent.
        .with_http2_prior_knowledge(false) // Don't use HTTP/2 exclusively.
        .with_delay(0) // Don't delay requests.
        .with_request_timeout(None) // Max timeout for requests.
        .with_tld(true) // Detect TLDs.
        .with_subdomains(true) // Crawl subdomains.
        .build()?; // Build the website.

    website.scrape().await;

    Ok(website)
}
