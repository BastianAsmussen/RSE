USE rse;

CREATE TABLE IF NOT EXISTS sites
(
    id          INT(11)      NOT NULL AUTO_INCREMENT,
    url         VARCHAR(255) NOT NULL,
    is_accurate TINYINT(1)   NOT NULL DEFAULT 1, -- Whether or not the site is accurate.

    PRIMARY KEY (id)
) ENGINE = InnoDB
  DEFAULT CHARSET = utf8;
