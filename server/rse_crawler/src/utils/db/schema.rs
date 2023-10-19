// @generated automatically by Diesel CLI.

diesel::table! {
    forward_links (id) {
        id -> Integer,
        page_id -> Nullable<Integer>,
        #[max_length = 255]
        url -> Nullable<Varchar>,
    }
}

diesel::table! {
    keywords (id) {
        id -> Integer,
        page_id -> Nullable<Integer>,
        #[max_length = 64]
        keyword -> Nullable<Varchar>,
        frequency -> Nullable<Integer>,
    }
}

diesel::table! {
    pages (id) {
        id -> Integer,
        #[max_length = 255]
        url -> Varchar,
        #[max_length = 255]
        title -> Nullable<Varchar>,
        #[max_length = 255]
        description -> Nullable<Varchar>,
    }
}

diesel::joinable!(forward_links -> pages (page_id));
diesel::joinable!(keywords -> pages (page_id));

diesel::allow_tables_to_appear_in_same_query!(forward_links, keywords, pages,);
