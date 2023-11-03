use crate::crawler::Crawler;
use crate::scrapers::web::Web;
use common::utils;
use log::info;
use reqwest::header::{HeaderMap, HeaderValue, CONNECTION, USER_AGENT};
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
    headers.insert(USER_AGENT, utils::env::scraper::get_user_agent());
    headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));

    let http_client = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(utils::env::scraper::get_http_timeout())
        .build()
        .expect("Failed to build HTTP client!");
    let scraper = Arc::new(Web::new(http_client, utils::env::scraper::get_max_depth()));

    info!("Starting crawler...");
    crawler.run(scraper).await;
}
