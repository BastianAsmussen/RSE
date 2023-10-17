use serde::Deserialize;

/// Representation of a query.
///
/// # Fields
/// * `text` - The text of the query.
#[derive(Debug, Deserialize)]
pub struct Query {
    #[serde(rename = "q")]
    pub text: String,
}

impl Query {
    /// Creates a new `Query` instance.
    ///
    /// # Arguments
    /// * `text` - The text of the query.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}
