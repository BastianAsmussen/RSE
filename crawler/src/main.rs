mod crawler;

use std::collections::HashSet;

use anyhow::Result;
use crawler::Crawler;
use reqwest::Url;

#[tokio::main]
async fn main() -> Result<()> {
    let (thread_sender, thread_receiver) = std::sync::mpsc::channel();
    tokio::spawn(async move {
        loop {
            let handle = thread_receiver
                .recv()
                .expect("Failed to read from channel!");
            if let Err(why) = handle.await {
                eprintln!("An error occurred: {why}");
            };
        }
    });

    let (url_sender, url_receiver) = std::sync::mpsc::channel();
    url_sender.send(Url::parse("https://en.wikipedia.org/wiki/Main_Page")?)?;

    let mut seen_urls = HashSet::new();
    loop {
        let next_url = url_receiver.recv()?;
        println!("Found '{}'.", next_url.as_str());

        if seen_urls.contains(&next_url) {
            continue;
        }

        let url_sender = url_sender.clone();
        seen_urls.insert(next_url.clone());

        let handle = tokio::spawn(async move {
            let mut crawler = Crawler::new(url_sender, next_url);

            crawler.crawl().await
        });

        thread_sender.send(handle)?;
    }
}
