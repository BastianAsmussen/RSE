use const_format::formatcp;
use log::warn;
use reqwest::header::HeaderValue;
use std::env;
use std::time::Duration;

/// The default HTTP timeout.
const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(10);

/// The default user agent.
const DEFAULT_USER_AGENT: &str = formatcp!("RSE/{}", env!("CARGO_PKG_VERSION"));

/// The default minimum word frequency.
const DEFAULT_MINIMUM_WORD_FREQUENCY: usize = 1;

/// The default maximum word frequency.
const DEFAULT_MAXIMUM_WORD_FREQUENCY: usize = 100;

/// The default minimum word length.
const DEFAULT_MINIMUM_WORD_LENGTH: usize = 3;

/// The default maximum word length.
const DEFAULT_MAXIMUM_WORD_LENGTH: usize = 20;

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
#[must_use]
pub fn get_http_timeout() -> Duration {
    env::var_os("HTTP_TIMEOUT").map_or_else(
        || {
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

/// Gets the user agent.
///
/// # Returns
///
/// * `HeaderValue` - The user agent.
///
/// # Panics
///
/// * If `USER_AGENT` is not valid UTF-8.
/// * If `USER_AGENT` is not a valid header value.
#[allow(clippy::expect_used)]
#[must_use]
pub fn get_user_agent() -> HeaderValue {
    HeaderValue::from_str(&env::var_os("USER_AGENT").map_or_else(
        || {
            warn!("USER_AGENT is not set! Using default value of {DEFAULT_USER_AGENT}...",);

            DEFAULT_USER_AGENT.to_string()
        },
        |user_agent| {
            user_agent
                .to_str()
                .expect("USER_AGENT must be valid UTF-8!")
                .to_string()
        },
    ))
    .expect("USER_AGENT must be a valid header value!")
}

/// Gets the boundaries.
///
/// # Returns
///
/// * `(usize, usize, usize, usize)` - The boundaries, in order: minimum word frequency, maximum word frequency, minimum word length, maximum word length.
#[must_use]
pub fn get_word_boundaries() -> (usize, usize, usize, usize) {
    (
        get_minimum_word_frequency(),
        get_maximum_word_frequency(),
        get_minimum_word_length(),
        get_maximum_word_length(),
    )
}

/// Gets the minimum word frequency.
///
/// # Returns
///
/// * `usize` - The minimum word frequency.
#[allow(clippy::expect_used)]
fn get_minimum_word_frequency() -> usize {
    env::var_os("MINIMUM_WORD_FREQUENCY").map_or_else(
        || {
            warn!(
                "MINIMUM_WORD_FREQUENCY is not set! Using default value of {DEFAULT_MINIMUM_WORD_FREQUENCY}...",
            );

            DEFAULT_MINIMUM_WORD_FREQUENCY
        },
        |minimum_word_frequency| {
            minimum_word_frequency
                .to_str()
                .expect("MINIMUM_WORD_FREQUENCY must be valid UTF-8!")
                .parse::<usize>()
                .expect("MINIMUM_WORD_FREQUENCY must be a valid number!")
        },
    )
}

/// Gets the maximum word frequency.
///
/// # Returns
///
/// * `usize` - The maximum word frequency.
#[allow(clippy::expect_used)]
fn get_maximum_word_frequency() -> usize {
    env::var_os("MAXIMUM_WORD_FREQUENCY").map_or_else(
        || {
            warn!(
                "MAXIMUM_WORD_FREQUENCY is not set! Using default value of {DEFAULT_MAXIMUM_WORD_FREQUENCY}...",
            );

            DEFAULT_MAXIMUM_WORD_FREQUENCY
        },
        |maximum_word_frequency| {
            maximum_word_frequency
                .to_str()
                .expect("MAXIMUM_WORD_FREQUENCY must be valid UTF-8!")
                .parse::<usize>()
                .expect("MAXIMUM_WORD_FREQUENCY must be a valid number!")
        },
    )
}

/// Gets the minimum word length.
///
/// # Returns
///
/// * `usize` - The minimum word length.
#[allow(clippy::expect_used)]
fn get_minimum_word_length() -> usize {
    env::var_os("MINIMUM_WORD_LENGTH").map_or_else(
        || {
            warn!(
                "MINIMUM_WORD_LENGTH is not set! Using default value of {DEFAULT_MINIMUM_WORD_LENGTH}...",
            );

            DEFAULT_MINIMUM_WORD_LENGTH
        },
        |minimum_word_length| {
            minimum_word_length
                .to_str()
                .expect("MINIMUM_WORD_LENGTH must be valid UTF-8!")
                .parse::<usize>()
                .expect("MINIMUM_WORD_LENGTH must be a valid number!")
        },
    )
}

/// Gets the maximum word length.
///
/// # Returns
///
/// * `usize` - The maximum word length.
#[allow(clippy::expect_used)]
fn get_maximum_word_length() -> usize {
    env::var_os("MAXIMUM_WORD_LENGTH").map_or_else(
        || {
            warn!(
                "MAXIMUM_WORD_LENGTH is not set! Using default value of {DEFAULT_MAXIMUM_WORD_LENGTH}...",
            );

            DEFAULT_MAXIMUM_WORD_LENGTH
        },
        |maximum_word_length| {
            maximum_word_length
                .to_str()
                .expect("MAXIMUM_WORD_LENGTH must be valid UTF-8!")
                .parse::<usize>()
                .expect("MAXIMUM_WORD_LENGTH must be a valid number!")
        },
    )
}
