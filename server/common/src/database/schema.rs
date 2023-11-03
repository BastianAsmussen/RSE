// @generated automatically by Diesel CLI.

diesel::table! {
    forward_links (from_page_id, to_page_url) {
        from_page_id -> Int4,
        #[max_length = 8192]
        to_page_url -> Varchar,
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
    pages (id) {
        id -> Int4,
        #[max_length = 8192]
        url -> Varchar,
        last_crawled_at -> Timestamp,
        #[max_length = 256]
        title -> Nullable<Varchar>,
        #[max_length = 1024]
        description -> Nullable<Varchar>,
    }
}

diesel::joinable!(forward_links -> pages (from_page_id));
diesel::joinable!(keywords -> pages (page_id));

diesel::allow_tables_to_appear_in_same_query!(forward_links, keywords, pages,);
