mod crawler;
mod robots;
mod spiders;

use crate::spiders::web::Web;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    env_logger::init();

    let crawler = crawler::Crawler::new(
        utils::env::crawler::get_delay(),
        utils::env::workers::get_crawlers(),
        utils::env::workers::get_processors(),
    );

    let spider = Arc::new(Web::new(
        &utils::env::spider::get_http_timeout(),
        &utils::env::spider::get_user_agent(),
        utils::env::spider::get_word_boundaries(),
    ));
    crawler.run(spider).await;
}
