-- CREATE TABLE forward_links
-- (
--     from_page_id INT NOT NULL,           -- The ID of the page that contains the link.
--     to_page_id   INT NOT NULL,           -- The ID of the page that the link points to.
--
--     frequency    INT NOT NULL DEFAULT 1, -- The number of times the link appears on the page.
--
--     CHECK (from_page_id <> to_page_id),  -- A page cannot link to itself.
--
--     PRIMARY KEY (from_page_id, to_page_id),
--
--     FOREIGN KEY (from_page_id) REFERENCES pages (id) ON DELETE CASCADE,
--     FOREIGN KEY (to_page_id) REFERENCES pages (id) ON DELETE CASCADE
-- );

CREATE TABLE forward_links
(
    from_page_id INT NOT NULL,           -- The ID of the page that contains the link.

    to_page_url  VARCHAR(8192),          -- The URL of the page that the link points to.
    frequency    INT NOT NULL DEFAULT 1, -- The number of times the link appears on the page.

    PRIMARY KEY (from_page_id, to_page_url),

    FOREIGN KEY (from_page_id) REFERENCES pages (id) ON DELETE CASCADE
);

CREATE OR REPLACE FUNCTION check_self_link() RETURNS TRIGGER AS
$$
BEGIN
    IF EXISTS (SELECT 1
               FROM pages
               WHERE NEW.from_page_id = id
                 AND NEW.to_page_url = url) THEN
        RAISE EXCEPTION 'A page cannot link to itself!';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER enforce_no_self_link
    BEFORE INSERT OR UPDATE
    ON forward_links
    FOR EACH ROW
EXECUTE FUNCTION check_self_link();