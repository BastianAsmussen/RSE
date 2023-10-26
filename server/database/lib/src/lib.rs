use crate::model::{NewForwardLink, NewMetadata, NewPage, Page, Metadata};
use diesel::{ConnectionResult, ExpressionMethods, QueryDsl, SelectableHelper, TextExpressionMethods};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use std::collections::HashMap;
use url::Url;
use log::{error, info};
use serde::Deserialize;

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
/// # Returns
///
/// * `Ok(Page)` - The created page if successful.
/// * `Err(Box<dyn std::error::Error>)` - If the page could not be created.
///
/// # Errors
///
/// * If the page could not be created.
pub async fn create_page(
    conn: &mut AsyncPgConnection,
    url: &Url,
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
/// * `Ok(Vec<Page>)` - The oldest pages if successful.
/// * `Err(Box<dyn std::error::Error>)` - If the pages could not be retrieved.
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
/// * `Ok(())` - If the metadata was successfully created.
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
/// * `Ok(())` - If the links were successfully created.
/// * `Err(Box<dyn std::error::Error>)` - If the links were not created.
///
/// # Errors
///
/// * If the links could not be created.
pub async fn create_links<S>(
    conn: &mut AsyncPgConnection,
    from_url: &Url,
    links: &HashMap<Url, i32, S>,
) -> Result<(), diesel::result::Error>
where
    S: std::hash::BuildHasher + Send + Sync,
{
    use crate::schema::forward_links::dsl::forward_links;

    let mut new_links = Vec::new();
    for (to_url, frequency) in links {
        let from_page = match get_page_by_url(conn, from_url).await {
            Ok(page) => page,
            Err(err) => {
                error!("{err}");

                continue;
            }
        };
        let to_page = match get_page_by_url(conn, to_url).await {
            Ok(page) => page,
            Err(err) => {
                error!("{err}");

                continue;
            }
        };

        new_links.push(NewForwardLink {
            from_page_id: from_page.id,
            to_page_id: to_page.id,
            frequency: *frequency,
        });
    }

    let total_links = new_links.len();

    match diesel::insert_into(forward_links)
        .values(new_links)
        .execute(conn)
        .await {
        Ok(_) => {
            info!("Inserted {total_links} links.");

            Ok(())
        }
        Err(err) => {
            error!("{err}");

            Err(err)
        },
    }
}

/// Gets a page by its URL.
///
/// # Arguments
///
/// * `conn`: The database connection.
/// * `url`: The URL of the page.
///
/// # Returns
///
/// * `Ok(Page)` - The page if successful.
/// * `Err(diesel::result::Error)` - If the page could not be retrieved.
///
/// # Errors
///
/// * If the page could not be retrieved.
pub async fn get_page_by_url(
    conn: &mut AsyncPgConnection,
    url: &Url,
) -> Result<Page, diesel::result::Error> {
    use crate::schema::pages::dsl::{pages, url as url_column};

    pages.filter(url_column.eq(url.as_str())).first(conn).await
}

/// Gets metadata by page ID.
///
/// # Arguments
///
/// * `conn`: The database connection.
/// * `page_id`: The ID of the page.
///
/// # Returns
///
/// * `Ok(Vec<Metadata>)` - The metadata if successful.
/// * `Err(diesel::result::Error)` - If the metadata could not be retrieved.
///
/// # Errors
///
/// * If the metadata could not be retrieved.
pub async fn get_metadata_by_page_id(
    conn: &mut AsyncPgConnection,
    page_id: i32,
) -> Result<Vec<Metadata>, diesel::result::Error> {
    use crate::schema::metadata::dsl::{metadata, page_id as page_id_column};

    metadata
        .filter(page_id_column.eq(page_id))
        .select(Metadata::as_select())
        .load(conn)
        .await
}

#[derive(Debug, Deserialize)]
pub struct CompletePage {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<String>,
}

pub async fn get_description_like(
    conn: &mut AsyncPgConnection,
    query: &str,
) -> Result<Vec<CompletePage>, diesel::result::Error> {
    use crate::schema::pages::dsl::{pages};
    use crate::schema::metadata::dsl::{metadata};

    let data = metadata
        .filter(schema::metadata::dsl::name.eq("description"))
        .filter(schema::metadata::dsl::content.like(format!("%{query}%")))
        .select(Metadata::as_select())
        .load(conn)
        .await?;

    let page = pages
        .filter(schema::pages::dsl::id.eq(data[0].page_id))
        .select(Page::as_select())
        .load(conn)
        .await?;

    let mut complete_pages = Vec::new();
    for value in data {
        let description = value.content;

        complete_pages.push(CompletePage {
            url: page[0].url.clone(),
            title: None,
            description: Some(description),
            keywords: None,
        });
    }

    Ok(complete_pages)
}