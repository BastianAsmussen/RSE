-- Your SQL goes here
CREATE TABLE metadata
(
    id      SERIAL PRIMARY KEY,
    page_id INT           NOT NULL,

    name    VARCHAR(1024) NOT NULL,
    content VARCHAR(1024) NOT NULL,

    FOREIGN KEY (page_id) REFERENCES pages (id) ON DELETE CASCADE
)