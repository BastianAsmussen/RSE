use serde_yaml::Value;

use std::fs::File;
use std::io::Read;
use std::path::Path;

use error::Error;
use log::info;
use serde::Deserialize;

/// 1 MB as bytes.
const MAX_BYTES_TO_READ: u64 = 1_024_000;

/// Reads data from a file.
///
/// # Arguments
///
/// * `file_path`: The path to the file.
/// * `strategy`: The strategy to use to read the data.
///
/// # Returns
///
/// * `Result<Option<Vec<String>>, std::io::Error>` - The data.
fn read_data_from_file<T, F>(
    file_path: T,
    strategy: F,
) -> Result<Option<Vec<String>>, std::io::Error>
where
    T: AsRef<Path>,
    F: Fn(&str) -> Option<Vec<String>>,
{
    let file = File::open(file_path)?;
    let mut content = String::new();
    file.take(MAX_BYTES_TO_READ).read_to_string(&mut content)?;

    Ok(strategy(&content))
}

/// A list of seed URLs.
///
/// # Fields
///
/// * `search_engines`: A list of search engines.
/// * `news_websites`: A list of news websites.
/// * `social_media_platforms`: A list of social media platforms.
/// * `academic_and_research_databases`: A list of academic and research databases.
/// * `e_commerce_websites`: A list of e-commerce websites.
/// * `government_websites`: A list of government websites.
/// * `blogs_and_personal_websites`: A list of blogs and personal websites.
/// * `reference_websites`: A list of reference websites.
/// * `technology_news_and_forums`: A list of technology news and forums.
/// * `educational_institutions`: A list of educational institutions.
/// * `open_data_repositories`: A list of open data repositories.
/// * `video_sharing_platforms`: A list of video sharing platforms.
/// * `forums_and_community_sites`: A list of forums and community sites.
/// * `health_and_medical_websites`: A list of health and medical websites.
/// * `local_and_regional_news`: A list of local and regional news.
/// * `niche_or_specialized_websites`: A list of niche or specialized websites.
#[derive(Deserialize)]
struct SeedURLs {
    search_engines: Option<Vec<String>>,
    news_websites: Option<Vec<String>>,
    social_media_platforms: Option<Vec<String>>,
    academic_and_research_databases: Option<Vec<String>>,
    e_commerce_websites: Option<Vec<String>>,
    government_websites: Option<Vec<String>>,
    blogs_and_personal_websites: Option<Vec<String>>,
    reference_websites: Option<Vec<String>>,
    technology_news_and_forums: Option<Vec<String>>,
    educational_institutions: Option<Vec<String>>,
    open_data_repositories: Option<Vec<String>>,
    video_sharing_platforms: Option<Vec<String>>,
    forums_and_community_sites: Option<Vec<String>>,
    health_and_medical_websites: Option<Vec<String>>,
    local_and_regional_news: Option<Vec<String>>,
    niche_or_specialized_websites: Option<Vec<String>>,
}

/// A list of stop words.
///
/// # Fields
///
/// * `words`: A list of stop words.
#[derive(Deserialize)]
struct StopWords {
    words: Option<Vec<String>>,
}

trait SeedUrlStrategy {
    fn read_seed_urls(&self, content: &str) -> Option<Vec<String>>;
}

trait StopWordsStrategy {
    fn read_stop_words(&self, content: &str) -> Option<Vec<String>>;
}

struct JSONStrategy;
struct YAMLStrategy;
struct TextStrategy;

impl SeedUrlStrategy for JSONStrategy {
    fn read_seed_urls(&self, content: &str) -> Option<Vec<String>> {
        let seed_urls: SeedURLs = serde_json::from_str(content).ok()?;
        let all_urls = vec![
            seed_urls.search_engines,
            seed_urls.news_websites,
            seed_urls.social_media_platforms,
            seed_urls.academic_and_research_databases,
            seed_urls.e_commerce_websites,
            seed_urls.government_websites,
            seed_urls.blogs_and_personal_websites,
            seed_urls.reference_websites,
            seed_urls.technology_news_and_forums,
            seed_urls.educational_institutions,
            seed_urls.open_data_repositories,
            seed_urls.video_sharing_platforms,
            seed_urls.forums_and_community_sites,
            seed_urls.health_and_medical_websites,
            seed_urls.local_and_regional_news,
            seed_urls.niche_or_specialized_websites,
        ];

        let result: Vec<String> = all_urls.into_iter().flatten().flatten().collect();

        Some(result)
    }
}

impl SeedUrlStrategy for YAMLStrategy {
    fn read_seed_urls(&self, content: &str) -> Option<Vec<String>> {
        let seed_urls: Value = serde_yaml::from_str(content).ok()?;
        if let Value::Sequence(urls) = seed_urls {
            let result = urls
                .iter()
                .filter_map(|url| url.as_str())
                .map(std::string::ToString::to_string)
                .collect();

            Some(result)
        } else {
            None
        }
    }
}

impl SeedUrlStrategy for TextStrategy {
    fn read_seed_urls(&self, content: &str) -> Option<Vec<String>> {
        let result = content
            .lines()
            .map(std::string::ToString::to_string)
            .collect();

        Some(result)
    }
}

impl StopWordsStrategy for JSONStrategy {
    fn read_stop_words(&self, content: &str) -> Option<Vec<String>> {
        let stop_words: StopWords = serde_json::from_str(content).ok()?;
        let result = stop_words.words?;

        Some(result)
    }
}

impl StopWordsStrategy for YAMLStrategy {
    fn read_stop_words(&self, content: &str) -> Option<Vec<String>> {
        let stop_words: Value = serde_yaml::from_str(content).ok()?;
        if let Value::Sequence(words) = stop_words {
            let result = words
                .iter()
                .filter_map(|word| word.as_str())
                .map(std::string::ToString::to_string)
                .collect();

            Some(result)
        } else {
            None
        }
    }
}

