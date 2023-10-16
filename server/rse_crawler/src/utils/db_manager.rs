use std::env;
use std::error::Error;

use log::info;

use mysql_async::prelude::Queryable;
use mysql_async::{Conn, Pool, Row};
use spider::page::Page;

use sha256::digest;

/// Based on the environment variables, this function will generate a URL to connect to the database.
///
/// # Returns
///
///* A `Result` containing either a `String` representation of the URL, or an `Error` type.
///
/// # Errors
///
///* The function will early return if one or more of the following environment variables are not set:
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
///
/// * `database_url`: A string slice representation of the databae URL.
///
/// # Returns
///
/// * A `Result` containing either a `Pool` or an `Error` type.
///
/// # Errors
///
/// * The function will early return if it fails to establish a temporary connection to the database.
pub async fn get_pool(database_url: &str) -> Result<Pool, Box<dyn Error>> {
    let pool = Pool::new(database_url);

    // Test the connection.
    let conn = pool.get_conn().await?;
    drop(conn);

    Ok(pool)
}

/// Checks if a given page is up to date or not.
///
/// # Arguments
///
/// * `conn`: A mutable reference to a database connection.
/// * `page`: A reference to the `Page` in question.
///
/// # Returns
///
/// * A `bool` being either `true` or `false` depeinding on whether the database has an up to date copy of the `page`.
pub async fn is_page_up_to_date(conn: &mut Conn, page: &Page) -> bool {
    let select_query = "SELECT content FROM web_pages WHERE url = ?";
    let result = conn
        .exec_map(select_query, (page.get_url(),), |row: Row| {
            // Extract the 'content' column from the result row
            row.get_opt::<String, &str>("content")
        })
        .await;

    if let Ok(content) = result {
        if content.is_empty() {
            info!("No content for {} in database...", page.get_url());

            return false;
        }

        let Some(Ok(content)) = content[0].clone() else {
            info!("No content for {} in database...", page.get_url());

            return false;
        };

        // Get the local hash from the database and compare it with the "fresh" hash we just scraped.
        let local_hash = digest(content);
        let fresh_hash = digest(page.get_html());

        if local_hash == fresh_hash {
            info!("{} is up to date!", page.get_url());

            true
        } else {
            info!("{} isn't up to date!", page.get_url());

            false
        }
    } else {
        info!("No content for {} in database...", page.get_url());

        false
    }
}
