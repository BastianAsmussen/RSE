use std::env;

const DEFAULT_WORKER_THREADS: usize = 1;

/// Gets the number of worker threads.
///
/// # Returns
///
/// * `usize` - The number of worker threads.
///
/// # Panics
///
/// * If `WORKER_THREADS` is not valid UTF-8.
/// * If `WORKER_THREADS` is not a valid number.
pub fn get_worker_threads() -> usize {
    env::var_os("WORKER_THREADS")
        .map(|worker_threads| {
            worker_threads
                .to_str()
                .expect("WORKER_THREADS must be valid UTF-8!")
                .parse::<usize>()
                .expect("WORKER_THREADS must be a valid number!")
        })
        .unwrap_or(DEFAULT_WORKER_THREADS)
}
