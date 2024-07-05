CREATE TABLE websites
(
    id              SERIAL PRIMARY KEY,

    url             VARCHAR(8192) NOT NULL UNIQUE,
    last_crawled_at TIMESTAMP     NOT NULL DEFAULT now()
);

CREATE INDEX url_idx ON websites (url);
