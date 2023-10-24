use crate::model::{NewPage, Page};
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
    title: Option<&str>,
    description: Option<&str>,
) -> Result<Page, diesel::result::Error> {
    use crate::schema::pages::dsl::pages;

    let new_page = NewPage {
        url: url.to_string(),
        title: title.map(std::string::ToString::to_string),
        description: description.map(std::string::ToString::to_string),
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

/// Creates a new keyword.
///
/// # Arguments
///
/// * `conn`: The database connection.
/// * `page_id`: The ID of the page the keyword is on.
/// * `keyword`: The keyword and its frequency.
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error>>` - If the keyword was successfully created.
/// * `Err(Box<dyn std::error::Error>)` - If the keyword was not created.
///
/// # Errors
///
/// * If the keyword could not be created.
pub async fn create_keyword(
    conn: &mut AsyncPgConnection,
    page_id: i32,
    keyword: (&str, &i32),
) -> Result<(), diesel::result::Error> {
    use crate::schema::keywords::dsl::keywords;

    let (keyword, frequency) = keyword;
    diesel::insert_into(keywords)
        .values((
            schema::keywords::page_id.eq(page_id),
            schema::keywords::keyword.eq(keyword),
            schema::keywords::frequency.eq(frequency),
        ))
        .execute(conn)
        .await?;

    Ok(())
}

/// Creates a new link.
///
/// # Arguments
///
/// * `conn`: The database connection.
/// * `page_id`: The ID of the page the link is on.
/// * `url`: The URL of the link.
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error>>` - If the link was successfully created.
/// * `Err(Box<dyn std::error::Error>)` - If the link was not created.
///
/// # Errors
///
/// * If the link could not be created.
pub async fn create_link(
    conn: &mut AsyncPgConnection,
    page_id: i32,
    url: &str,
) -> Result<(), diesel::result::Error> {
    use crate::schema::forward_links::dsl::forward_links;

    diesel::insert_into(forward_links)
        .values((
            schema::forward_links::page_id.eq(page_id),
            schema::forward_links::url.eq(url),
        ))
        .execute(conn)
        .await?;

    Ok(())
}

pub async 