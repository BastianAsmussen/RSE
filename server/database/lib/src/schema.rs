// @generated automatically by Diesel CLI.

diesel::table! {
    forward_links (id) {
        id -> Int4,
        page_id -> Int4,
        #[max_length = 2048]
        url -> Varchar,
    }
}

diesel::table! {
    keywords (id) {
        id -> Int4,
        page_id -> Int4,
        #[max_length = 64]
        keyword -> Varchar,
        frequency -> Int4,
    }
}

diesel::table! {
    pages (id) {
        id -> Int4,
        #[max_length = 2048]
        url -> Varchar,
        #[max_length = 255]
        title -> Nullable<Varchar>,
        #[max_length = 1024]
        description -> Nullable<Varchar>,
        last_crawled_at -> Timestamp,
    }
}

diesel::joinable!(forward_links -> pages (page_id));
diesel::joinable!(keywords -> pages (page_id));

diesel::allow_tables_to_appear_in_same_query!(
    forward_links,
    keywords,
    pages,
);
