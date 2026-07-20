use reqwest::{Client, Url};
use serde::Deserialize;
use std::time::Duration;

const DICTIONARY_API_URL: &str = "https://api.dictionaryapi.dev/api/v2/entries/en";

#[derive(Debug)]
pub struct Pronunciation {
    pub phonetic: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DictionaryEntry {
    #[serde(default)]
    phonetic: Option<String>,
    #[serde(default)]
    phonetics: Vec<DictionaryPhonetic>,
}

#[derive(Debug, Deserialize)]
struct DictionaryPhonetic {
    #[serde(default)]
    text: Option<String>,
}

pub async fn lookup(word: &str) -> Result<Pronunciation, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(8))
        .user_agent("VocabularyBuilder/0.1")
        .build()
        .map_err(|error| format!("Failed to create dictionary client: {error}"))?;
    let endpoint = dictionary_url(word)?;
    let response = client
        .get(endpoint)
        .send()
        .await
        .map_err(|error| format!("Free Dictionary API request failed: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Free Dictionary API returned HTTP {}.",
            response.status()
        ));
    }

    let entries: Vec<DictionaryEntry> = response
        .json()
        .await
        .map_err(|error| format!("Failed to read Free Dictionary API response: {error}"))?;
    let phonetic = preferred_phonetic(&entries);

    Ok(Pronunciation { phonetic })
}

fn dictionary_url(word: &str) -> Result<Url, String> {
    let mut endpoint = Url::parse(DICTIONARY_API_URL)
        .map_err(|error| format!("Invalid dictionary URL: {error}"))?;
    endpoint
        .path_segments_mut()
        .map_err(|_| "Dictionary URL cannot accept a word path.".to_string())?
        .push(word);
    Ok(endpoint)
}

fn preferred_phonetic(entries: &[DictionaryEntry]) -> Option<String> {
    entries
        .iter()
        .filter_map(|entry| entry.phonetic.as_deref())
        .chain(
            entries
                .iter()
                .flat_map(|entry| entry.phonetics.iter())
                .filter_map(|phonetic| phonetic.text.as_deref()),
        )
        .map(str::trim)
        .find(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_ipa_from_dictionary_entries() {
        let entries = vec![DictionaryEntry {
            phonetic: None,
            phonetics: vec![DictionaryPhonetic {
                text: Some("həˈləʊ".to_string()),
            }],
        }];

        assert_eq!(preferred_phonetic(&entries).as_deref(), Some("həˈləʊ"));
    }
}
