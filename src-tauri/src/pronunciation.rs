use reqwest::{Client, Url};
use serde::Deserialize;
use std::time::Duration;

const DICTIONARY_API_URL: &str = "https://api.dictionaryapi.dev/api/v2/entries/en";

#[derive(Debug)]
pub struct Pronunciation {
    pub phonetic: Option<String>,
    pub parts_of_speech: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DictionaryEntry {
    #[serde(default)]
    phonetic: Option<String>,
    #[serde(default)]
    phonetics: Vec<DictionaryPhonetic>,
    #[serde(default)]
    meanings: Vec<DictionaryMeaning>,
}

#[derive(Debug, Deserialize)]
struct DictionaryPhonetic {
    #[serde(default)]
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DictionaryMeaning {
    #[serde(default, rename = "partOfSpeech")]
    part_of_speech: Option<String>,
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
    let parts_of_speech = preferred_parts_of_speech(&entries);

    Ok(Pronunciation {
        phonetic,
        parts_of_speech,
    })
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

fn preferred_parts_of_speech(entries: &[DictionaryEntry]) -> Vec<String> {
    let mut parts_of_speech = Vec::new();
    for part_of_speech in entries
        .iter()
        .flat_map(|entry| entry.meanings.iter())
        .filter_map(|meaning| meaning.part_of_speech.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if !parts_of_speech
            .iter()
            .any(|existing| existing == part_of_speech)
        {
            parts_of_speech.push(part_of_speech.to_string());
        }
    }
    parts_of_speech
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
            meanings: vec![
                DictionaryMeaning {
                    part_of_speech: Some("noun".to_string()),
                },
                DictionaryMeaning {
                    part_of_speech: Some("verb".to_string()),
                },
                DictionaryMeaning {
                    part_of_speech: Some("noun".to_string()),
                },
            ],
        }];

        assert_eq!(preferred_phonetic(&entries).as_deref(), Some("həˈləʊ"));
        assert_eq!(preferred_parts_of_speech(&entries), ["noun", "verb"]);
    }
}
