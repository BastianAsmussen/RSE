use scraper::error::SelectorErrorKind;
use scraper::{Html, Selector};
use std::collections::HashMap;
use stemmer::Stemmer;

/// A backend for a web page.
#[derive(Debug)]
pub struct Backend(Html);

impl Backend {
    /// Create a new backend with HTML.
    ///
    /// # Arguments
    ///
    /// * `html` - The HTML of the page.
    pub fn new(html: Html) -> Self {
        Self(html)
    }

    /// Gets the title of a page.
    ///
    /// # Returns
    ///
    /// * `Ok(Option<String>)` - The title of the page.
    /// * `Err(SelectorErrorKind<'static>)` - If the selector failed.
    ///
    /// # Errors
    ///
    /// * If the selector fails to parse the title selector.
    /// * If the selector fails to select an element from the document.
    pub fn get_title(&self) -> Result<Option<String>, SelectorErrorKind<'static>> {
        let selector = Selector::parse("title")?;

        let title = self
            .0
            .select(&selector)
            .next()
            .map(|element| element.inner_html());

        Ok(title)
    }

    /// Gets the description of a page.
    ///
    /// # Returns
    ///
    /// * `Ok(Option<String>)` - The description of the page.
    /// * `Err(SelectorErrorKind<'static>)` - If the selector failed.
    ///
    /// # Errors
    ///
    /// * If the selector fails to parse the description selector.
    /// * If the selector fails to select an element from the document.
    /// * If the selector fails to get the content attribute of the element.
    pub fn get_description(&self) -> Result<Option<String>, SelectorErrorKind<'static>> {
        let selector = Selector::parse("meta[name=description]")?;

        Ok(self
            .0
            .select(&selector)
            .next()
            .and_then(|element| element.value().attr("content"))
            .map(std::string::ToString::to_string))
    }

    /// Gets the keywords of a page.
    ///
    /// # Returns
    ///
    /// * `Ok(HashMap<String, i32>)` - The keywords of the page.
    /// * `Err(SelectorErrorKind<'static>)` - If the selector failed.
    ///
    /// # Errors
    ///
    /// * If the selector fails to parse the body selector.
    /// * If the selector fails to select an element from the document.
    /// * If the selector fails to get the inner HTML of the element.
    /// * If the stop words file fails to read.
    ///
    /// # Notes
    ///
    /// * The stop words file is located at `stop_words.txt`.
    /// * The keywords are stemmed.
    /// * The keywords are counted.
    /// * The keywords are filtered.
    pub fn get_keywords(&self) -> Result<HashMap<String, i32>, Box<dyn std::error::Error>> {
        let selector = Selector::parse("meta[name=keywords]")?;

        let mut keywords = self
            .0
            .select(&selector)
            .next()
            .and_then(|element| element.value().attr("content"))
            .map(|keywords| {
                keywords
                    .split(',')
                    .map(str::trim)
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<String>>()
            });

        // Filter the keywords.
        Ok(if let Some(keywords) = &mut keywords {
            // Remove stop words from the keywords.
            self.remove_stop_words(keywords)?;
            // Stem the keywords.
            self.stem_keywords(keywords);

            // Count the keywords.
            let mut keywords_count = HashMap::new();
            for word in &*keywords {
                let count = keywords_count.entry(word.to_string()).or_insert(0);
                *count += 1;
            }

            keywords_count
        } else {
            HashMap::new()
        })
    }

    /// Removes stop words from a list of keywords.
    ///
    /// # Arguments
    ///
    /// * `keywords` - The keywords to remove the stop words from.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the stop words were successfully removed.
    /// * `Err(std::io::Error)` - If the stop words failed to be read.
    ///
    /// # Errors
    ///
    /// * If the stop words file fails to read.
    /// * If the stop words file fails to parse.
    ///
    /// # Notes
    ///
    /// * The stop words file is located at `stop_words.txt`.
    fn remove_stop_words(&self, keywords: &mut Vec<String>) -> Result<(), std::io::Error> {
        let stop_words = std::fs::read_to_string("stop_words.txt")?;
        stop_words
            .lines()
            .for_each(|stop_word| keywords.retain(|keyword| keyword != stop_word));

        Ok(())
    }

    /// Stems a list of keywords.
    ///
    /// # Arguments
    ///
    /// * `keywords` - The keywords to stem.Vec
    fn stem_keywords(&self, keywords: &mut [String]) {
        let mut stemmer = Stemmer::new("english").expect("Failed to create stemmer!");

        keywords
            .iter_mut()
            .for_each(|keyword| *keyword = stemmer.stem(keyword));
    }

    /// Gets the links of a page.
    ///
    /// # Returns
    ///
    /// * `Ok(Option<Vec<String>>)` - The outbound links of the page.
    /// * `Err(SelectorErrorKind<'static>)` - If the selector failed.
    ///
    /// # Errors
    ///
    /// * If the selector fails to parse the links selector.
    /// * If the selector fails to select an element from the document.
    /// * If the selector fails to get the href attribute of the element.
    pub fn get_links(&self) -> Result<Option<Vec<String>>, SelectorErrorKind<'static>> {
        let selector = Selector::parse("a")?;

        let links = Some(
            self.0
                .select(&selector)
                .map(|element| element.value().attr("href"))
                .filter(Option::is_some)
                .map(|href| href.expect("Failed to get href attribute!").to_string())
                .filter(|href| href.starts_with("http://") || href.starts_with("https://"))
                .collect::<Vec<String>>(),
        );

        Ok(links)
    }
}
