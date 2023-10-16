use std::env;
use std::error::Error;
use log::error;

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
fn get_database_url() -> Result<String, Box<dyn Error>> {
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
/// # Returns
/// * A `Result` containing either a `Pool` or an `Error` type.
///
/// # Errors
/// * The function will early return if it fails to establish a temporary connection to the database.
pub async fn get_pool() -> Result<Pool, Box<dyn Error>> {
    let Ok(database_url) = get_database_url() else {
        error!("Missing environment variables!");

        return Err("Missing environment variables!".into());
    };

    let pool = Pool::new(&*database_url);

    // Test the connection.
    let conn = pool.get_conn().await?;
    drop(conn);

    Ok(pool)
}
