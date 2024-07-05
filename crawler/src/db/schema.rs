// @generated automatically by Diesel CLI.

diesel::table! {
    websites (id) {
        id -> Int4,
        #[max_length = 8192]
        url -> Varchar,
        last_crawled_at -> Timestamp,
    }
}
