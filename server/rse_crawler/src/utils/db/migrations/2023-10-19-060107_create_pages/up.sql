CREATE TABLE pages
(
    id          INT AUTO_INCREMENT PRIMARY KEY,

    url         VARCHAR(255) NOT NULL UNIQUE,

    title       VARCHAR(255) DEFAULT NULL,
    description VARCHAR(255) DEFAULT NULL
)