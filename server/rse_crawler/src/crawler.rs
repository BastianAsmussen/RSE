use crate::indexer;
use crate::utils::db::model::{ForwardLink, Keyword, Page};
use log::info;
use scraper::{error::SelectorErrorKind, Html, Selector};

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
    output: &mut Vec<Page>,
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
    output.push(page);

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
    output: &mut Vec<Page>,
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
fn get_title(document: &Html) -> Result<Option<String>, SelectorErrorKind<'static>> {
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
fn get_description(document: &Html) -> Result<Option<String>, SelectorErrorKind<'static>> {
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
fn get_keywords(document: &Html) -> Result<Option<Vec<Keyword>>, SelectorErrorKind<'static>> {
    let selector = Selector::parse("meta[name=keywords]")?;

    let keywords = document
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

    /*
     TODO:
     - Remove stop words from the keywords.
     - Stem the keywords.
     - Count the frequency of the keywords.
     */

    Ok(_)
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
fn get_links(document: &Html) -> Result<Option<Vec<ForwardLink>>, SelectorErrorKind<'static>> {
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



    Ok()
}
