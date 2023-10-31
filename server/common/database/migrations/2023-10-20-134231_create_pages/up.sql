CREATE TABLE pages
(
    id              SERIAL PRIMARY KEY,

    url             VARCHAR(8192) NOT NULL UNIQUE,
    last_crawled_at TIMESTAMP     NOT NULL DEFAULT NOW(),

    title           VARCHAR(256)           DEFAULT NULL,
    description     VARCHAR(1024)          DEFAULT NULL
);