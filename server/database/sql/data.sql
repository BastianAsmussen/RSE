USE rse;

CREATE TABLE IF NOT EXISTS data
(
    id           INT(11)  NOT NULL AUTO_INCREMENT,
    site_id      INT(11)  NOT NULL UNIQUE,

    last_updated DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP, -- When the site was last updated.
    html         BLOB     NOT NULL,                           -- The HTML contents of the site.

    PRIMARY KEY (id),
    FOREIGN KEY (site_id) REFERENCES sites (id)
) ENGINE = InnoDB
  DEFAULT CHARSET = utf8;
