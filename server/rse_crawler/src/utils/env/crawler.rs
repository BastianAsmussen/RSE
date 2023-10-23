use log::warn;

/// The default maximum depth to crawl to.
const DEFAULT_MAX_CRAWL_DEPTH: usize = 1;

/// Get the maximum depth to crawl to.
///
/// # Returns
///
/// * The maximum depth to crawl to.
///
/// # Notes
///
/// * If the `MAX_CRAWL_DEPTH` environment variable isn't set, the default value is used.
/// * The default value is `default_max_crawl_depth`.
pub fn get_max_crawl_depth() -> usize {
    std::env::var_os("MAX_CRAWL_DEPTH").map_or_else(
        || DEFAULT_MAX_CRAWL_DEPTH,
        |max_crawl_depth| {
            let Some(max_crawl_depth) = max_crawl_depth.to_str() else {
                warn!("Failed to parse MAX_CRAWL_DEPTH to string slice, defaulting to {DEFAULT_MAX_CRAWL_DEPTH}...");

                return 2;
            };

            match max_crawl_depth.parse::<usize>() {
                Ok(max_crawl_depth) => max_crawl_depth,
                Err(why) => {
                    warn!("MAX_CRAWL_DEPTH isn't a valid number, defaulting to {DEFAULT_MAX_CRAWL_DEPTH}... (Error: {why})");

                    DEFAULT_MAX_CRAWL_DEPTH
                }
            }
        },
    )
}
