CREATE TABLE forward_links
(
    id      INT AUTO_INCREMENT PRIMARY KEY,
    page_id INT,
    url     VARCHAR(255),

    FOREIGN KEY (page_id) REFERENCES pages (id) ON DELETE CASCADE
)