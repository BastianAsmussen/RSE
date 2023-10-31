mod crawler;
mod robots;
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

    let spider = Arc::new(Web::new(
        &env::spider::get_http_timeout(),
        &env::spider::get_user_agent(),
        env::spider::get_word_boundaries(),
    ));
    crawler.run(spider).await;
}
