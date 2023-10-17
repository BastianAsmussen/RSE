use log::error;
use scraper::{Html, Selector};
use spider::page::Page;
use spider::website::Website;
use std::collections::HashMap;

/// Data scraped from a page.
///
/// # Fields
/// * `url` - The URL of the page.
/// * `title` - The title of the page.
/// * `description` - The description of the page.
/// * `keywords` - The keywords of the page.
/// * `outbound_links` - The outbound links of the page.
#[derive(Debug)]
pub struct Data {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub outbound_links: Option<Vec<String>>,
}

impl Data {
    /// Creates a new `Data` instance.
    ///
    /// # Arguments
    /// * `url` - The URL of the page.
    /// * `title` - The title of the page.
    /// * `description` - The description of the page.
    /// * `keywords` - The keywords of the page.
    /// * `outbound_links` - The outbound links of the page.
    pub fn new(
        url: &str,
        title: Option<&str>,
        description: Option<&str>,
        keywords: Option<&[String]>,
        outbound_links: Option<&[String]>,
    ) -> Self {
        Self {
            url: url.to_string(),
            title: title.map(std::string::ToString::to_string),
            description: description.map(std::string::ToString::to_string),
            keywords: keywords.map(<[String]>::to_vec),
            outbound_links: outbound_links.map(<[String]>::to_vec),
        }
    }
}

/// Scrapes data from a website.
///
/// # Arguments
/// * `website` - The website to scrape data from.
///
/// # Returns
/// A map of URLs to scraped data.
///
/// # Errors
/// * If the title selector fails to parse.
/// * If the title selector fails to select an element.
/// * If the title selector fails to get the inner HTML of the element.
/// * If the description selector fails to parse.
/// * If the description selector fails to select an element.
/// * If the description selector fails to get the content of the element.
/// * If the keywords selector fails to parse.
/// * If the keywords selector fails to select an element.
/// * If the keywords selector fails to get the content of the element.
/// * If the keywords fail to split.
/// * If the links selector fails to parse.
/// * If the links selector fails to select an element.
/// * If the links selector fails to get the href attribute of the element.
/// * If the links fail to split.
pub fn scrape(website: &Website) -> HashMap<String, Data> {
    let mut page_data = HashMap::new();

    let pages = website
        .get_pages()
        .iter()
        .flat_map(|pages| pages.iter().collect::<Vec<&Page>>())
        .collect::<Vec<&Page>>();

    for page in pages {
        let html = page.get_html();
        let document = Html::parse_document(&html);

        let url = page.get_url();
        let title = get_title(&document);
        let description = get_description(&document);
        let keywords = get_keywords(&document);
        let outbound_links = get_links(&document);

        let data = Data::new(
            url,
            title.as_deref(),
            description.as_deref(),
            keywords.as_deref(),
            outbound_links.as_deref(),
        );

        page_data.insert(url.to_string(), data);
    }

    page_data
}

/// Gets the title of a page.
///
/// # Arguments
/// * `document` - The HTML document to get the title from.
///
/// # Returns
/// The title of the page.
///
/// # Errors
/// * If the selector fails to parse the title selector.
/// * If the selector fails to select an element from the document.
/// * If the selector fails to get the inner HTML of the element.
fn get_title(document: &Html) -> Option<String> {
    let selector = match Selector::parse("title") {
        Ok(selector) => selector,
        Err(e) => {
            error!("Failed to parse title selector! (Error: {e})");

            return None;
        }
    };

    document
        .select(&selector)
        .next()
        .map(|element| element.inner_html())
}

/// Gets the description of a page.
///
/// # Arguments
/// * `document` - The HTML document to get the description from.
///
/// # Returns
/// The description of the page.
///
/// # Errors
/// * If the selector fails to parse the description selector.
/// * If the selector fails to select an element from the document.
/// * If the selector fails to get the content of the element.
fn get_description(document: &Html) -> Option<String> {
    let selector = match Selector::parse("meta[name=description]") {
        Ok(selector) => selector,
        Err(e) => {
            error!("Failed to parse description selector! (Error: {e})");

            return None;
        }
    };

    document
        .select(&selector)
        .next()
        .and_then(|element| element.value().attr("content"))
        .map(std::string::ToString::to_string)
}

/// Gets the keywords of a page.
///
/// # Arguments
/// * `document` - The HTML document to get the keywords from.
///
/// # Returns
/// The keywords of the page.
///
/// # Errors
/// * If the selector fails to parse the keywords selector.
/// * If the selector fails to select an element from the document.
/// * If the selector fails to get the content of the element.
/// * If the keywords fail to split.
fn get_keywords(document: &Html) -> Option<Vec<String>> {
    let selector = match Selector::parse("meta[name=keywords]") {
        Ok(selector) => selector,
        Err(e) => {
            error!("Failed to parse keywords selector! (Error: {e})");

            return None;
        }
    };

    document
        .select(&selector)
        .next()
        .and_then(|element| element.value().attr("content"))
        .map(|keywords| {
            keywords
                .split(',')
                .map(std::string::ToString::to_string)
                .collect()
        })
}

/// Gets the outbound links of a page.
///
/// # Arguments
/// * `document` - The HTML document to get the outbound links from.
///
/// # Returns
/// The outbound links of the page.
///
/// # Errors
/// * If the selector fails to parse the links selector.
/// * If the selector fails to select an element from the document.
/// * If the selector fails to get the href attribute of the element.
fn get_links(document: &Html) -> Option<Vec<String>> {
    let selector = match Selector::parse("a") {
        Ok(selector) => selector,
        Err(e) => {
            error!("Failed to parse links selector! (Error: {e})");

            return None;
        }
    };

    Some(
        document
            .select(&selector)
            .map(|element| element.value().attr("href"))
            .filter(Option::is_some)
            .map(|href| href.expect("Failed to get href attribute!").to_string())
            .collect::<Vec<String>>(),
    )
}
