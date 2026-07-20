use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use url::Url;
use uuid::Uuid;

pub const WORDS_PER_PAGE: u32 = 10;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WordStatus {
    Unfamiliar,
    Known,
    Familiar,
}

impl WordStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Unfamiliar => "unfamiliar",
            Self::Known => "known",
            Self::Familiar => "familiar",
        }
    }

    fn from_str(value: &str) -> rusqlite::Result<Self> {
        match value {
            "unfamiliar" => Ok(Self::Unfamiliar),
            "known" => Ok(Self::Known),
            "familiar" => Ok(Self::Familiar),
            _ => Err(rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                format!("Unknown word status: {value}").into(),
            )),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyWord {
    pub id: String,
    pub word: String,
    pub url: String,
    pub status: WordStatus,
    pub phonetic: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordInput {
    pub word: String,
    pub url: String,
    pub status: WordStatus,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordListRequest {
    pub status: Option<WordStatus>,
    pub page: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WordPage {
    pub words: Vec<VocabularyWord>,
    pub total: u32,
}

pub struct VocabularyRepository {
    connection: Connection,
}

impl VocabularyRepository {
    pub fn open(path: &Path) -> Result<Self, String> {
        let connection = Connection::open(path).map_err(|error| error.to_string())?;
        let repository = Self { connection };
        repository.initialize()?;
        Ok(repository)
    }

    #[cfg(test)]
    fn in_memory() -> Self {
        let repository = Self {
            connection: Connection::open_in_memory().expect("in-memory database should open"),
        };
        repository.initialize().expect("schema should initialize");
        repository
    }

    fn initialize(&self) -> Result<(), String> {
        self.connection
            .execute_batch(
                "
                CREATE TABLE IF NOT EXISTS words (
                    id TEXT PRIMARY KEY NOT NULL,
                    word TEXT NOT NULL,
                    url TEXT NOT NULL,
                    status TEXT NOT NULL CHECK(status IN ('unfamiliar', 'known', 'familiar')),
                    phonetic TEXT,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_words_status_updated_at
                    ON words(status, updated_at DESC);
                ",
            )
            .map_err(|error| error.to_string())?;
        self.ensure_column("phonetic", "TEXT")?;
        Ok(())
    }

    pub fn list(&self, request: WordListRequest) -> Result<WordPage, String> {
        if request.page == 0 {
            return Err("Page number must start at 1.".to_string());
        }

        let total = self.count(request.status.as_ref())?;
        let offset = (u64::from(request.page) - 1)
            .checked_mul(u64::from(WORDS_PER_PAGE))
            .ok_or_else(|| "Page number is out of range.".to_string())?;
        let offset =
            i64::try_from(offset).map_err(|_| "Page number is out of range.".to_string())?;
        let mut words = Vec::new();
        if let Some(status) = request.status {
            let mut statement = self
                .connection
                .prepare(
                    "SELECT id, word, url, status, phonetic, created_at, updated_at
                     FROM words WHERE status = ?1
                     ORDER BY updated_at DESC, word COLLATE NOCASE LIMIT ?2 OFFSET ?3",
                )
                .map_err(|error| error.to_string())?;
            let rows = statement
                .query_map(
                    params![status.as_str(), WORDS_PER_PAGE, offset],
                    row_to_word,
                )
                .map_err(|error| error.to_string())?;
            for row in rows {
                words.push(row.map_err(|error| error.to_string())?);
            }
        } else {
            let mut statement = self
                .connection
                .prepare(
                    "SELECT id, word, url, status, phonetic, created_at, updated_at
                     FROM words ORDER BY updated_at DESC, word COLLATE NOCASE
                     LIMIT ?1 OFFSET ?2",
                )
                .map_err(|error| error.to_string())?;
            let rows = statement
                .query_map(params![WORDS_PER_PAGE, offset], row_to_word)
                .map_err(|error| error.to_string())?;
            for row in rows {
                words.push(row.map_err(|error| error.to_string())?);
            }
        }
        Ok(WordPage { words, total })
    }

    pub fn create(&self, input: WordInput) -> Result<VocabularyWord, String> {
        let (word, url) = validate_input(input.word, input.url)?;
        let now = timestamp()?;
        let record = VocabularyWord {
            id: Uuid::new_v4().to_string(),
            word,
            url,
            status: input.status,
            phonetic: None,
            created_at: now,
            updated_at: now,
        };
        self.connection
            .execute(
                "INSERT INTO words (id, word, url, status, phonetic, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    record.id,
                    record.word,
                    record.url,
                    record.status.as_str(),
                    record.phonetic,
                    record.created_at,
                    record.updated_at
                ],
            )
            .map_err(|error| error.to_string())?;
        Ok(record)
    }

    pub fn update(&self, id: &str, input: WordInput) -> Result<VocabularyWord, String> {
        let (word, url) = validate_input(input.word, input.url)?;
        let updated_at = timestamp()?;
        let changed = self
            .connection
            .execute(
                "UPDATE words SET word = ?1, url = ?2, status = ?3, updated_at = ?4 WHERE id = ?5",
                params![word, url, input.status.as_str(), updated_at, id],
            )
            .map_err(|error| error.to_string())?;
        if changed == 0 {
            return Err("Word to edit was not found.".to_string());
        }
        self.find(id)?
            .ok_or_else(|| "Updated word was not found.".to_string())
    }

    pub fn save_pronunciation(
        &self,
        id: &str,
        phonetic: Option<String>,
    ) -> Result<VocabularyWord, String> {
        let updated_at = timestamp()?;
        let changed = self
            .connection
            .execute(
                "UPDATE words SET phonetic = ?1, updated_at = ?2 WHERE id = ?3",
                params![phonetic, updated_at, id],
            )
            .map_err(|error| error.to_string())?;
        if changed == 0 {
            return Err("Word for pronunciation was not found.".to_string());
        }
        self.find(id)?
            .ok_or_else(|| "Saved pronunciation was not found.".to_string())
    }

    pub fn delete(&self, id: &str) -> Result<(), String> {
        let changed = self
            .connection
            .execute("DELETE FROM words WHERE id = ?1", [id])
            .map_err(|error| error.to_string())?;
        if changed == 0 {
            return Err("Word to delete was not found.".to_string());
        }
        Ok(())
    }

    fn find(&self, id: &str) -> Result<Option<VocabularyWord>, String> {
        self.connection
            .query_row(
                "SELECT id, word, url, status, phonetic, created_at, updated_at FROM words WHERE id = ?1",
                [id],
                row_to_word,
            )
            .optional()
            .map_err(|error| error.to_string())
    }

    fn count(&self, status: Option<&WordStatus>) -> Result<u32, String> {
        let total: i64 = if let Some(status) = status {
            self.connection.query_row(
                "SELECT COUNT(*) FROM words WHERE status = ?1",
                [status.as_str()],
                |row| row.get(0),
            )
        } else {
            self.connection
                .query_row("SELECT COUNT(*) FROM words", [], |row| row.get(0))
        }
        .map_err(|error| error.to_string())?;
        u32::try_from(total).map_err(|_| "Word count exceeds the supported range.".to_string())
    }

    fn ensure_column(&self, column: &str, definition: &str) -> Result<(), String> {
        if self.column_exists(column)? {
            return Ok(());
        }
        self.connection
            .execute(
                &format!("ALTER TABLE words ADD COLUMN {column} {definition}"),
                [],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn column_exists(&self, column: &str) -> Result<bool, String> {
        let mut statement = self
            .connection
            .prepare("PRAGMA table_info(words)")
            .map_err(|error| error.to_string())?;
        let columns = statement
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        Ok(columns.iter().any(|existing| existing == column))
    }
}

fn row_to_word(row: &rusqlite::Row<'_>) -> rusqlite::Result<VocabularyWord> {
    let status: String = row.get(3)?;
    Ok(VocabularyWord {
        id: row.get(0)?,
        word: row.get(1)?,
        url: row.get(2)?,
        status: WordStatus::from_str(&status)?,
        phonetic: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn validate_input(word: String, url: String) -> Result<(String, String), String> {
    let word = word.trim().to_string();
    if word.is_empty() {
        return Err("A word is required.".to_string());
    }
    if word.chars().count() > 120 {
        return Err("A word cannot exceed 120 characters.".to_string());
    }

    let url = url.trim().to_string();
    let parsed = Url::parse(&url).map_err(|_| "A valid source URL is required.".to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        return Err("Source URL must start with http:// or https://.".to_string());
    }
    Ok((word, url))
}

fn timestamp() -> Result<i64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .map_err(|_| "System time is before the Unix epoch.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(word: &str, status: WordStatus) -> WordInput {
        WordInput {
            word: word.to_string(),
            url: "https://dictionary.example.com/word".to_string(),
            status,
        }
    }

    #[test]
    fn creates_filters_updates_and_deletes_words() {
        let repository = VocabularyRepository::in_memory();
        let created = repository
            .create(input("ephemeral", WordStatus::Unfamiliar))
            .unwrap();
        repository
            .save_pronunciation(&created.id, Some("ɪˈfemərəl".to_string()))
            .unwrap();
        assert_eq!(
            repository
                .list(WordListRequest {
                    status: Some(WordStatus::Unfamiliar),
                    page: 1,
                })
                .unwrap()
                .total,
            1
        );

        let updated = repository
            .update(&created.id, input("ephemeral", WordStatus::Familiar))
            .unwrap();
        assert_eq!(updated.status, WordStatus::Familiar);
        assert_eq!(updated.phonetic.as_deref(), Some("ɪˈfemərəl"));
        assert_eq!(
            repository
                .list(WordListRequest {
                    status: Some(WordStatus::Unfamiliar),
                    page: 1,
                })
                .unwrap()
                .total,
            0
        );
        assert_eq!(
            repository
                .list(WordListRequest {
                    status: Some(WordStatus::Familiar),
                    page: 1,
                })
                .unwrap()
                .total,
            1
        );

        repository.delete(&created.id).unwrap();
        assert_eq!(
            repository
                .list(WordListRequest {
                    status: None,
                    page: 1,
                })
                .unwrap()
                .total,
            0
        );
    }

    #[test]
    fn rejects_invalid_input() {
        assert!(validate_input(" ".to_string(), "https://example.com".to_string()).is_err());
        assert!(validate_input("word".to_string(), "file:///local".to_string()).is_err());
    }

    #[test]
    fn paginates_words_and_filters_by_status() {
        let repository = VocabularyRepository::in_memory();
        let empty_page = repository
            .list(WordListRequest {
                status: None,
                page: 1,
            })
            .unwrap();
        assert_eq!(empty_page.total, 0);
        assert!(empty_page.words.is_empty());

        repository
            .create(input("word-00", WordStatus::Known))
            .unwrap();
        let one_word_page = repository
            .list(WordListRequest {
                status: None,
                page: 1,
            })
            .unwrap();
        assert_eq!(one_word_page.total, 1);
        assert_eq!(one_word_page.words.len(), 1);

        for index in 1..10 {
            let status = if index < 3 {
                WordStatus::Known
            } else {
                WordStatus::Unfamiliar
            };
            repository
                .create(input(&format!("word-{index:02}"), status))
                .unwrap();
        }

        let ten_word_page = repository
            .list(WordListRequest {
                status: None,
                page: 1,
            })
            .unwrap();
        assert_eq!(ten_word_page.total, WORDS_PER_PAGE);
        assert_eq!(ten_word_page.words.len(), WORDS_PER_PAGE as usize);

        repository
            .create(input("word-10", WordStatus::Unfamiliar))
            .unwrap();
        let first_page = repository
            .list(WordListRequest {
                status: None,
                page: 1,
            })
            .unwrap();
        assert_eq!(first_page.total, 11);
        assert_eq!(first_page.words.len(), WORDS_PER_PAGE as usize);

        let second_page = repository
            .list(WordListRequest {
                status: None,
                page: 2,
            })
            .unwrap();
        assert_eq!(second_page.total, 11);
        assert_eq!(second_page.words.len(), 1);

        let known_words = repository
            .list(WordListRequest {
                status: Some(WordStatus::Known),
                page: 1,
            })
            .unwrap();
        assert_eq!(known_words.total, 3);
        assert!(known_words
            .words
            .iter()
            .all(|word| word.status == WordStatus::Known));
    }

    #[test]
    fn stores_phonetic() {
        let repository = VocabularyRepository::in_memory();
        let created = repository
            .create(input("hello", WordStatus::Unfamiliar))
            .unwrap();

        let enriched = repository
            .save_pronunciation(&created.id, Some("həˈləʊ".to_string()))
            .unwrap();

        assert_eq!(enriched.phonetic.as_deref(), Some("həˈləʊ"));
    }
}
