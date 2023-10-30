use log::warn;
use std::time::Duration;

/// The default delay between each request.
const DEFAULT_DELAY: Duration = Duration::from_secs(1);

/// Get the delay between each request.
///
/// # Returns
///
/// * The delay between each request in seconds.
///
/// # Notes
///
/// * If the `DELAY` environment variable isn't set, the default value is used.
/// * The default value is `DEFAULT_DELAY`.
pub fn get_delay() -> Duration {
    std::env::var_os("DELAY").map_or_else(
        || DEFAULT_DELAY,
        |delay| {
            let Some(delay) = delay.to_str() else {
                warn!(
                    "Failed to parse DELAY to string slice, defaulting to {}s...",
                    DEFAULT_DELAY.as_secs()
                );

                return DEFAULT_DELAY;
            };

            match delay.parse::<u64>() {
                Ok(delay) => Duration::from_secs(delay),
                Err(why) => {
                    warn!(
                        "DELAY isn't a valid number, defaulting to {}s... (Error: {why})",
                        DEFAULT_DELAY.as_secs()
                    );

                    DEFAULT_DELAY
                }
            }
        },
    )
}
