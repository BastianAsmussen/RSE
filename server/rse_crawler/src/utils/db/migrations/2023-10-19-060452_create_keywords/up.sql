CREATE TABLE keywords
(
    id        INT AUTO_INCREMENT PRIMARY KEY,
    page_id   INT,
    keyword   VARCHAR(64),
    frequency INT,

    FOREIGN KEY (page_id) REFERENCES pages (id) ON DELETE CASCADE
)