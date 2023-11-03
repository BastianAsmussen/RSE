use std::time::Instant;

/// A timer that can be used to time multiple functions.
/// Timings are stored in nanoseconds.
#[derive(Debug)]
pub struct Timer {
    times: Vec<u128>,
}

impl Timer {
    /// Creates a new timer with no times recorded.
    #[must_use]
    pub const fn new() -> Self {
        Self { times: Vec::new() }
    }

    /// Time a function and return the elapsed time in nanoseconds and the result of the function call as a tuple.
    ///
    /// # Arguments
    /// * `function` - The function to time.
    ///
    /// # Examples
    /// ```
    /// use crate::common::utils::timer::Timer;
    ///
    /// let mut timer = Timer::new();
    ///
    /// let (time, result) = timer.time(|| {
    ///     let mut sum = 0;
    ///     while sum != 1_000_000 {
    ///         sum += 1;
    ///     }
    ///
    ///     sum
    /// });
    ///
    /// println!("Function took {} nanoseconds.", time);
    /// println!("Result of function call: {}", result);
    /// ```
    pub fn time<F, R>(&mut self, function: F) -> (u128, R)
    where
        F: FnOnce() -> R + Send,
    {
        let start = Instant::now();
        let result = function();
        let elapsed = start.elapsed().as_nanos();

        self.times.push(elapsed);

        (elapsed, result)
    }

    /// Get the total time elapsed by all timers.
    ///
    /// # Examples
    /// ```
    /// use crate::common::utils::timer::Timer;
    ///
    /// let mut timer = Timer::new();
    ///
    /// let (time, result) = timer.time(|| {
    ///     let mut sum = 0;
    ///     while sum != 1_000_000 {
    ///         sum += 1;
    ///     }
    ///
    ///     sum
    /// });
    /// println!("Function took {} nanoseconds.", time);
    ///
    /// let (time, result) = timer.time(|| {
    ///     let mut sum = 0;
    ///     while sum != 5_000_000 {
    ///         sum += 1;
    ///     }
    ///
    ///     sum
    /// });
    ///
    /// println!("Function took {} nanoseconds.", time);
    ///
    /// // Prints the time it took for both functions to run in nanoseconds.
    /// println!("Total Time: {} nanoseconds.", timer.total_time());
    #[must_use]
    pub fn total_time(&self) -> u128 {
        self.times.iter().sum()
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a time in nanoseconds into a human-readable string.
///
/// # Arguments
/// * `nanos` - The time in nanoseconds to format.
///
/// # Examples
/// ```
/// use crate::common::utils::timer::{format_time, Timer};
///
/// let mut timer = Timer::new();
///
/// let (time, result) = timer.time(|| {
///    let mut sum = 0;
///    while sum != 1_000_000 {
///        sum += 1;
///    }
///
///    sum
/// });
///
/// println!("Function took {}.", format_time(&time));
/// ```
#[must_use]
pub fn format_time(nanos: &u128) -> String {
    let units = [
        ("year", 31_557_600_000_000_000),
        ("month", 2_629_800_000_000_000),
        ("day", 86_400_000_000_000),
        ("hour", 3_600_000_000_000),
        ("minute", 60_000_000_000),
        ("second", 1_000_000_000),
        ("millisecond", 1_000_000),
        ("microsecond", 1_000),
        ("nanosecond", 1),
    ];

    let mut nanos_remaining = *nanos;
    let mut result = String::new();
    let mut units_displayed = 0;

    for (unit, nanos_per_unit) in &units {
        let count = nanos_remaining / nanos_per_unit;
        nanos_remaining %= nanos_per_unit;

        if count > 0 {
            if units_displayed > 0 {
                result.push_str(", ");

                // Add an "and" after the second to last unit.
                // E.g. 1 second, 200 milliseconds, 501 microseconds, and 1 nanosecond.
                if units_displayed == units.len() - 2 {
                    result.push_str("and ");
                }
            }

            result.push_str(&format!("{count} {unit}"));

            if count > 1 {
                result.push('s');
            }

            units_displayed += 1;
        }
    }

    result
}
