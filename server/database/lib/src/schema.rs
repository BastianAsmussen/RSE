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
    metadata (id) {
        id -> Int4,
        page_id -> Int4,
        #[max_length = 512]
        name -> Varchar,
        #[max_length = 512]
        content -> Varchar,
    }
}

diesel::table! {
    pages (id) {
        id -> Int4,
        #[max_length = 8192]
        url -> Varchar,
        last_crawled_at -> Timestamp,
    }
}

diesel::joinable!(forward_links -> pages (page_id));
diesel::joinable!(metadata -> pages (page_id));

diesel::allow_tables_to_appear_in_same_query!(
    forward_links,
    metadata,
    pages,
);
