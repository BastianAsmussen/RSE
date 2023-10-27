CREATE TABLE keywords
(
    id        SERIAL PRIMARY KEY,
    page_id   INT          NOT NULL,

    word      VARCHAR(128) NOT NULL,
    frequency INT          NOT NULL DEFAULT 1,

    FOREIGN KEY (page_id) REFERENCES pages (id) ON DELETE CASCADE
)
