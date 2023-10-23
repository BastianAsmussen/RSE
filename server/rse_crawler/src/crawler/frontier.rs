/*
Crawler Frontier:
1. Maintain a list of URLs to be crawled.
2. Maintain a list of URLs that have been crawled.
3. Provide a method to get the next URL to be crawled.
4. Provide a method to add a new URL to be crawled.
*/

use db::model::{NewPage, Page};

#[derive(Debug)]
pub struct Frontier {
    to_be_crawled: Vec<NewPage>,
    crawled: Vec<Page>,
}

impl Frontier {
    /// Create a new frontier with seed URLs.
    ///
    /// # Arguments
    ///
    /// * `seed_pages` - The seed pages to be crawled.
    /// * `oldest_pages` - The number of oldest / most urgent pages from the database to be crawled.
    ///
    /// # Returns
    ///
    /// * A `Result` with the created frontier if successful.
    ///
    /// # Errors
    ///
    /// * If the oldest pages could not be retrieved.
    pub async fn new(
        seed_pages: &[NewPage],
        oldest_pages: i64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Fetch the oldest pages from the database.
        let oldest_pages = db::get_oldest_pages(oldest_pages).await?;

        // Merge the seed pages and the oldest pages.
        let oldest_pages = oldest_pages
            .into_iter()
            .map(|page| NewPage {
                url: page.url,
                title: page.title,
                description: page.description,
            })
            .collect::<Vec<NewPage>>();

        let pages = seed_pages
            .iter()
            .chain(oldest_pages.iter())
            .cloned()
            .collect::<Vec<NewPage>>();

        Ok(Self {
            to_be_crawled: pages,
            crawled: Vec::new(),
        })
    }

    /// Get the next page to be crawled.
    ///
    /// # Returns
    ///
    /// * `Some(NewPage)` - The next URL to be crawled.
    /// * `None` - If there is no URL to be crawled.
    pub fn get_next_page(&mut self) -> Option<NewPage> {
        self.to_be_crawled.pop()
    }

    /// Add a new URL to be crawled.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to be crawled.
    pub fn inform_crawled(&mut self, page: Page) {
        self.crawled.push(page);
    }
}
