USE rse;

CREATE TABLE IF NOT EXISTS web_pages
(
    id          INT AUTO_INCREMENT PRIMARY KEY,
    url         VARCHAR(2048) NOT NULL,
    title       VARCHAR(255),
    description TEXT,
    content     TEXT,
    timestamp   TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS keywords
(
    id        INT AUTO_INCREMENT PRIMARY KEY,
    page_id   INT,
    keyword   VARCHAR(64),
    frequency INT,

    FOREIGN KEY (page_id) REFERENCES web_pages (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS users
(
    user_id  INT AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(64) NOT NULL
);

CREATE TABLE IF NOT EXISTS search_history
(
    id        INT AUTO_INCREMENT PRIMARY KEY,
    user_id   INT,
    query     VARCHAR(255),
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE
);

-- Create full-text indexes for efficient text-based search.
ALTER TABLE web_pages
    ADD FULLTEXT INDEX idx_fulltext_search (title, content);
