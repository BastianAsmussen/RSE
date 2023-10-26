use serde::Deserialize;
use db::CompletePage;

#[derive(Debug, Deserialize)]
pub struct Info {
    #[serde(rename = "q")]
    pub query: String,
}

impl Info {
    pub async fn search(&self) -> Result<Vec<CompletePage>, Box<dyn std::error::Error>> {
        let Ok(mut conn) = db::get_connection().await else {
            return Err("Failed to get database connection!".into())
        };

        let pages = db::get_description_like(&mut conn, &self.query).await?;

        Ok(pages)
    }
}