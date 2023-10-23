-- Your SQL goes here
CREATE TABLE forward_links
(
    id      SERIAL PRIMARY KEY,
    page_id INT           NOT NULL,
    url     VARCHAR(2048) NOT NULL,

    FOREIGN KEY (page_id) REFERENCES pages (id) ON DELETE CASCADE
)