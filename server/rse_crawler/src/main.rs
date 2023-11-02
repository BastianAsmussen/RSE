use crate::crawler::Crawler;
use crate::scrapers::web::Web;
use log::info;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_ENCODING, CONNECTION, USER_AGENT};
use std::sync::Arc;

mod crawler;
mod robots;
mod scrapers;

#[tokio::main]
#[allow(clippy::expect_used)]
async fn main() {
    env_logger::init();

    let crawler = Crawler::new(
        utils::env::crawler::get_delay(),
        utils::env::workers::get_crawlers(),
        utils::env::workers::get_processors(),
    );

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, utils::env::spider::get_user_agent());
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
    );
    headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate"));
    headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));

    let http_client = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(utils::env::spider::get_http_timeout())
        .build()
        .expect("Failed to build HTTP client!");
    let scraper = Arc::new(Web::new(http_client, utils::env::spider::get_max_depth()));

    info!("Starting crawler...");
    crawler.run(scraper).await;
}
