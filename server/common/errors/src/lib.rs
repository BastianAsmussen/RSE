use diesel::ConnectionError;
use serde::Serialize;
use std::io;
use std::num::TryFromIntError;
use thiserror::Error;

/// An errors.
///
/// # Variants
///
/// * `Internal`: An internal error.
/// * `IO`: An IO error.
/// * `Reqwest`: A reqwest error.
/// * `InvalidUrl`: An invalid URL.
/// * `InvalidBoundaries`: Invalid boundaries.
/// * `Database`: A database error.
/// * `NumberParseError`: A number parse error.
/// * `Query`: A query error.
#[derive(Error, Serialize, Debug, Clone)]
pub enum Error {
    #[error("Internal")]
    Internal(String),
    #[error("IO: {0}")]
    IO(String),
    #[error("Reqwest: {0}")]
    Reqwest(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Invalid Boundaries: {0}")]
    InvalidBoundaries(String),
    #[error("Database: {0}")]
    Database(String),
    #[error("Parse Error: {0}")]
    NumberParseError(String),
    #[error("Query: {0}")]
    Query(String),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IO(err.to_string())
    }
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

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        Self::Database(err.to_string())
    }
}

impl From<ConnectionError> for Error {
    fn from(err: ConnectionError) -> Self {
        Self::Database(err.to_string())
    }
}

impl From<TryFromIntError> for Error {
    fn from(err: TryFromIntError) -> Self {
        Self::NumberParseError(err.to_string())
    }
}
