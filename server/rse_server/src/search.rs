use database::CompletePage;
use error::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    /// * `Result<Output, Box<dyn std::errors::Error>>` - The search results.
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
            }
            None => return Err(Error::Query("No query provided!".into())),
        };

        let Ok(mut conn) = database::get_connection().await else {
            return Err(Error::Database("Failed to get database connection!".into()));
        };

        let query = utils::words::extract(query, rust_stemmers::Algorithm::English);

        // Get pages like the query, if any.
        let Some(pages) = database::get_pages_with_words(
            &mut conn,
            query.keys().map(std::string::ToString::to_string).collect(),
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
                keywords: database::get_keywords_by_page_id(&mut conn, page_id).await?,
            };

            results.push(page);
        }

        // Calculate the backlinks for each page.
        let mut backlinks = HashMap::new();
        for result in &results {
            let page_backlinks = database::get_backlinks(&mut conn, result).await?;

            for backlink in page_backlinks {
                let count = backlinks.entry(backlink).or_insert(0);
                *count += 1;
            }
        }

        // Rank the results based on the number of backlinks and keywords.
        let mut page_scores = results
            .iter()
            .map(|page| {
                let backlink_count = backlinks.get(page).unwrap_or(&0);
                let keyword_count = page.keywords.clone().unwrap_or_default().len();

                (page, backlink_count + keyword_count)
            })
            .collect::<Vec<_>>();
        page_scores.sort_by(|a, b| b.1.cmp(&a.1));

        let pages = page_scores
            .iter()
            .map(|(page, _)| page)
            .collect::<Vec<_>>()
            .iter()
            .map(|page| CompletePage {
                page: page.page.clone(),
                keywords: None,
            })
            .collect::<Vec<_>>();

        Ok(Output {
            query: self.query.clone(),
            pages: Some(pages),
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
/// * `errors`: An errors, if any.
#[derive(Debug, Serialize)]
pub struct Output {
    pub query: Option<String>,
    pub pages: Option<Vec<CompletePage>>,
    pub error: Option<Error>,
}
