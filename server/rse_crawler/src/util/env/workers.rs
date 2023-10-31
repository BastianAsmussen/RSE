use log::warn;
use std::env;

/// The default number of worker threads for crawling.
const DEFAULT_CRAWLING_WORKERS: usize = 1;

/// The default number of worker threads for processing.
const DEFAULT_PROCESSING_WORKERS: usize = 1;

/// Gets the number of workers for crawling.
///
/// # Returns
///
/// * `usize` - The number of workers.
///
/// # Panics
///
/// * If `CRAWLING_WORKERS` is not valid UTF-8.
/// * If `CRAWLING_WORKERS` is not a valid number.
#[allow(clippy::expect_used)]
pub fn get_crawling_workers() -> usize {
    env::var_os("CRAWLING_WORKERS").map_or_else(
        || {
            warn!(
                "CRAWLING_WORKERS is not set! Using default value of {DEFAULT_CRAWLING_WORKERS}..."
            );

            DEFAULT_CRAWLING_WORKERS
        },
        |worker_threads| {
            worker_threads
                .to_str()
                .expect("CRAWLING_WORKERS must be valid UTF-8!")
                .parse::<usize>()
                .expect("CRAWLING_WORKERS must be a valid number!")
        },
    )
}

/// Gets the number of workers for processing.
///
/// # Returns
///
/// * `usize` - The number of workers.
///
/// # Panics
///
/// * If `PROCESSING_WORKERS` is not valid UTF-8.
/// * If `PROCESSING_WORKERS` is not a valid number.
#[allow(clippy::expect_used)]
pub fn get_processing_workers() -> usize {
    env::var_os("PROCESSING_WORKERS").map_or_else(|| {
            warn!(
                "PROCESSING_WORKERS is not set! Using default value of {DEFAULT_PROCESSING_WORKERS}..."
            );

            DEFAULT_PROCESSING_WORKERS
        },
        |worker_threads| {
            worker_threads
                .to_str()
                .expect("PROCESSING_WORKERS must be valid UTF-8!")
                .parse::<usize>()
                .expect("PROCESSING_WORKERS must be a valid number!")
        },
    )
}
