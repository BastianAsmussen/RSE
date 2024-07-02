use anyhow::Result;
use reqwest::Url;
use scraper::{Html, Selector};

#[derive(Debug)]
pub struct Crawler {
    url: Url,
    body: Option<String>,
    found_links: Option<Vec<Url>>,
}

impl Crawler {
    pub const fn new(url: Url) -> Self {
        Self {
            url,
            body: None,
            found_links: None,
        }
    }

    pub const fn body(&self) -> &Option<String> {
        &self.body
    }

    pub const fn found_links(&self) -> &Option<Vec<Url>> {
        &self.found_links
    }

    pub async fn crawl(&mut self) -> Result<()> {
        let body = reqwest::get(self.url.as_str()).await?.text().await?;
        let found_links = self.find_links(&body);

        self.body = Some(body);
        self.found_links = Some(found_links);

        Ok(())
    }

    fn find_links(&self, body: &str) -> Vec<Url> {
        let document = Html::parse_document(body);
        let anchors = Selector::parse("a").expect("Failed to create anchor tag selector!");

        let mut links = Vec::new();
        for element in document.select(&anchors) {
            // If the element has no href, skip it.
            let Some(path) = element.value().attr("href") else {
                continue;
            };

            let mut url = self.url.clone();
            url.set_path(path);

            // Make sure we only check HTTP(s) sites.
            match url.scheme() {
                "http" | "https" => links.push(url),
                _ => continue,
            }
        }

        links
    }
}
