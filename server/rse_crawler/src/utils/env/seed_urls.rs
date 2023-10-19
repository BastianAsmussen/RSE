use serde::Deserialize;
use std::error::Error;

use serde_yaml::Value;

use std::fs::File;
use std::io::Read;
use std::path::Path;

use log::{error, info};

/// 1 MB as bytes.
const MAX_BYTES_TO_READ: u64 = 1_024_000;

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

// Define a trait for different file format strategies
trait SeedURLStrategy {
    fn read_seed_urls(&self, content: &str) -> Option<Vec<String>>;
}

struct JSONStrategy;

impl SeedURLStrategy for JSONStrategy {
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

struct YAMLStrategy;

impl SeedURLStrategy for YAMLStrategy {
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

// Context that uses the strategy
struct SeedURLReader<'a> {
    strategy: &'a dyn SeedURLStrategy,
}

impl<'a> SeedURLReader<'a> {
    fn new(strategy: &'a dyn SeedURLStrategy) -> Self {
        SeedURLReader { strategy }
    }

    fn read_seed_urls_from_file<T>(
        &self,
        file_path: T,
    ) -> Result<Option<Vec<String>>, Box<dyn Error>>
    where
        T: AsRef<Path>,
    {
        let file = File::open(file_path)?;
        let mut content = String::new();
        file.take(MAX_BYTES_TO_READ).read_to_string(&mut content)?;

        Ok(self.strategy.read_seed_urls(&content))
    }
}

/// Fetch all the seed URLs from the provided file.
///
/// The file is specified by the `SEED_URLS` environment variable and can be of many file types,
/// but will mostly be denoted as `JSON`.
///
/// # Returns
///
/// * An `Option` containing either `Some(Vec<String>)` or `None`.
pub fn fetch() -> Option<Vec<String>> {
    // Load the file path from the environment variable.
    let Ok(file_path) = std::env::var_os("SEED_URLS")?.into_string() else {
        error!("Failed to parse seed URLs path to string!");

        return None;
    };
    info!("Loading seed URLs from {}...", file_path);

    // Define the reader.
    let path = Path::new(&file_path);
    let extension = path.extension()?;
    let reader = match extension.to_str() {
        Some("json") => {
            info!("Using JSON strategy for seed URL reader...");

            SeedURLReader::new(&JSONStrategy)
        }
        Some("yaml" | "yml") => {
            info!("Using YAML strategy for seed URL reader...");

            SeedURLReader::new(&YAMLStrategy)
        }
        _ => {
            error!("Invalid file extension, no reader implemented!");

            return None;
        }
    };

    match reader.read_seed_urls_from_file(path) {
        Ok(seed_urls) => {
            let Some(urls) = seed_urls else {
                error!("Failed to read seed URLs, empty data!");

                return None;
            };

            Some(urls)
        }
        Err(err) => {
            error!("Failed to read seed URLs!");
            error!("Error: {err}");

            None
        }
    }
}