impl StopWordsStrategy for TextStrategy {
    fn read_stop_words(&self, content: &str) -> Option<Vec<String>> {
        let result = content
            .lines()
            .map(std::string::ToString::to_string)
            .collect();

        Some(result)
    }
}

struct SeedURLReader<'a> {
    strategy: &'a dyn SeedUrlStrategy,
}

impl<'a> SeedURLReader<'a> {
    fn new(strategy: &'a dyn SeedUrlStrategy) -> Self {
        SeedURLReader { strategy }
    }

    fn read_seed_urls_from_file<T>(
        &self,
        file_path: T,
    ) -> Result<Option<Vec<String>>, std::io::Error>
    where
        T: AsRef<Path>,
    {
        read_data_from_file(file_path, |content| self.strategy.read_seed_urls(content))
    }
}

struct StopWordsReader<'a> {
    strategy: &'a dyn StopWordsStrategy,
}

impl<'a> StopWordsReader<'a> {
    fn new(strategy: &'a dyn StopWordsStrategy) -> Self {
        StopWordsReader { strategy }
    }

    fn read_stop_words_from_file<T>(
        &self,
        file_path: T,
    ) -> Result<Option<Vec<String>>, std::io::Error>
    where
        T: AsRef<Path>,
    {
        read_data_from_file(file_path, |content| self.strategy.read_stop_words(content))
    }
}

/// Fetch all the seed URLs from the provided file.
///
/// The file is specified by the `SEED_URLS` environment variable and can be of many file types,
/// but will mostly be denoted as `JSON`.
///
/// # Returns
///
/// * `Result<Vec<String>, Error>` - The seed URLs.
///
/// # Errors
///
/// * If the file extension is invalid.
/// * If the file extension is not supported.
/// * If the file cannot be read.
/// * If the file cannot be parsed.
/// * If the file is not valid UTF-8.
/// * If the file is not a valid JSON, YAML, or text file.
///
/// # Panics
///
/// * If `SEED_URLS` is not set.
/// * If `SEED_URLS` is not valid UTF-8.
#[allow(clippy::expect_used)]
pub fn fetch_seed_urls() -> Result<Vec<String>, Error> {
    // Load the file path from the environment variable.
    let file_path = std::env::var_os("SEED_URLS")
        .expect("SEED_URLS must be set!")
        .to_str()
        .expect("SEED_URLS must be valid UTF-8!")
        .to_string();

    info!("Loading seed URLs from {file_path}...");

    // Define the reader.
    let path = Path::new(&file_path);
    let Some(extension) = path.extension() else {
        return Err(Error::Internal(
            "Invalid file extension, no reader implemented for \"\"!".into(),
        ));
    };
    let reader = match extension.to_str() {
        Some("json") => SeedURLReader::new(&JSONStrategy),
        Some("yaml" | "yml") => SeedURLReader::new(&YAMLStrategy),
        Some("txt") => SeedURLReader::new(&TextStrategy),
        Some(extension) => {
            return Err(Error::Internal(format!(
                "Invalid file extension, no reader implemented for \".{extension}\"!"
            )));
        }
        None => {
            return Err(Error::Internal(
                "Invalid file extension, no reader implemented for \"\"!".into(),
            ))
        }
    };

    // Read the seed URLs from the file.
    (reader.read_seed_urls_from_file(path)?).map_or_else(
        || Err(Error::Internal("Failed to read seed URLs!".into())),
        Ok,
    )
}

/// Fetch all the stop words from the provided file.
///
/// The file is specified by the `STOP_WORDS` environment variable and can be of many file types,
/// but will mostly be denoted as `text`.
///
/// # Returns
///
/// * `Result<Vec<String>, Error>` - The stop words.
///
/// # Errors
///
/// * If the file extension is invalid.
/// * If the file extension is not supported.
/// * If the file cannot be read.
/// * If the file cannot be parsed.
/// * If the file is not valid UTF-8.
/// * If the file is not a valid JSON, YAML, or text file.
///
/// # Panics
///
/// * If `SEED_URLS` is not set.
/// * If `SEED_URLS` is not valid UTF-8.
#[allow(clippy::expect_used)]
pub fn fetch_stop_words() -> Result<Vec<String>, Error> {
    // Load the file path from the environment variable.
    let file_path = std::env::var_os("STOP_WORDS")
        .expect("STOP_WORDS must be set!")
        .to_str()
        .expect("STOP_WORDS must be valid UTF-8!")
        .to_string();

    info!("Loading stop words from {file_path}...");

    // Define the reader.
    let path = Path::new(&file_path);
    let Some(extension) = path.extension() else {
        return Err(Error::Internal(
            "Invalid file extension, no reader implemented for \"\"!".into(),
        ));
    };
    let reader = match extension.to_str() {
        Some("json") => StopWordsReader::new(&JSONStrategy),
        Some("yaml" | "yml") => StopWordsReader::new(&YAMLStrategy),
        Some("txt") => StopWordsReader::new(&TextStrategy),
        Some(extension) => {
            return Err(Error::Internal(format!(
                "Invalid file extension, no reader implemented for \".{extension}\"!"
            )));
        }
        None => {
            return Err(Error::Internal(
                "Invalid file extension, no reader implemented for \"\"!".into(),
            ))
        }
    };

    // Read the stop words from the file.
    (reader.read_stop_words_from_file(path)?).map_or_else(
        || Err(Error::Internal("Failed to read stop words!".into())),
        Ok,
    )
}
