// @generated automatically by Diesel CLI.

diesel::table! {
    forward_links (from_page_id, to_page_id) {
        from_page_id -> Int4,
        to_page_id -> Int4,
        frequency -> Int4,
    }
}

diesel::table! {
    keywords (id) {
        id -> Int4,
        page_id -> Int4,
        #[max_length = 128]
        word -> Varchar,
        frequency -> Int4,
    }
}

diesel::table! {
    metadata (id) {
        id -> Int4,
        page_id -> Int4,
        #[max_length = 1024]
        name -> Varchar,
        #[max_length = 1024]
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

diesel::joinable!(keywords -> pages (page_id));
diesel::joinable!(metadata -> pages (page_id));

diesel::allow_tables_to_appear_in_same_query!(forward_links, keywords, metadata, pages,);
