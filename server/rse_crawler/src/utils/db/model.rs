use diesel::{Insertable, Queryable, Selectable};

/// A web page.
///
/// # Fields
///
/// * `url`: The URL of the page.
/// * `title`: The title of the page.
/// * `description`: The description of the page.
#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::utils::db::schema::pages)]
pub struct Page {
    pub url: String,

    pub title: Option<String>,
    pub description: Option<String>,
}

/// A keyword.
///
/// # Fields
///
/// * `keyword`: The keyword.
/// * `frequency`: The frequency of the keyword.
#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::utils::db::schema::keywords)]
pub struct Keyword {
    pub keyword: String,
    pub frequency: i32,
}

/// A forward link.
///
/// # Fields
///
/// * `url`: The URL of the forward link.
#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::utils::db::schema::forward_links)]
pub struct ForwardLink {
    pub url: String,
}
