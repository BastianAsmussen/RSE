use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Get the current time in milliseconds.
///
/// # Returns
///
/// * A `Duration` representing the current time in milliseconds.
pub fn get_ms_time() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards!")
}
