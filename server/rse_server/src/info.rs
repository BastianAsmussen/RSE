use std::collections::HashMap;
use db::CompletePage;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Info {
    #[serde(rename = "q")]
    pub query: String,
}

impl Info {
    pub async fn search(&self) -> Result<Vec<CompletePage>, Box<dyn std::error::Error>> {
        let Ok(mut conn) = db::get_connection().await else {
            return Err("Failed to get database connection!".into());
        };

        // Get keywords like the query, and get the pages that have those keywords.
        let Some(keywords) = db::get_keywords_like(&mut conn, &self.query.to_lowercase()).await? else {
            return Err("No keywords in any pages found!".into());
        };

        let mut results = Vec::new();
        for keyword in keywords {
            let Some(pages) = db::get_page_by_id(&mut conn, keyword.id).await? else {
                return Err("No pages found!".into());
            };

            for page in pages {
                let Some(metadata) = db::get_metadata_by_page_id(&mut conn, page.id).await? else {
                    return Err("No metadata found!".into());
                };

                let keywords = db::get_keywords_by_page_id(&mut conn, page.id).await?;

                let mut title = None;
                let mut description = None;

                for metadatum in metadata {
                    match metadatum.name.as_str() {
                        "title" => title = Some(metadatum.content),
                        "description" => description = Some(metadatum.content),
                        _ => (),
                    }
                }

                let complete_page = CompletePage {
                    page,
                    title,
                    description,
                    keywords,
                };

                results.push(complete_page);
            }
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

        Ok(results)
    }
}
