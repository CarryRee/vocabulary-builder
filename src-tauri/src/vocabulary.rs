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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WordSort {
    #[default]
    UpdatedAtDesc,
    UpdatedAtAsc,
    WordAsc,
}

impl WordSort {
    fn order_by(&self) -> &'static str {
        match self {
            Self::UpdatedAtDesc => "updated_at DESC, word COLLATE NOCASE ASC, id ASC",
            Self::UpdatedAtAsc => "updated_at ASC, word COLLATE NOCASE ASC, id ASC",
            Self::WordAsc => "word COLLATE NOCASE ASC, updated_at DESC, id ASC",
        }
    }
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
    pub parts_of_speech: Vec<String>,
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
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub sort: WordSort,
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
                    parts_of_speech TEXT NOT NULL DEFAULT '[]',
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_words_status_updated_at
                    ON words(status, updated_at DESC);
                ",
            )
            .map_err(|error| error.to_string())?;
        self.ensure_column("phonetic", "TEXT")?;
        self.ensure_column("parts_of_speech", "TEXT NOT NULL DEFAULT '[]'")?;
        Ok(())
    }

    pub fn list(&self, request: WordListRequest) -> Result<WordPage, String> {
        if request.page == 0 {
            return Err("Page number must start at 1.".to_string());
        }

        let query = normalize_search_query(request.query);
        let search_pattern = query.as_deref().map(search_pattern);
        let total = self.count(request.status.as_ref(), search_pattern.as_deref())?;
        let order_by = request.sort.order_by();
        let offset = (u64::from(request.page) - 1)
            .checked_mul(u64::from(WORDS_PER_PAGE))
            .ok_or_else(|| "Page number is out of range.".to_string())?;
        let offset =
            i64::try_from(offset).map_err(|_| "Page number is out of range.".to_string())?;
        let mut words = Vec::new();
        if let Some(status) = request.status {
            if let Some(search_pattern) = search_pattern.as_deref() {
                let statement_sql = format!(
                    "SELECT id, word, url, status, phonetic, parts_of_speech, created_at, updated_at
                     FROM words
                     WHERE status = ?1 AND word COLLATE NOCASE LIKE ?2 ESCAPE '\\'
                     ORDER BY {order_by} LIMIT ?3 OFFSET ?4"
                );
                let mut statement = self
                    .connection
                    .prepare(&statement_sql)
                    .map_err(|error| error.to_string())?;
                let rows = statement
                    .query_map(
                        params![status.as_str(), search_pattern, WORDS_PER_PAGE, offset],
                        row_to_word,
                    )
                    .map_err(|error| error.to_string())?;
                for row in rows {
                    words.push(row.map_err(|error| error.to_string())?);
                }
            } else {
                let statement_sql = format!(
                    "SELECT id, word, url, status, phonetic, parts_of_speech, created_at, updated_at
                     FROM words WHERE status = ?1
                     ORDER BY {order_by} LIMIT ?2 OFFSET ?3"
                );
                let mut statement = self
                    .connection
                    .prepare(&statement_sql)
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
            }
        } else if let Some(search_pattern) = search_pattern.as_deref() {
            let statement_sql = format!(
                "SELECT id, word, url, status, phonetic, parts_of_speech, created_at, updated_at
                 FROM words
                 WHERE word COLLATE NOCASE LIKE ?1 ESCAPE '\\'
                 ORDER BY {order_by} LIMIT ?2 OFFSET ?3"
            );
            let mut statement = self
                .connection
                .prepare(&statement_sql)
                .map_err(|error| error.to_string())?;
            let rows = statement
                .query_map(params![search_pattern, WORDS_PER_PAGE, offset], row_to_word)
                .map_err(|error| error.to_string())?;
            for row in rows {
                words.push(row.map_err(|error| error.to_string())?);
            }
        } else {
            let statement_sql = format!(
                "SELECT id, word, url, status, phonetic, parts_of_speech, created_at, updated_at
                 FROM words ORDER BY {order_by}
                 LIMIT ?1 OFFSET ?2"
            );
            let mut statement = self
                .connection
                .prepare(&statement_sql)
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
            parts_of_speech: Vec::new(),
            created_at: now,
            updated_at: now,
        };
        self.connection
            .execute(
                "INSERT INTO words (id, word, url, status, phonetic, parts_of_speech, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    record.id,
                    record.word,
                    record.url,
                    record.status.as_str(),
                    record.phonetic,
                    "[]",
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

    pub fn save_dictionary_data(
        &self,
        id: &str,
        phonetic: Option<String>,
        parts_of_speech: Vec<String>,
    ) -> Result<VocabularyWord, String> {
        let parts_of_speech = normalize_parts_of_speech(parts_of_speech);
        let parts_of_speech = serde_json::to_string(&parts_of_speech)
            .map_err(|error| format!("Failed to serialize parts of speech: {error}"))?;
        let updated_at = timestamp()?;
        let changed = self
            .connection
            .execute(
                "UPDATE words SET phonetic = ?1, parts_of_speech = ?2, updated_at = ?3 WHERE id = ?4",
                params![phonetic, parts_of_speech, updated_at, id],
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
                "SELECT id, word, url, status, phonetic, parts_of_speech, created_at, updated_at FROM words WHERE id = ?1",
                [id],
                row_to_word,
            )
            .optional()
            .map_err(|error| error.to_string())
    }

    fn count(
        &self,
        status: Option<&WordStatus>,
        search_pattern: Option<&str>,
    ) -> Result<u32, String> {
        let total: i64 = match (status, search_pattern) {
            (Some(status), Some(search_pattern)) => self.connection.query_row(
                "SELECT COUNT(*) FROM words
                 WHERE status = ?1 AND word COLLATE NOCASE LIKE ?2 ESCAPE '\\'",
                params![status.as_str(), search_pattern],
                |row| row.get(0),
            ),
            (Some(status), None) => self.connection.query_row(
                "SELECT COUNT(*) FROM words WHERE status = ?1",
                [status.as_str()],
                |row| row.get(0),
            ),
            (None, Some(search_pattern)) => self.connection.query_row(
                "SELECT COUNT(*) FROM words WHERE word COLLATE NOCASE LIKE ?1 ESCAPE '\\'",
                [search_pattern],
                |row| row.get(0),
            ),
            (None, None) => self
                .connection
                .query_row("SELECT COUNT(*) FROM words", [], |row| row.get(0)),
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

fn normalize_search_query(query: Option<String>) -> Option<String> {
    query.and_then(|value| {
        let value = value.trim();
        (!value.is_empty()).then(|| value.to_string())
    })
}

fn search_pattern(query: &str) -> String {
    let escaped = query
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}

fn row_to_word(row: &rusqlite::Row<'_>) -> rusqlite::Result<VocabularyWord> {
    let status: String = row.get(3)?;
    let parts_of_speech: String = row.get(5)?;
    let parts_of_speech = serde_json::from_str(&parts_of_speech).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            5,
            rusqlite::types::Type::Text,
            format!("Invalid parts of speech JSON: {error}").into(),
        )
    })?;
    Ok(VocabularyWord {
        id: row.get(0)?,
        word: row.get(1)?,
        url: row.get(2)?,
        status: WordStatus::from_str(&status)?,
        phonetic: row.get(4)?,
        parts_of_speech,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn normalize_parts_of_speech(parts_of_speech: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for part_of_speech in parts_of_speech {
        let part_of_speech = part_of_speech.trim();
        if !part_of_speech.is_empty()
            && !normalized
                .iter()
                .any(|existing: &String| existing == part_of_speech)
        {
            normalized.push(part_of_speech.to_string());
        }
    }
    normalized
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

    fn list_request(status: Option<WordStatus>, query: Option<&str>, page: u32) -> WordListRequest {
        WordListRequest {
            status,
            query: query.map(ToString::to_string),
            sort: WordSort::default(),
            page,
        }
    }

    #[test]
    fn creates_filters_updates_and_deletes_words() {
        let repository = VocabularyRepository::in_memory();
        let created = repository
            .create(input("ephemeral", WordStatus::Unfamiliar))
            .unwrap();
        repository
            .save_dictionary_data(
                &created.id,
                Some("ɪˈfemərəl".to_string()),
                vec!["adjective".to_string()],
            )
            .unwrap();
        assert_eq!(
            repository
                .list(list_request(Some(WordStatus::Unfamiliar), None, 1))
                .unwrap()
                .total,
            1
        );

        let updated = repository
            .update(&created.id, input("ephemeral", WordStatus::Familiar))
            .unwrap();
        assert_eq!(updated.status, WordStatus::Familiar);
        assert_eq!(updated.phonetic.as_deref(), Some("ɪˈfemərəl"));
        assert_eq!(updated.parts_of_speech, vec!["adjective".to_string()]);
        assert_eq!(
            repository
                .list(list_request(Some(WordStatus::Unfamiliar), None, 1))
                .unwrap()
                .total,
            0
        );
        assert_eq!(
            repository
                .list(list_request(Some(WordStatus::Familiar), None, 1))
                .unwrap()
                .total,
            1
        );

        repository.delete(&created.id).unwrap();
        assert_eq!(
            repository.list(list_request(None, None, 1)).unwrap().total,
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
        let empty_page = repository.list(list_request(None, None, 1)).unwrap();
        assert_eq!(empty_page.total, 0);
        assert!(empty_page.words.is_empty());

        repository
            .create(input("word-00", WordStatus::Known))
            .unwrap();
        let one_word_page = repository.list(list_request(None, None, 1)).unwrap();
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

        let ten_word_page = repository.list(list_request(None, None, 1)).unwrap();
        assert_eq!(ten_word_page.total, WORDS_PER_PAGE);
        assert_eq!(ten_word_page.words.len(), WORDS_PER_PAGE as usize);

        repository
            .create(input("word-10", WordStatus::Unfamiliar))
            .unwrap();
        let first_page = repository.list(list_request(None, None, 1)).unwrap();
        assert_eq!(first_page.total, 11);
        assert_eq!(first_page.words.len(), WORDS_PER_PAGE as usize);

        let second_page = repository.list(list_request(None, None, 2)).unwrap();
        assert_eq!(second_page.total, 11);
        assert_eq!(second_page.words.len(), 1);

        let known_words = repository
            .list(list_request(Some(WordStatus::Known), None, 1))
            .unwrap();
        assert_eq!(known_words.total, 3);
        assert!(known_words
            .words
            .iter()
            .all(|word| word.status == WordStatus::Known));
    }

    #[test]
    fn searches_words_across_pages_and_statuses() {
        let repository = VocabularyRepository::in_memory();
        repository
            .create(input("Ephemeral", WordStatus::Unfamiliar))
            .unwrap();
        repository
            .create(input("ephemeris", WordStatus::Familiar))
            .unwrap();
        repository
            .create(input("transport", WordStatus::Known))
            .unwrap();
        repository
            .create(input("100%_ready", WordStatus::Known))
            .unwrap();

        let partial_match = repository
            .list(list_request(None, Some("  PHEM  "), 1))
            .unwrap();
        assert_eq!(partial_match.total, 2);
        assert_eq!(partial_match.words.len(), 2);

        let filtered_match = repository
            .list(list_request(Some(WordStatus::Unfamiliar), Some("ephem"), 1))
            .unwrap();
        assert_eq!(filtered_match.total, 1);
        assert_eq!(filtered_match.words[0].word, "Ephemeral");

        let literal_wildcards = repository.list(list_request(None, Some("%_"), 1)).unwrap();
        assert_eq!(literal_wildcards.total, 1);
        assert_eq!(literal_wildcards.words[0].word, "100%_ready");

        for index in 0..11 {
            repository
                .create(input(
                    &format!("search-result-{index:02}"),
                    WordStatus::Known,
                ))
                .unwrap();
        }
        let first_page = repository
            .list(list_request(None, Some("search-result"), 1))
            .unwrap();
        assert_eq!(first_page.total, 11);
        assert_eq!(first_page.words.len(), WORDS_PER_PAGE as usize);

        let second_page = repository
            .list(list_request(None, Some("search-result"), 2))
            .unwrap();
        assert_eq!(second_page.words.len(), 1);

        let blank_query = repository.list(list_request(None, Some("   "), 1)).unwrap();
        assert_eq!(blank_query.total, 15);
    }

    #[test]
    fn sorts_words_by_update_time_or_word() {
        let repository = VocabularyRepository::in_memory();
        let alpha = repository
            .create(input("alpha", WordStatus::Known))
            .unwrap();
        let beta = repository.create(input("beta", WordStatus::Known)).unwrap();
        let gamma = repository
            .create(input("gamma", WordStatus::Known))
            .unwrap();

        for (id, updated_at) in [(&alpha.id, 10_i64), (&beta.id, 30), (&gamma.id, 20)] {
            repository
                .connection
                .execute(
                    "UPDATE words SET updated_at = ?1 WHERE id = ?2",
                    params![updated_at, id],
                )
                .unwrap();
        }

        let newest_first = repository.list(list_request(None, None, 1)).unwrap().words;
        assert_eq!(newest_first[0].word, "beta");

        let mut oldest_first = list_request(None, None, 1);
        oldest_first.sort = WordSort::UpdatedAtAsc;
        assert_eq!(
            repository.list(oldest_first).unwrap().words[0].word,
            "alpha"
        );

        let mut alphabetical = list_request(None, None, 1);
        alphabetical.sort = WordSort::WordAsc;
        let words = repository.list(alphabetical).unwrap().words;
        assert_eq!(
            words.into_iter().map(|word| word.word).collect::<Vec<_>>(),
            vec!["alpha", "beta", "gamma"]
        );
    }

    #[test]
    fn stores_dictionary_data() {
        let repository = VocabularyRepository::in_memory();
        let created = repository
            .create(input("hello", WordStatus::Unfamiliar))
            .unwrap();

        let enriched = repository
            .save_dictionary_data(
                &created.id,
                Some("həˈləʊ".to_string()),
                vec!["noun".to_string(), "verb".to_string(), "noun".to_string()],
            )
            .unwrap();

        assert_eq!(enriched.phonetic.as_deref(), Some("həˈləʊ"));
        assert_eq!(
            enriched.parts_of_speech,
            vec!["noun".to_string(), "verb".to_string()]
        );
    }
}
