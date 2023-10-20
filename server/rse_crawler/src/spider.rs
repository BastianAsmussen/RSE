use async_recursion::async_recursion;
use log::{info, warn};
use spider::website::Website;

#[async_recursion(?Send)]
pub async fn crawl_url(url: &str, depth: usize) -> Result<Vec<Website>, std::io::Error> {
    let arrows = "-> ".repeat(depth);
    if depth > crate::utils::env::crawler::get_max_crawl_depth() {
        warn!("{arrows}Budget exceeded for URL \"{}\"!", url);

        return Ok(Vec::new());
    }

    info!("{arrows}Crawling URL \"{url}\"...");

    let mut website = Website::new(url);
    website
        .with_respect_robots_txt(true)
        .with_subdomains(true)
        .with_tld(false)
        .with_delay(0)
        .with_request_timeout(None)
        .with_http2_prior_knowledge(false)
        .with_user_agent(Some("RSE/0.1.0"))
        .with_headers(None)
        .with_proxies(None);
    website.build()?;
    website.crawl().await;

    let mut new_websites = vec![website.clone()];

    let links = website.get_links();

    for link in links {
        /*
        let html = Html::parse_document(&link.get_html());

        let Ok(title) = crawler::get_title(&html) else {
            warn!("No title found on page \"{}\"!", link.get_url());

            continue;
        };

        let Ok(description) = crawler::get_description(&html) else {
            warn!("No description found on page \"{}\"!", link.get_url());

            continue;
        };

        let Ok(keywords) = crawler::get_keywords(&html) else {
            warn!("No keywords found on page \"{}\"!", link.get_url());

            continue;
        };

        let Ok(Some(links)) = crawler::get_links(&html) else {
            warn!("No links found on page \"{}\"!", link.get_url());

            continue;
        };
         */

        new_websites.extend(crawl_url(link.as_ref(), depth + 1).await?);
    }

    Ok(new_websites)
}
