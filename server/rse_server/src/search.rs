use db::CompletePage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use error::Error;

/// A query.
///
/// # Fields
///
/// * `query`: The query string.
#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    #[serde(rename = "q")]
    pub query: Option<String>,
}

impl Info {
    /// Searches for pages.
    ///
    /// # Returns
    ///
    /// * `Result<Output, Box<dyn std::error::Error>>` - The search results.
    ///
    /// # Errors
    ///
    /// * If the database connection fails.
    /// * If no pages are found.
    pub async fn search(&self) -> Result<Output, Error> {
        // Get the query.
        let query = match &self.query {
            Some(query) => {
                if query.is_empty() {
                    return Err(Error::Query("No query provided!".into()));
                }

                query
            },
            None => return Err(Error::Query("No query provided!".into())),
        };

        let Ok(mut conn) = db::get_connection().await else {
            return Err(Error::Database("Failed to get database connection!".into()));
        };

        // Get pages like the query, if any.
        let Some(pages) = db::get_pages_with_words(
            &mut conn,
            query
                .to_lowercase()
                .split_whitespace()
                .collect::<Vec<_>>(),
        )
        .await?
        else {
            return Err(Error::Query("No pages found!".into()));
        };

        let mut results = Vec::new();
        for page in pages {
            let page_id = page.id;
            let page = CompletePage {
                page,
                keywords: db::get_keywords_by_page_id(&mut conn, page_id).await?,
            };

            results.push(page);
        }

        // Rank the results.
        let mut backlinks = HashMap::new();
        for result in &results {
            let page_backlinks = db::get_backlinks(&mut conn, result).await?;

            for backlink in page_backlinks {
                let count = backlinks.entry(backlink).or_insert(0);
                *count += 1;
            }
        }

        results.sort_by(|a, b| {
            let a_backlinks = backlinks.get(a).unwrap_or(&0);
            let b_backlinks = backlinks.get(b).unwrap_or(&0);

            b_backlinks.cmp(a_backlinks)
        });

        Ok(Output {
            query: self.query.clone(),
            pages: Some(results),
            error: None,
        })
    }
}

/// The results of a search.
///
/// # Fields
///
/// * `query`: The query, if any.
/// * `pages`: The pages that match the query, if any.
/// * `error`: An error, if any.
#[derive(Debug, Serialize)]
pub struct Output {
    pub query: Option<String>,
    pub pages: Option<Vec<CompletePage>>,
    pub error: Option<Error>,
}
