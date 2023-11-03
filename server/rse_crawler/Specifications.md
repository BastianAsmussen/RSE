# Specifications

## RSE Crawler
- [ ] The crawler must log all actions it performs, and all errors it encounters.
- [ ] The crawler can crawl a given website, and find all the links on that website.
- [ ] The crawler has 3 queues, first, for the links to be crawled, second, for the links that have been crawled, and third, for the links to be processed.
- [ ] The crawler can work over multiple threads.
- [ ] The crawler can be stopped at any time, and will resume from where it left off by using a WAL (Write Ahead Log).
- [ ] The crawler only exits crawling when the queue of links to be crawled is empty, and the queue of links to be processed is empty.
- [ ] Crawling should be done by looking up the next link in the queue of links to be crawled, and then adding the links found on that page to the queue of links to be crawled.
- [ ] The crawler should have a maximum depth of crawling (e.g. `5`), so that it does not crawl the entire internet, or `-1` for infinite depth.
- [ ] The crawler needs to have an option to respect and comply with the `robots.txt` file of the website.
- [ ] All options should be configurable via environment variables, and have default values.
