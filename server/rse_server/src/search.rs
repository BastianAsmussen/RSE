use database::CompletePage;
use error::Error;
use log::warn;
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
    #[allow(clippy::expect_used, clippy::cast_precision_loss)]
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

        // Map the pages to their keywords.
        let mut unordered_pages = Vec::new();
        for page in pages {
            let page_id = page.id;
            let page = CompletePage {
                page,
                keywords: database::get_keywords_by_page_id(&mut conn, page_id).await?,
            };

            unordered_pages.push(page);
        }

        // Find the backlinks for each page.
        let mut backlinks = HashMap::new();
        for page in &unordered_pages {
            let page_backlinks = database::get_backlinks(&mut conn, page).await?;

            for backlink in page_backlinks {
                let count = backlinks.entry(backlink).or_insert(0);
                *count += 1;
            }
        }

        // Sum up the token counts for each page, and use that as the relevance score for the page.
        let mut relevance_scores = HashMap::new();
        for page in &unordered_pages {
            let mut score = 0;
            let Some(keywords) = &page.keywords else {
                warn!("No keywords for page: {}", page.page.url);

                continue;
            };

            // For each keyword, add the frequency of the keyword times the frequency of the word in the query.
            for keyword in keywords {
                if let Some(frequency) = query.get(&keyword.word) {
                    score += frequency * usize::try_from(keyword.frequency)?;
                }
            }

            // Add the score to the page.
            relevance_scores.insert(page, score);
        }

        let rating_factor = utils::env::ranker::get_rating_factor();
        let ranker_constant = utils::env::ranker::get_ranker_constant();

        // Calculate the actual page rank.
        let mut page_ranks = HashMap::new();
        for page in relevance_scores.keys().copied() {
            let mut rank = rating_factor;
            for backlink in &unordered_pages {
                if let Some(frequency) = backlinks.get(backlink) {
                    if backlink.page.id == page.page.id {
                        continue;
                    }

                    // Rank is the sum of the relevance scores of the backlinks divided by the number of backlinks.
                    rank += (relevance_scores
                        .get(backlink)
                        .expect("Failed to get backlink score!")
                        / frequency) as f64;
                }

                rank *= ranker_constant;

                // Add the rank to the page.
                page_ranks.insert(page, rank);
            }
        }

        // Order the pages by their rank.
        let pages = {
            let mut pages = Vec::new();
            for (page, rank) in page_ranks {
                pages.push((page, rank));
            }

            pages.sort_by(|(_, rank_a), (_, rank_b)| {
                rank_b
                    .partial_cmp(rank_a)
                    .expect("Failed to compare ranks!")
            });
            pages
                .into_iter()
                .map(|(page, _)| page.clone())
                .collect::<Vec<_>>()
        };

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
/// * `errors`: An errors, if any.
/// * `pages`: The pages that match the query, if any.
#[derive(Debug, Serialize)]
pub struct Output {
    pub query: Option<String>,
    pub error: Option<Error>,
    pub pages: Option<Vec<CompletePage>>,
}
