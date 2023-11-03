use crate::database::model::{ForwardLink, Keyword, NewForwardLink, NewKeyword, NewPage, Page};
use crate::errors::Error;
use diesel::{ConnectionResult, ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use url::Url;

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
/// * `Err(Error)` - If the page could not be created.
///
/// # Errors
///
/// * If the page could not be created.
pub async fn create_page(
    conn: &mut AsyncPgConnection,
    url: &Url,
    title: Option<&str>,
    description: Option<&str>,
) -> Result<Page, Error> {
    use crate::database::schema::pages::dsl::pages;

    if let Some(page) = get_page_by_url(conn, url).await? {
        info!("Page already exists: {}", url.to_string());

        return Ok(page);
    };

    let new_page = NewPage {
        url: url.to_string(),

        title: title.map(std::string::ToString::to_string),
        description: description.map(std::string::ToString::to_string),
    };

    Ok(diesel::insert_into(pages)
        .values(&new_page)
        .returning(Page::as_returning())
        .get_result(conn)
        .await?)
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
/// * `Err(Error)` - If the pages could not be retrieved.
///
/// # Errors
///
/// * If the pages could not be retrieved.
pub async fn get_oldest_pages(limit: i64) -> Result<Vec<Page>, Error> {
    use crate::database::schema::pages::dsl::pages;
    use crate::database::schema::pages::last_crawled_at;

    let Ok(mut conn) = get_connection().await else {
        return Err(Error::Database("Failed to get database connection!".into()));
    };

    Ok(pages
        .order(last_crawled_at.asc())
        .limit(limit)
        .load(&mut conn)
        .await?)
}

/// Creates new keywords.
///
/// # Arguments
///
/// * `conn`: The database connection.
/// * `data`: The keywords to create.
///
/// # Returns
///
/// * `Ok(())`: If the keywords were successfully created.
/// * `Err(diesel::result::Error)`: If the keywords weren't successfully inserted.
///
/// # Errors
///
/// * If the database failed to create the keywords.
pub async fn create_keywords(
    conn: &mut AsyncPgConnection,
    data: &[NewKeyword],
) -> Result<(), diesel::result::Error> {
    use crate::database::schema::keywords::dsl::keywords;

    diesel::insert_into(keywords)
        .values(data)
        .execute(conn)
        .await?;

    Ok(())
}

/// Creates new forward links.
///
/// # Arguments
///
/// * `conn`: The database connection.
/// * `from_page_url`: The page to create forward links for.
/// * `to_page_urls`: The pages to create forward links to.
///
/// # Returns
///
/// * `Ok(())` - If the forward links were successfully created.
/// * `Err(Error>)` - If the forward links were not created.
///
/// # Errors
///
/// * If the forward links could not be created.
pub async fn create_forward_links<S>(
    conn: &mut AsyncPgConnection,
    from_page_url: &Url,
    to_page_urls: &HashMap<Url, i32, S>,
) -> Result<(), Error>
where
    S: std::hash::BuildHasher + Send + Sync,
    RandomState: std::hash::BuildHasher,
{
    use crate::database::schema::forward_links::dsl::forward_links;

    // Get the page we're creating forward links for.
    let Some(from_page) = get_page_by_url(conn, from_page_url).await? else {
        return Err(Error::Database(format!(
            "Failed to find page with URL: {from_page_url}!"
        )));
    };

    let mut new_forward_links = Vec::new();
    for (to_url, frequency) in to_page_urls {
        /*
        let Some(to_page) = get_page_by_url(conn, to_url).await? else {
            return Err(Error::Database(format!(
                "Failed to find page with URL: {to_url}!"
            )));
        };

        new_forward_links.push(NewForwardLink {
            from_page_id: from_page.id,
            to_page_id: to_page.id,
            frequency: *frequency,
        });
         */

        new_forward_links.push(NewForwardLink {
            from_page_id: from_page.id,
            to_page_url: to_url.to_string(),
            frequency: *frequency,
        });
    }

    diesel::insert_into(forward_links)
        .values(new_forward_links)
        .execute(conn)
        .await?;

    Ok(())
}

/// Gets a page by its ID.
///
/// # Arguments
///
/// * `conn`: The database connection.
/// * `page_id`: The ID of the page.
///
/// # Returns
///
/// * `Ok(Some(Page))` - The page if successful.
/// * `Ok(None)` - If no page was found.
/// * `Err(diesel::result::Error)` - If the page could not be retrieved.
///
/// # Errors
///
/// * If the page could not be retrieved.
pub async fn get_page_by_id(
    conn: &mut AsyncPgConnection,
    page_id: i32,
) -> Result<Option<Page>, Error> {
    use crate::database::schema::pages::dsl::pages;
    use crate::database::schema::pages::id;

    Ok(pages
        .filter(id.eq(page_id))
        .select(Page::as_select())
        .first(conn)
        .await
        .optional()?)
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
/// * `Ok(Some(Page))` - The page, if successful.
/// * `Ok(None)` - If no page was found.
/// * `Err(Error)` - If the page could not be retrieved.
///
/// # Errors
///
/// * If the page could not be retrieved.
pub async fn get_page_by_url(
    conn: &mut AsyncPgConnection,
    url: &Url,
) -> Result<Option<Page>, Error> {
    use crate::database::schema::pages::dsl::{pages, url as url_column};

    Ok(pages
        .filter(url_column.eq(url.to_string()))
        .select(Page::as_select())
        .first(conn)
        .await
        .optional()?)
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct CompletePage {
    pub page: Page,
    pub keywords: Option<Vec<Keyword>>,
}

/// Gets keywords by page ID.
///
/// # Arguments
///
/// * `conn`: The database connection.
/// * `page_id`: The ID of the page.
///
/// # Returns
///
/// * `Ok(Some(Vec<Keyword>))` - The keywords if successful.
/// * `Ok(None)` - If no keywords were found.
/// * `Err(diesel::result::Error)` - If the keywords could not be retrieved.
///
/// # Errors
///
/// * If the keywords could not be retrieved.
pub async fn get_keywords_by_page_id(
    conn: &mut AsyncPgConnection,
    page_id: i32,
) -> Result<Option<Vec<Keyword>>, diesel::result::Error> {
    use crate::database::schema::keywords::dsl::{keywords, page_id as page_id_column};

    keywords
        .filter(page_id_column.eq(page_id))
        .select(Keyword::as_select())
        .load(conn)
        .await
        .optional()
}

/// Get a series of pages matching a list of words
///
/// # Arguments
///
/// * `conn`: The database connection.
/// * `words`: The words to search for.
///
/// # Returns
///
/// * `Ok(Some(Vec<Page>))` - The pages if successful.
/// * `Ok(None)` - If no pages were found.
/// * `Err(Error)` - If the pages could not be retrieved.
///
/// # Errors
///
/// * If the pages could not be retrieved.
pub async fn get_pages_with_words(
    conn: &mut AsyncPgConnection,
    words: Vec<String>,
) -> Result<Option<Vec<Page>>, Error> {
    use crate::database::schema::keywords::dsl::keywords;
    use crate::database::schema::pages::dsl::pages;

    // Search for pages that contain the words in their keywords.
    let pages_with_keywords = keywords
        .filter(schema::keywords::dsl::word.eq_any(&words))
        .inner_join(pages)
        .distinct()
        .select(Page::as_select())
        .load(conn)
        .await
        .optional()?;

    // Search for pages that contain the words in their title or description.
    let pages_with_title = pages
        .filter(schema::pages::dsl::title.eq_any(&words))
        .select(Page::as_select())
        .load(conn)
        .await
        .optional()?;

    let pages_with_description = pages
        .filter(schema::pages::dsl::description.eq_any(&words))
        .select(Page::as_select())
        .load(conn)
        .await
        .optional()?;

    // Combine the results.
    let mut found_pages = Vec::new();

    if let Some(mut data) = pages_with_keywords {
        found_pages.append(&mut data);
    }

    if let Some(mut data) = pages_with_title {
        found_pages.append(&mut data);
    }

    if let Some(mut data) = pages_with_description {
        found_pages.append(&mut data);
    }

    if found_pages.is_empty() {
        return Ok(None);
    }

    Ok(Some(found_pages))
}

/// Get the backlinks for a given page.
///
/// # Arguments
///
/// * `conn`: The database connection.
/// * `page`: The page to get backlinks for.
///
/// # Returns
///
/// * `Ok(Vec<CompletePage>)` - The backlinks if successful.
/// * `Err(Box<dyn std::errors::Error>)` - If the backlinks could not be retrieved.
///
/// # Errors
///
/// * If the backlinks could not be retrieved.
pub async fn get_backlinks(
    conn: &mut AsyncPgConnection,
    page: &CompletePage,
) -> Result<Vec<CompletePage>, Error> {
    use crate::database::schema::forward_links::dsl::forward_links;
    use crate::database::schema::pages::dsl::pages;

    let mut backlinks = Vec::new();

    let links = forward_links
        .filter(schema::forward_links::dsl::to_page_url.eq(&page.page.url))
        .select(ForwardLink::as_select())
        .load(conn)
        .await?;

    for link in links {
        let page = pages
            .filter(schema::pages::dsl::id.eq(link.from_page_id))
            .select(Page::as_select())
            .first(conn)
            .await?;

        let keywords = get_keywords_by_page_id(conn, page.id).await?;

        let complete_page = CompletePage { page, keywords };

        backlinks.push(complete_page);
    }

    Ok(backlinks)
}
