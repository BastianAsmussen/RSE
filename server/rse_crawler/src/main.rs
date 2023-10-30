mod crawler;
mod error;
mod spiders;
mod util;

use crate::spiders::web::Web;
use crate::util::env;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    env_logger::init();

    let crawler = crawler::Crawler::new(
        env::crawler::get_delay(),
        env::workers::get_crawling_workers(),
        env::workers::get_processing_workers(),
    );

    let spider = Arc::new(Web::new());
    crawler.run(spider).await;
}
