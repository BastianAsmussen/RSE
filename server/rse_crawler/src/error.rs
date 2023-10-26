use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Internal")]
    Internal(String),
    #[error("Spider is not valid: {0}")]
    InvalidSpider(String),
    #[error("Reqwest: {0}")]
    Reqwest(String),
    #[error("Scraper: {0}")]
    Scraper(String),
    #[error("Regex: {0}")]
    Regex(String),
    #[error("Diesel: {0}")]
    Diesel(String),
    #[error("URL: {0}")]
    Url(String),
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Self::Internal(error)
    }
}

impl From<&str> for Error {
    fn from(error: &str) -> Self {
        Self::Internal(error.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::Reqwest(error.to_string())
    }
}

impl From<scraper::error::SelectorErrorKind<'_>> for Error {
    fn from(error: scraper::error::SelectorErrorKind) -> Self {
        Self::Scraper(error.to_string())
    }
}

impl From<regex::Error> for Error {
    fn from(error: regex::Error) -> Self {
        Self::Regex(error.to_string())
    }
}

impl From<diesel::result::Error> for Error {
    fn from(error: diesel::result::Error) -> Self {
        Self::Diesel(error.to_string())
    }
}

impl From<diesel::result::ConnectionError> for Error {
    fn from(error: diesel::result::ConnectionError) -> Self {
        Self::Diesel(error.to_string())
    }
}

impl From<url::ParseError> for Error {
    fn from(error: url::ParseError) -> Self {
        Self::Url(error.to_string())
    }
}
