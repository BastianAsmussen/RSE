USE rse;

CREATE TABLE IF NOT EXISTS pages
(
    id          INT AUTO_INCREMENT PRIMARY KEY,

    url         VARCHAR(2048) NOT NULL,

    title       VARCHAR(255) DEFAULT NULL,
    description VARCHAR(255) DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS keywords
(
    id        INT AUTO_INCREMENT PRIMARY KEY,
    page_id   INT,
    keyword   VARCHAR(64),
    frequency INT,

    FOREIGN KEY (page_id) REFERENCES pages (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS forward_links
(
    id      INT AUTO_INCREMENT PRIMARY KEY,
    page_id INT,
    url     VARCHAR(2048),

    FOREIGN KEY (page_id) REFERENCES pages (id) ON DELETE CASCADE
);

-- -- Create full-text indexes for efficient text-based search.
-- ALTER TABLE pages
--     ADD FULLTEXT INDEX pages_title_idx (title),
--     ADD FULLTEXT INDEX pages_description_idx (description);
--
-- ALTER TABLE keywords
--     ADD FULLTEXT INDEX keywords_keyword_idx (keyword);
