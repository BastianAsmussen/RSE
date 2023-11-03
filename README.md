# RSE (Rust Search Engine)
RSE is a project I'm working on to better understand how search engines actually work under the hood.
It's written entirely in Rust.

## Table of Contents
* [Usage](#usage)
  * [Setup](#setup)
  * [Environment Variables](#environment-variables)
  * [API](#api)
  * [Examples](#examples)
* [Entity Relationship Diagram](#entity-relationship-diagram)
* [License](#license)

## Usage
RSE can run on any system with Docker enabled.

### Setup
1. Clone the repository.
2. Enter the `server` directory.
3. Run the `setup.sh` script to set up the database.
4. Now you can start RSE by running `docker compose up --build`. (You can also run it in the background by adding the `-d` flag.)

### Environment Variables
RSE uses environment variables to configure the server. You can set them in a `.env` file in root directory.

| Variable                 | Description                                      | Default                                  |
|--------------------------|--------------------------------------------------|------------------------------------------|
| `DATABASE_URL`           | The URL of the database.                         | `postgres://root:toor@1.0.0.2:5432/rse"` |
| `SEED_URLS`              | The seed URLs to crawl.                          | None                                     |
| `STOP_WORDS`             | The stop words to use.                           | None                                     |
| `CRAWLING_WORKERS`       | The number of websites to crawl at once.         | `1`                                      |
| `PROCESSING_WORKERS`     | The number of websites to process at once.       | `1`                                      |
| `CRAWL_DELAY`            | The delay between crawling websites (in seconds) | `1`                                      |
| `MAXIMUM_DEPTH`          | The maximum depth to crawl.                      | Infinity                                 |
| `MINIMUM_WORD_FREQUENCY` | The minimum frequency of a word to be indexed.   | `1`                                      |
| `MAXIMUM_WORD_FREQUENCY` | The maximum frequency of a word to be indexed.   | `1024`                                   |
| `MINIMUM_WORD_LENGTH`    | The minimum length of a word to be indexed.      | `2`                                      |
| `MAXIMUM_WORD_LENGTH`    | The maximum length of a word to be indexed.      | `128`                                    |
| `USER_AGENT`             | The user agent to use for HTTP requests.         | `RSE/1.0.0`                              |
| `HTTP_TIMEOUT`           | The timeout for HTTP requests (in seconds).      | `10`                                     |
| `LISTEN_ADDRESS`         | The address the web server will listen on.       | `0.0.0.0:8080`                           |

### API
RSE exposes a simple API to search web. It's available at `http://localhost:8080/?q=<query>` by default.

### Examples
* `http://localhost:8080/?q=hello+world`
* [Environment](.env)
* [Seed URLs](server/rse_crawler/seed_urls.json)
* [Stop Words](server/rse_crawler/stop_words.json)

## Entity Relationship Diagram
![Entity Relationship Diagram](server/img/erd.png)

## License
RSE is licensed under the [MIT License](LICENSE).
