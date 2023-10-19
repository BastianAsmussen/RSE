use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Info {
    #[serde(rename = "q")]
    pub query: String,
}
