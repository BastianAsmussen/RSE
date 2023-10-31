CREATE TABLE forward_links
(
    from_page_id INT NOT NULL,           -- The ID of the page that contains the link.
    to_page_id   INT NOT NULL,           -- The ID of the page that the link points to.

    frequency    INT NOT NULL DEFAULT 1, -- The number of times the link appears on the page.

    -- CHECK (from_page_id <> to_page_id),  -- A page cannot link to itself.

    PRIMARY KEY (from_page_id, to_page_id),

    FOREIGN KEY (from_page_id) REFERENCES pages (id) ON DELETE CASCADE,
    FOREIGN KEY (to_page_id) REFERENCES pages (id) ON DELETE CASCADE
);