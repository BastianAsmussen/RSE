-- Your SQL goes here
CREATE TABLE pages
(
    id              SERIAL PRIMARY KEY,

    url             VARCHAR(2048) NOT NULL UNIQUE,

    title           VARCHAR(255)           DEFAULT NULL,
    description     VARCHAR(1024)          DEFAULT NULL,

    last_crawled_at TIMESTAMP     NOT NULL DEFAULT NOW()
)