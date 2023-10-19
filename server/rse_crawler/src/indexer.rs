use crate::utils::db::model::{ForwardLink, Keyword, Page};
use crate::utils::db::schema::pages::dsl::pages;
use diesel_async::{AsyncMysqlConnection, RunQueryDsl};

/// Create a new website in the database.
///
/// # Arguments
///
/// * `page`: The page to create.
/// * `keywords`: The keywords for the page.
/// * `forward_links`: The forward links for the page.
///
/// # Returns
///
/// * `Ok(())` if the website was created successfully, otherwise an `Err`.
pub async fn new_website(
    page: &Page,
    keywords: &Option<Vec<Keyword>>,
    forward_links: &Option<Vec<ForwardLink>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = crate::utils::env::database::establish_connection().await?;

    create_page(&mut conn, page).await?;

    if let Some(keywords) = keywords {
        for keyword in keywords {
            create_keyword(&mut conn, &keyword).await?;
        }
    }

    if let Some(forward_links) = forward_links {
        for forward_link in forward_links {
            create_forward_link(&mut conn, &forward_link).await?;
        }
    }

    Ok(())
}

/// Create a page in the database.
///
/// # Arguments
///
/// * `conn`: The database connection.
/// * `page`: The page to create.
///
/// # Returns
///
/// * `Ok(())` if the page was created successfully, otherwise an `Err`.
pub async fn create_page(
    conn: &mut AsyncMysqlConnection,
    page: &Page,
) -> Result<(), diesel::result::Error> {
    diesel::insert_into(pages)
        .values(page)
        .execute(conn)
        .await?;

    Ok(())
}

/// Create a keyword in the database.
///
/// # Arguments
///
/// * `conn`: The database connection.
/// * `keyword`: The keyword to create.
///
/// # Returns
///
/// * `Ok(())` if the keyword was created successfully, otherwise an `Err`.
pub async fn create_keyword(
    conn: &mut AsyncMysqlConnection,
    keyword: &Keyword,
) -> Result<(), diesel::result::Error> {
    diesel::insert_into(crate::utils::db::schema::keywords::table)
        .values(keyword)
        .execute(conn)
        .await?;

    Ok(())
}

/// Create a forward link in the database.
///
/// # Arguments
///
/// * `conn`: The database connection.
/// * `forward_link`: The forward link to create.
///
/// # Returns
///
/// * `Ok(())` if the forward link was created successfully, otherwise an `Err`.
pub async fn create_forward_link(
    conn: &mut AsyncMysqlConnection,
    forward_link: &ForwardLink,
) -> Result<(), diesel::result::Error> {
    diesel::insert_into(crate::utils::db::schema::forward_links::table)
        .values(forward_link)
        .execute(conn)
        .await?;

    Ok(())
}
