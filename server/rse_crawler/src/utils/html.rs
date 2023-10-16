use regex::Regex;
use std::collections::HashMap;

use scraper::{Html, Selector};

use crate::utils::db_manager::is_page_up_to_date;
use crate::utils::time;
use log::{error, warn};
use mysql_async::prelude::Queryable;
use mysql_async::{Conn, Row};
use spider::page::Page;

#[derive(Debug, Clone)]
pub struct Metadata {
    pub title: String,
    pub description: String,
    pub keywords: HashMap<String, u32>,
}

impl Metadata {
    fn new(title: &str, description: &str, keywords: &HashMap<String, u32>) -> Self {
        Self {
            title: title.to_string(),
            description: description.to_string(),
            keywords: keywords.clone(),
        }
    }

    /// Generate HTML metadata from HTML.
    ///
    /// # Arguments
    ///
    /// * `document`: A reference to a `Html` struct.
    ///
    /// # Returns
    ///
    /// * An `Option` containing either the generated `Metadata` or `None`.
    pub fn get_metadata(document: &Html) -> Option<Self> {
        let Some(title) = get_title(document) else {
            error!("No suitable title was found!");

            return None;
        };
        let description = get_description(document).unwrap_or(String::new());
        let Some(keywords) = get_keywords(document) else {
            error!("Failed to find keywords!");

            return None;
        };

        Some(Self::new(&title, &description, &keywords))
    }
}

fn get_title(document: &Html) -> Option<String> {
    let Ok(selector) = Selector::parse("title") else {
        error!("Failed to create title selector!");

        return None;
    };

    for element in document.select(&selector) {
        let title = element.text().collect::<String>();

        // Check if the title is not empty.
        if !title.is_empty() {
            return Some(title);
        }
    }

    // If no non-empty title is found, return None.
    warn!("No valid title was found!");

    None
}

fn get_description(document: &Html) -> Option<String> {
    let Ok(selector) = Selector::parse("description") else {
        error!("Failed to create description selector!");

        return None;
    };

    for element in document.select(&selector) {
        let description = element.text().collect::<String>();

        // Check if the description is not empty.
        if !description.is_empty() {
            return Some(description);
        }
    }

    // If no non-empty description is found, return None.
    None
}

fn process_text(text: &str) -> Option<String> {
    let text = text.to_lowercase();

    // Remove punctuation and special characters
    let Ok(regex) = Regex::new(r"[:punct]") else {
        error!("Failed to create regex!");

        return None;
    };
    let text = regex.replace_all(&text, "");

    let stop_words = ["a", "an", "the", "and", "in", "of", "on", "to", "is", "it"];
    let text = text
        .split_whitespace()
        .filter(|word| !stop_words.contains(word))
        .collect::<Vec<&str>>()
        .join(" ");

    Some(text)
}

fn get_keywords(document: &Html) -> Option<HashMap<String, u32>> {
    let Ok(selector) = Selector::parse("p, h1, h2, h3") else {
        error!("Failed to create keyword selector!");

        return None;
    };

    let mut keywords: HashMap<String, u32> = HashMap::new();

    for element in document.select(&selector) {
        let Some(text) = process_text(&element.text().collect::<String>()) else {
            error!("Failed to process text, skipping...");

            continue;
        };

        let words: Vec<&str> = text.split_whitespace().collect();
        for word in words {
            *keywords.entry(word.to_string()).or_insert(0) += 1;
        }
    }

    Some(keywords)
}

