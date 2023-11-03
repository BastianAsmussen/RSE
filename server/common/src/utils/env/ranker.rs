use log::warn;

/// The default ranker constant.
const DEFAULT_RANKER_CONSTANT: f64 = 0.7;

/// The default rating factor.
const DEFAULT_RATING_FACTOR: f64 = 0.4;

/// Get the ranker constant used to calculate the rank of a page.
///
/// # Returns
///
/// * The ranker constant.
///
/// # Notes
///
/// * If the `RANKER_CONSTANT` environment variable isn't set, the default value is used.
/// * The default value is `DEFAULT_RANKER_CONSTANT`.
#[must_use]
pub fn get_ranker_constant() -> f64 {
    std::env::var_os("RANKER_CONSTANT").map_or_else(
        || DEFAULT_RANKER_CONSTANT,
        |ranker_constant| {
            let Some(ranker_constant) = ranker_constant.to_str() else {
                warn!("Failed to parse RANKER_CONSTANT to string slice, defaulting to {DEFAULT_RANKER_CONSTANT}...",);

                return DEFAULT_RANKER_CONSTANT;
            };

            match ranker_constant.parse::<f64>() {
                Ok(ranker_constant) => ranker_constant,
                Err(why) => {
                    warn!("RANKER_CONSTANT isn't a valid number, defaulting to {DEFAULT_RANKER_CONSTANT}... (Error: {why})");

                    DEFAULT_RANKER_CONSTANT
                }
            }
        },
    )
}

/// Get the rating factor used to calculate the rank of a page.
///
/// # Returns
///
/// * The rating factor.
///
/// # Notes
///
/// * If the `RATING_FACTOR` environment variable isn't set, the default value is used.
/// * The default value is `DEFAULT_RATING_FACTOR`.
#[must_use]
pub fn get_rating_factor() -> f64 {
    std::env::var_os("RATING_FACTOR").map_or_else(
        || DEFAULT_RATING_FACTOR,
        |rating_factor| {
            let Some(rating_factor) = rating_factor.to_str() else {
                warn!("Failed to parse RATING_FACTOR to string slice, defaulting to {DEFAULT_RATING_FACTOR}...",);

                return DEFAULT_RATING_FACTOR;
            };

            match rating_factor.parse::<f64>() {
                Ok(rating_factor) => rating_factor,
                Err(why) => {
                    warn!("RATING_FACTOR isn't a valid number, defaulting to {DEFAULT_RATING_FACTOR}... (Error: {why})");

                    DEFAULT_RATING_FACTOR
                }
            }
        },
    )
}
