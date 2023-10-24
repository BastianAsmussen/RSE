-- Your SQL goes here
CREATE TABLE keywords
(
    id        SERIAL PRIMARY KEY,
    page_id   INT         NOT NULL,
    keyword   VARCHAR(64) NOT NULL,
    frequency INT         NOT NULL,

    FOREIGN KEY (page_id) REFERENCES pages (id) ON DELETE CASCADE
)