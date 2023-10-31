use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// A web page.
///
/// # Fields
///
/// * `id`: The ID of the page.
///
/// * `url`: The URL of the page.
/// * `last_crawled_at`: The last time the page was crawled.
#[derive(
    Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Queryable, Selectable, Insertable,
)]
#[diesel(table_name = crate::schema::pages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Page {
    pub id: i32,

    pub url: String,
    pub last_crawled_at: SystemTime,

    pub title: Option<String>,
    pub description: Option<String>,
}

/// A new web page.
///
/// # Fields
///
/// * `url`: The URL of the page.
///
/// * `title`: The title of the page.
/// * `description`: The description of the page.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::pages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewPage {
    pub url: String,

    pub title: Option<String>,
    pub description: Option<String>,
}

/// A keyword.
///
/// # Fields
///
/// * `id`: The ID of the keyword.
/// * `page_id`: The ID of the page the keyword is on.
///
/// * `word`: The word of the keyword.
/// * `frequency`: The frequency of the keyword.
#[derive(Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::keywords)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Keyword {
    pub id: i32,
    pub page_id: i32,

    pub word: String,
    pub frequency: i32,
}

/// A new keyword.
///
/// # Fields
///
/// * `page_id`: The ID of the page the keyword is on.
///
/// * `word`: The word of the keyword.
/// * `frequency`: The frequency of the keyword.
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::keywords)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewKeyword {
    pub page_id: i32,

    pub word: String,
    pub frequency: i32,
}

/// A forward link.
///
/// # Fields
///
/// * `from_page_id`: The ID of the page the forward link is on.
/// * `to_page_id`: The ID of the page the forward link points to.
///
/// * `frequency`: The frequency of the forward link.
#[derive(Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::forward_links)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ForwardLink {
    pub from_page_id: i32,
    pub to_page_id: i32,

    pub frequency: i32,
}

/// A new forward link.
///
/// # Fields
///
/// * `from_page_id`: The ID of the page the forward link is on.
/// * `to_page_id`: The ID of the page the forward link points to.
///
/// * `frequency`: The frequency of the forward link.
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::forward_links)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewForwardLink {
    pub from_page_id: i32,
    pub to_page_id: i32,

    pub frequency: i32,
}
