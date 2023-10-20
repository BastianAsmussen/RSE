use crate::utils::db::model::{ForwardLink, Keyword, Page};
use log::info;
use rust_stemmers::{Algorithm, Stemmer};
use scraper::{error::SelectorErrorKind, Html, Selector};
use std::collections::HashMap;

/// A website is a page with keywords and forward links.
///
/// # Fields
///
/// * `page` - The page of the website.
/// * `keywords` - The keywords of the website.
/// * `forward_links` - The forward links of the website.
#[derive(Debug)]
pub struct Website {
    pub page: Page,
    pub keywords: Vec<Keyword>,
    pub forward_links: Option<Vec<ForwardLink>>,
}

/// Crawl a given URL and the URLs it links to, to a certain depth.
///
/// # Arguments
/// * `output`: A mutable reference to where to store the crawled URLs.
/// * `url`: The URL to crawl.
/// * `max_depth`: The maximum depth the crawler is allowed to go for a certain URL.
/// * `current_depth`: How deep the crawler currently is.
///
/// # Returns
///
/// * `Ok(())` if the crawling was successful, otherwise an `Err`.
pub fn crawl_url(
    output: &mut Vec<Website>,
    url: &str,
    max_depth: usize,
    current_depth: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    if current_depth >= max_depth {
        return Ok(());
    }

    info!("Crawling URL: {}", url);

    let body = reqwest::blocking::get(url)?.text()?;

    let document = Html::parse_document(&body);

    let title = get_title(&document)?;
    let description = get_description(&document)?;
    let keywords = get_keywords(&document)?;
    let forward_links = get_links(&document)?;

    let page = Page {
        url: url.to_string(),
        title,
        description,
    };

    let website = Website {
        page,
        keywords,
        forward_links: forward_links.clone(),
    };

    output.push(website);

    if let Some(links) = forward_links {
        for link in links {
            crawl_url(output, &link.url, max_depth, current_depth + 1)?;
        }
    }

    Ok(())
}

/// Crawl a list of URLs and the URLs they link to, to a certain depth.
///
/// # Arguments
///
/// * `output`: A mutable reference to where to store the crawled URLs.
/// * `urls`: The list of URLs to crawl.
/// * `max_depth`: How deep the crawler is allowed to go per URL.
///
/// # Returns
///
/// * `Ok(())` if the crawling was successful, otherwise an `Err`.
pub fn crawl_urls(
    output: &mut Vec<Website>,
    urls: Vec<&str>,
    max_depth: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    for url in urls {
        crawl_url(output, url, max_depth, 0)?;
    }

    Ok(())
}

/// Gets the title of a page.
///
/// # Arguments
///
/// * `document` - The HTML document to get the title from.
///
/// # Returns
///
/// * Either `Ok(Option)`, or an `Err`.
///
/// # Errors
///
/// * If the selector fails to parse the title selector.
/// * If the selector fails to select an element from the document.
/// * If the selector fails to get the inner HTML of the element.
pub fn get_title(document: &Html) -> Result<Option<String>, SelectorErrorKind<'static>> {
    let selector = Selector::parse("title")?;

    Ok(document
        .select(&selector)
        .next()
        .map(|element| element.inner_html()))
}

/// Gets the description of a page.
///
/// # Arguments
///
/// * `document` - The HTML document to get the description from.
///
/// # Returns
///
/// * Either `Ok(Option)`, or an `Err`.
///
/// # Errors
///
/// * If the selector fails to parse the description selector.
/// * If the selector fails to select an element from the document.
/// * If the selector fails to get the content of the element.
pub fn get_description(document: &Html) -> Result<Option<String>, SelectorErrorKind<'static>> {
    let selector = Selector::parse("meta[name=description]")?;

    Ok(document
        .select(&selector)
        .next()
        .and_then(|element| element.value().attr("content"))
        .map(std::string::ToString::to_string))
}

/// Gets the keywords of a page.
///
/// # Arguments
///
/// * `document` - The HTML document to get the keywords from.
///
/// # Returns
///
/// * Either `Ok(Option)`, or an `Err`.
///
/// # Errors
///
/// * If the selector fails to parse the keywords selector.
/// * If the selector fails to select an element from the document.
/// * If the selector fails to get the content of the element.
/// * If the keywords fail to split.
pub fn get_keywords(document: &Html) -> Result<Vec<Keyword>, Box<dyn std::error::Error>> {
    let selector = Selector::parse("meta[name=keywords]")?;

    let mut keywords = document
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
        remove_stop_words(keywords)?;
        // Stem the keywords.
        stem_keywords(keywords);
        // Convert the keywords to a list of keywords with their frequency.
        to_keywords(keywords)
    } else {
        Vec::new()
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
/// * Either `Ok(())`, or an `Err`.
///
/// # Errors
///
/// * If the stop words file fails to read.
/// * If the stop words file fails to parse.
///
/// # Notes
///
/// * The stop words file is located at `stop_words.txt`.
fn remove_stop_words(keywords: &mut Vec<String>) -> Result<(), std::io::Error> {
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
fn stem_keywords(keywords: &mut [String]) {
    let stemmer = Stemmer::create(Algorithm::English);

    keywords
        .iter_mut()
        .for_each(|keyword| *keyword = stemmer.stem(keyword).to_string());
}

/// Converts a list of strings to a list of keywords with their frequency.
///
/// # Arguments
///
/// * `keywords` - The keywords to convert.
///
/// # Returns
///
/// * A list of keywords with their frequency.
///
/// # Notes
///
/// * The frequency of a keyword is the amount of times it occurs in the list.
fn to_keywords(keywords: &[String]) -> Vec<Keyword> {
    let mut output = HashMap::new();

    for keyword in keywords {
        let frequency = output.entry(keyword).or_insert(0);

        *frequency += 1;
    }

    output
        .iter()
        .map(|(keyword, frequency)| Keyword {
            keyword: (*keyword).to_string(),
            frequency: *frequency,
        })
        .collect()
}

/// Gets the links of a page.
///
/// # Arguments
///
/// * `document` - The HTML document to get the outbound links from.
///
/// # Returns
///
/// * Either `Ok(Option)` or an `Err`.
///
/// # Errors
///
/// * If the selector fails to parse the links selector.
/// * If the selector fails to select an element from the document.
/// * If the selector fails to get the href attribute of the element.
pub fn get_links(document: &Html) -> Result<Option<Vec<ForwardLink>>, SelectorErrorKind<'static>> {
    let selector = Selector::parse("a")?;

    let links = Some(
        document
            .select(&selector)
            .map(|element| element.value().attr("href"))
            .filter(Option::is_some)
            .map(|href| href.expect("Failed to get href attribute!").to_string())
            .filter(|href| href.starts_with("http://") || href.starts_with("https://"))
            .collect::<Vec<String>>(),
    );

    Ok(to_forward_links(&links))
}

/// Converts a list of strings to a list of forward links.
///
/// # Arguments
///
/// * `links` - The links to convert.
///
/// # Returns
///
/// * Either `Some(Vec)`, or `None`.
fn to_forward_links(links: &Option<Vec<String>>) -> Option<Vec<ForwardLink>> {
    Some(
        links
            .as_ref()?
            .iter()
            .map(|link| ForwardLink {
                url: link.to_string(),
            })
            .collect(),
    )
}