/// Index a list of pages.
///
/// # Arguments
///
/// * `conn`: A mutable reference to a database connection.
/// * `pages`: A reference to a list of `Page`s.
///
/// # Returns
///
/// * A `Vec` containing the generated `Metadata`.
pub async fn index_pages(conn: &mut Conn, pages: &[Page]) -> Vec<Metadata> {
    let mut metadatas: Vec<Metadata> = Vec::new();

    for (_, page) in pages.iter().enumerate() {
        let html = page.get_html();
        if html.is_empty() {
            warn!("Empty HTML at {}, skipping...", page.get_url());

            continue;
        }

        let document = Html::parse_document(&html);

        let Some(metadata) = Metadata::get_metadata(&document) else {
            error!("Failed to fetch metadata!");

            continue;
        };

        metadatas.push(metadata.clone());

        // Check if the page is up to date.
        if is_page_up_to_date(conn, page).await {
            continue;
        }

        let url = page.get_url();
        if url.len() > 2_048 {
            warn!("URL is too long, skipping...");

            continue;
        }

        // If the database doesn't contain a copy of the page, insert it.
        let select_query = "SELECT id FROM web_pages WHERE url = ?";
        let result = conn
            .exec_map(select_query, (url,), |row: Row| {
                // Extract the 'id' column from the result row
                row.get_opt::<u32, &str>("id")
            })
            .await;

        if let Ok(id) = result {
            if id.is_empty() {
                let url = page.get_url();
                if url.len() > 2_048 {
                    warn!("URL is too long, skipping...");

                    continue;
                }

                let html = page.get_html().replace('\n', "");
                let html = if html.len() > 65_535 {
                    warn!("HTML is too long, cutting...");

                    html[..65_535].to_string()
                } else {
                    html
                };

                let insert_query = "INSERT INTO web_pages (url, title, description, content, timestamp) VALUES (?, ?, ?, ?, ?)";
                let result = conn
                    .exec_drop(
                        insert_query,
                        (url, &metadata.title, &metadata.description, html, time::get_ms_time()),
                    )
                    .await;

                if let Err(err) = result {
                    error!("Failed to insert the page: {}", err);

                    continue;
                }

                let url = page.get_url();
                if url.len() > 2_048 {
                    warn!("URL is too long, skipping...");

                    continue;
                }

                let select_query = "SELECT id FROM web_pages WHERE url = ?";
                let result = conn
                    .exec_map(select_query, (url,), |row: Row| {
                        // Extract the 'id' column from the result row
                        row.get_opt::<u32, &str>("id")
                    })
                    .await;

                if let Ok(id) = result {
                    if id.is_empty() {
                        error!("Failed to get the ID of the page!");

                        continue;
                    }

                    let Some(Some(Ok(id))) = id.first().cloned() else {
                        error!("Failed to get the ID of the page!");

                        continue;
                    };

                    let metadata = metadatas.pop().expect("Failed to pop metadata!");

                    let keyword_query = "INSERT INTO keywords (page_id, keyword, frequency) VALUES (?, ?, ?)";
                    for (keyword, frequency) in metadata.keywords {
                        let result = conn
                            .exec_drop(keyword_query, (id, keyword, frequency))
                            .await;

                        if let Err(err) = result {
                            error!("Failed to insert the keyword: {}", err);
                        }
                    }
                }
                
                continue;
            }

            let Some(Some(Ok(id))) = id.first().cloned() else {
                error!("Failed to get the ID of the page!");

                continue;
            };

            let metadata = metadatas.pop().expect("Failed to pop metadata!");

            let html = page.get_html().replace('\n', "");
            let html = if html.len() > 65_535 {
                warn!("HTML is too long, cutting...");

                html[..65_535].to_string()
            } else {
                html
            };

            let update_query = "UPDATE web_pages SET title = ?, description = ?, content = ?, timestamp = ? WHERE id = ?";
            let result = conn
                .exec_drop(update_query, (&metadata.title, &metadata.description, html, time::get_ms_time(), id))
                .await;

            if let Err(err) = result {
                error!("Failed to update the page: {}", err);
            }

            let keyword_query = "INSERT INTO keywords (page_id, keyword, frequency) VALUES (?, ?, ?)";
            for (keyword, frequency) in metadata.keywords {
                let result = conn
                    .exec_drop(keyword_query, (id, keyword, frequency))
                    .await;

                if let Err(err) = result {
                    error!("Failed to insert the keyword: {}", err);
                }
            }

            continue;
        }
    }

    metadatas
}
