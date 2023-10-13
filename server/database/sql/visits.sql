USE rse;

CREATE TABLE IF NOT EXISTS visits
(
    id          INT(11) NOT NULL AUTO_INCREMENT,
    site_id     INT(11) NOT NULL,

    bounce_rate INT(11) NOT NULL, -- How fast the user left the site (higher is better).

    PRIMARY KEY (id),
    FOREIGN KEY (site_id) REFERENCES sites (id)
) ENGINE = InnoDB
  DEFAULT CHARSET = utf8;

CREATE INDEX site_id
    ON visits (site_id);
