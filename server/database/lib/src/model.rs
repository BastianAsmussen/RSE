use diesel::{Insertable, Queryable, Selectable};
use std::time::SystemTime;

/// A web page.
///
/// # Fields
///
/// * `id`: The ID of the page.
///
/// * `url`: The URL of the page.
/// * `last_crawled_at`: The last time the page was crawled.
#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::pages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Page {
    pub id: i32,

    pub url: String,
    pub last_crawled_at: SystemTime,
}

/// A new web page.
///
/// # Fields
///
/// * `url`: The URL of the page.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::pages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewPage {
    pub url: String,
}

/// A metadata value.
///
/// # Fields
///
/// * `id`: The ID of the keyword.
/// * `page_id`: The ID of the page the keyword is on.
///
/// * `name`: The name of the metadata value.
/// * `content`: The content of the metadata value.
#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::metadata)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Metadata {
    pub id: i32,
    pub page_id: i32,

    pub name: String,
    pub content: String,
}

/// A new metadata value.
///
/// # Fields
///
/// * `page_id`: The ID of the page the keyword is on.
///
/// * `name`: The name of the metadata.
/// * `content`: The content of the metadata.
#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::metadata)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewMetadata {
    pub page_id: i32,

    pub name: String,
    pub content: String,
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
