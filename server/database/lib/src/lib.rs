use crate::model::{NewForwardLink, NewMetadata, NewPage, Page};
use diesel::{ConnectionResult, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};

pub mod model;
mod schema;

/// Gets a database connection.
///
/// # Returns
///
/// * `ConnectionResult<AsyncPgConnection>` - The database connection if successful.
///
/// # Errors
///
/// * If the database connection could not be established.
///
/// # Panics
///
/// * If the `DATABASE_URL` environment variable is not set.
/// * If the `DATABASE_URL` environment variable is not valid UTF-8.
#[allow(clippy::expect_used)]
pub async fn get_connection() -> ConnectionResult<AsyncPgConnection> {
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
/// * `Result<Page, Box<dyn std::error::Error>>` - The created page if successful.
/// * `Err(Box<dyn std::error::Error>)` - If the page could not be created.
///
/// # Errors
///
/// * If the page could not be created.
pub async fn create_page(
    conn: &mut AsyncPgConnection,
    url: &str,
) -> Result<Page, diesel::result::Error> {
    use crate::schema::pages::dsl::pages;

    let new_page = NewPage {
        url: url.to_string(),
    };

    diesel::insert_into(pages)
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

/// Creates new metadata.
///
/// # Arguments
///
/// * `conn`: The database connection.
/// * `data`: The metadata to create.
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error>>` - If the metadata was successfully created.
/// * `Err(Box<dyn std::error::Error>)` - If the metadata was not created.
///
/// # Errors
///
/// * If the metadata could not be created.
pub async fn create_metadata(
    conn: &mut AsyncPgConnection,
    data: &[NewMetadata],
) -> Result<(), diesel::result::Error> {
    use crate::schema::metadata::dsl::metadata;
    
    diesel::insert_into(metadata)
        .values(data)
        .execute(conn)
        .await?;

    Ok(())
}

/// Creates a new link.
///
/// # Arguments
///
/// * `conn`: The database connection.
/// 
/// * `links`: The links to create.
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error>>` - If the links were successfully created.
/// * `Err(Box<dyn std::error::Error>)` - If the links were not created.
///
/// # Errors
///
/// * If the links could not be created.
pub async fn create_links(
    conn: &mut AsyncPgConnection,
    links: &[NewForwardLink]
) -> Result<(), diesel::result::Error> {
    use crate::schema::forward_links::dsl::forward_links;

    diesel::insert_into(forward_links)
        .values(links)
        .execute(conn)
        .await?;

    Ok(())
}
