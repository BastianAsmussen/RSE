use log::warn;
use std::env;

/// The default IP and port to listen on.
const DEFAULT_LISTENING_ADDRESS: (&str, u16) = ("0.0.0.0", 8080);

/// Get the IP and port to listen on.
///
/// # Returns
///
/// * `(String, u16)`: The IP and port to listen on.
///
/// # Panics
///
/// * If `LISTENING_ADDRESS` is not valid UTF-8.
/// * If `LISTENING_ADDRESS` is not in the format `ip:port`.
/// * If `LISTENING_ADDRESS`'s port is not a valid number.
/// * If `LISTENING_ADDRESS`'s port is not in the range `0..=65535`.
#[must_use]
#[allow(clippy::expect_used)]
pub fn get_address() -> (String, u16) {
    let (default_ip, default_port) = DEFAULT_LISTENING_ADDRESS;
    env::var_os("LISTENING_ADDRESS").map_or_else(
        || {
            warn!("LISTENING_ADDRESS is not set! Using default value of \"{default_ip}:{default_port}\"...");

            (default_ip.to_string(), default_port)
        },
        |listening_address| {
            let listening_address = listening_address.to_str().expect("LISTENING_ADDRESS must be valid UTF-8!");

            let mut listening_address = listening_address.split(':');
            let ip = listening_address.next().expect("LISTENING_ADDRESS must be in the format \"ip:port\"!").to_string();
            let port = listening_address.next().expect("LISTENING_ADDRESS must be in the format \"ip:port\"!").parse::<u16>().expect("LISTENING_ADDRESS must be in the format \"ip:port\"!");

            (ip, port)
        }
    )
}
