use std::env;
use std::error::Error;

use mysql_async::Pool;

/// Based on the environment variables, this function will generate a URL to connect to the database.
///
/// # Returns
/// * A `Result` containing either a `String` representation of the URL, or an `Error` type.
///
/// # Errors
/// * The function will early return if one or more of the following environment variables are not set:
/// - `MYSQL_SERVER`
/// - `MYSQL_PORT`
/// - `MYSQL_DATABASE`
/// - `MYSQL_USERNAME`
/// - `MYSQL_PASSWORD`
pub fn get_database_url() -> Result<String, Box<dyn Error>> {
    let server = env::var("MYSQL_SERVER")?;
    let port = env::var("MYSQL_PORT")?;
    let database = env::var("MYSQL_DATABASE")?;
    let username = env::var("MYSQL_USERNAME")?;
    let password = env::var("MYSQL_PASSWORD")?;

    let url = format!("mysql://{username}:{password}@{server}:{port}/{database}");

    Ok(url)
}

/// Get a `Pool` for the database.
///
/// # Arguments
/// * `database_url`: A string slice representation of the databae URL.
///
/// # Returns
/// * A `Result` containing either a `Pool` or an `Error` type.
///
/// # Errors
/// * The function will early return if it fails to establish a temporary connection to the database.
pub async fn get_pool(database_url: &str) -> Result<Pool, Box<dyn Error>> {
    let pool = Pool::new(database_url);

    // Test the connection.
    let conn = pool.get_conn().await?;
    drop(conn);

    Ok(pool)
}
