use log::warn;

pub mod seed_urls;
pub mod timer;

/// Get the number of worker threads to use.
///
/// # Returns
/// * The number of worker threads to use.
///
/// # Notes
/// * If the `WORKER_THREADS` environment variable isn't set, the default value is used.
/// * The default value is the number of logical cores on the system.
pub fn get_worker_threads() -> usize {
    std::env::var_os("WORKER_THREADS")
        .map_or_else(num_cpus::get, |thread_count| {
            let default_value = num_cpus::get();

            let Some(thread_count) = thread_count.to_str() else {
                warn!("Failed to parse WORKER_THREADS to string slice, defaulting to {default_value}...");

                return default_value;
            };

            match thread_count.parse::<usize>() {
                Ok(thread_count) => thread_count,
                Err(e) => {
                    warn!("WORKER_THREADS isn't a valid number, defaulting to {default_value}... (Error: {e})");

                    default_value
                }
            }
        })
}
