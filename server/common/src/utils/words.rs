use log::debug;
use regex::Regex;
use std::collections::HashMap;

/// Get words from content.
///
/// # Arguments
///
/// * `content` - The content to get words from.
/// * `language` - The language to stem the words in.
///
/// # Returns
///
/// * `HashMap<String, usize>` - The words and their frequencies.
///
/// # Panics
///
/// * If the illegal characters regex fails to compile.
#[allow(clippy::expect_used)]
pub fn extract(content: &str, language: rust_stemmers::Algorithm) -> HashMap<String, usize> {
    let raw_words = content
        .to_lowercase()
        .split_whitespace()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>();
    let mut extracted_words = HashMap::new();

    /*
     If a word isn't in this regex, it's illegal.
     Illegal characters are removed from the word.
     This is to prevent words like "hello!" and "world?" from being counted as different words.
    */
    let illegal_characters = Regex::new(r"[^a-zA-Z0-9\u{00C0}-\u{00FF}]+")
        .expect("Failed to compile illegal characters regex!");

    for word in raw_words {
        // Make sure the word doesn't contain illegal characters.
        let word = illegal_characters.replace_all(&word, "");
        if word.is_empty() {
            continue;
        }

        let frequency = extracted_words.entry(word.to_string()).or_insert(0);
        *frequency += 1;
    }

    stem(extracted_words, language)
}

/// Stem words.
///
/// # Arguments
///
/// * `words` - The words to stem.
/// * `language` - The language to stem the words in.
///
/// # Returns
///
/// * `HashMap<String, usize>` - The stemmed words and their frequencies.
fn stem(
    words: HashMap<String, usize>,
    language: rust_stemmers::Algorithm,
) -> HashMap<String, usize> {
    let stemmer = rust_stemmers::Stemmer::create(language);

    let mut stemmed_words = HashMap::new();
    for (word, frequency) in words {
        let stemmed_word = stemmer.stem(&word);

        if word != stemmed_word {
            debug!("Stemmed word: {word} -> {stemmed_word}");
        }

        let count = stemmed_words.entry(stemmed_word.to_string()).or_insert(0);
        *count += frequency;
    }

    stemmed_words
}
