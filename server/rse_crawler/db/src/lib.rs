use diesel::{ConnectionResult, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use crate::model::{NewPage, Page};

pub mod model;
mod schema;

async fn get_connection() -> ConnectionResult<AsyncPgConnection> {
    let url = std::env::var_os("DATABASE_URL")
        .expect("DATABASE_URL must be set!")
        .to_str()
        .expect("DATABASE_URL must be valid UTF-8!")
        .to_string();

    AsyncPgConnection::establish(&url).await
}

/// Creates a new page.
///
/// # Arguments
///
/// * `conn`: The database connection.
///
/// * `url`: The URL of the page.
///
/// * `title`: The title of the page.
/// * `description`: The description of the page.
///
/// # Returns
///
/// * A `Result` with the created page if successful.
///
/// # Errors
///
/// * If the page could not be created.
pub async fn create_page(conn: &mut AsyncPgConnection, url: &str, title: Option<&str>, description: Option<&str>) -> Result<Page, diesel::result::Error> {
    use crate::schema::pages;

    let new_page = NewPage {
        url: url.to_string(),
        title: title.map(std::string::ToString::to_string),
        description: description.map(std::string::ToString::to_string),
    };

    diesel::insert_into(pages::table)
        .values(&new_page)
        .returning(Page::as_returning())
        .get_result(conn)
        .await
}

/// Gets the oldest pages.
///
/// # Arguments
///
/// * `conn`: The database connection.
/// * `limit`: The maximum number of pages to get.
///
/// # Returns
///
/// * A `Result` with the oldest pages if successful.
///
/// # Errors
///
/// * If the pages could not be retrieved.
pub async fn get_oldest_pages(limit: i64) -> Result<Vec<Page>, Box<dyn std::error::Error>> {
    use crate::schema::pages::dsl::{last_crawled_at, pages};

    let Ok(mut conn) = get_connection().await else {
        return Err("Failed to get database connection!".into());
    };

    Ok(pages
        .order(last_crawled_at.asc())
        .limit(limit)
        .load(&mut conn)
        .await?)
}
