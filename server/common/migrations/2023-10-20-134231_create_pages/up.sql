CREATE TABLE pages
(
    id              SERIAL PRIMARY KEY,

    url             VARCHAR(8192) NOT NULL UNIQUE,
    last_crawled_at TIMESTAMP     NOT NULL DEFAULT NOW(),

    title           VARCHAR(256)           DEFAULT NULL,
    description     VARCHAR(1024)          DEFAULT NULL
);

-- Use indexing for faster searches.
CREATE INDEX pages_url_idx ON pages (url);
CREATE INDEX pages_title_idx ON pages (title);
CREATE INDEX pages_description_idx ON pages (description);