use log::warn;
use std::env;

/// The default number of worker threads for crawling.
// TODO: Change this back to 1.
const DEFAULT_CRAWLER_WORKERS: usize = 8;

/// The default number of worker threads for processing.
// TODO: Change this back to 1.
const DEFAULT_PROCESSING_WORKERS: usize = 8;

/// Gets the number of workers for crawling.
///
/// # Returns
///
/// * `usize` - The number of workers.
///
/// # Panics
///
/// * If `CRAWLER_WORKERS` is not valid UTF-8.
/// * If `CRAWLER_WORKERS` is not a valid number.
#[allow(clippy::expect_used)]
pub fn get_crawler_workers() -> usize {
    env::var_os("CRAWLER_WORKERS").map_or(
        {
            warn!(
                "CRAWLER_WORKERS is not set! Using default value of {DEFAULT_CRAWLER_WORKERS}..."
            );

            DEFAULT_CRAWLER_WORKERS
        },
        |worker_threads| {
            worker_threads
                .to_str()
                .expect("CRAWLER_WORKERS must be valid UTF-8!")
                .parse::<usize>()
                .expect("CRAWLER_WORKERS must be a valid number!")
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
    env::var_os("PROCESSING_WORKERS").map_or(
        {
            warn!(
                "PROCESSING_WORKERS is not set! Using default value of {DEFAULT_CRAWLER_WORKERS}..."
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
