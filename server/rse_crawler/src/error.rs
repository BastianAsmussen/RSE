use thiserror::Error;

/// An error.
///
/// # Variants
///
/// * `Internal`: An internal error.
/// * `Reqwest`: A reqwest error.
/// * `InvalidUrl`: An invalid URL.
#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Internal")]
    Internal(String),
    #[error("Reqwest: {0}")]
    Reqwest(String),
    #[error("URL is not valid: {0}")]
    InvalidUrl(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Reqwest(err.to_string())
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Self::InvalidUrl(err.to_string())
    }
}
