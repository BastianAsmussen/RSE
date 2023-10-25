use log::warn;
use regex::Regex;
use std::env;
use std::str::FromStr;
use std::time::Duration;

/// The regular expression used to extract URLs.
const DEFAULT_URL_REGEX: &str = r#"href="([^"]*)""#;

/// The default HTTP timeout.
const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(10);

/// Gets URL regex pattern.
///
/// # Returns
///
/// * `Regex` - The URL regex pattern.
///
/// # Panics
///
/// * If `URL_REGEX` is not a valid regex pattern.
/// * If `URL_REGEX` is not valid UTF-8.
/// * If `URL_REGEX` is not a valid regex pattern.
/// * If `DEFAULT_URL_REGEX` is not a valid regex pattern.
#[allow(clippy::expect_used)]
pub fn get_url_regex() -> Regex {
    env::var_os("URL_REGEX").map_or(
        {
            warn!("URL_REGEX is not set! Using default value of {DEFAULT_URL_REGEX}...");

            Regex::from_str(DEFAULT_URL_REGEX)
                .expect("DEFAULT_URL_REGEX must be a valid regex pattern!")
        },
        |url_regex| {
            Regex::from_str(url_regex.to_str().expect("URL_REGEX must be valid UTF-8!"))
                .expect("URL_REGEX must be a valid regex pattern!")
        },
    )
}

/// Gets the HTTP timeout.
///
/// # Returns
///
/// * `Duration` - The HTTP timeout in seconds.
///
/// # Panics
///
/// * If `HTTP_TIMEOUT` is not valid UTF-8.
/// * If `HTTP_TIMEOUT` is not a valid number.
#[allow(clippy::expect_used)]
pub fn get_http_timeout() -> Duration {
    env::var_os("HTTP_TIMEOUT").map_or(
        {
            warn!(
                "HTTP_TIMEOUT is not set! Using default value of {}...",
                DEFAULT_HTTP_TIMEOUT.as_secs()
            );

            DEFAULT_HTTP_TIMEOUT
        },
        |http_timeout| {
            Duration::from_secs(
                http_timeout
                    .to_str()
                    .expect("HTTP_TIMEOUT must be valid UTF-8!")
                    .parse::<u64>()
                    .expect("HTTP_TIMEOUT must be a valid number!"),
            )
        },
    )
}
