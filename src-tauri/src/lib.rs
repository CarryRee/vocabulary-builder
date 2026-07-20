mod pronunciation;
mod vocabulary;

use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind, TimezoneStrategy};
use tauri_plugin_opener::OpenerExt;
use vocabulary::{
    VocabularyRepository, VocabularyWord, WordInput, WordListRequest, WordPage, WordStatus,
};

struct AppState {
    repository: Mutex<Option<VocabularyRepository>>,
    database_path: Mutex<PathBuf>,
    settings_path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct StorageConfiguration {
    data_directory: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DataDirectory {
    directory: String,
}

#[tauri::command]
fn list_words(
    state: State<'_, AppState>,
    status: Option<WordStatus>,
    query: Option<String>,
    page: u32,
) -> Result<WordPage, String> {
    log::info!("list_words requested: status={status:?}, query={query:?}, page={page}");
    let requested_status = status.clone();
    let requested_query = query.clone();
    let result = access_repository(&state, |repository| {
        repository.list(WordListRequest {
            status,
            query,
            page,
        })
    });

    match &result {
        Ok(word_page) => log::info!(
            "list_words completed: query={requested_query:?}, page={page}, returned={}, total={}",
            word_page.words.len(),
            word_page.total
        ),
        Err(error) => {
            log::error!(
                "list_words failed: status={requested_status:?}, query={requested_query:?}, page={page}, error={error}"
            )
        }
    }

    result
}

#[tauri::command]
async fn create_word(
    state: State<'_, AppState>,
    input: WordInput,
) -> Result<VocabularyWord, String> {
    log::info!(
        "create_word requested: word={:?}, url={:?}, status={:?}",
        input.word,
        input.url,
        input.status
    );
    let created = access_repository(&state, |repository| repository.create(input));
    let created = match created {
        Ok(word) => {
            log::info!(
                "create_word completed: id={}, word={:?}, url={:?}, status={:?}",
                word.id,
                word.word,
                word.url,
                word.status
            );
            word
        }
        Err(error) => {
            log::error!("create_word failed: error={error}");
            return Err(error);
        }
    };

    Ok(lookup_missing_dictionary_data(&state, created).await)
}

#[tauri::command]
async fn update_word(
    state: State<'_, AppState>,
    id: String,
    input: WordInput,
) -> Result<VocabularyWord, String> {
    log::info!(
        "update_word requested: id={id}, word={:?}, url={:?}, status={:?}",
        input.word,
        input.url,
        input.status
    );
    let updated = access_repository(&state, |repository| repository.update(&id, input));
    let updated = match updated {
        Ok(word) => {
            log::info!(
                "update_word completed: id={}, word={:?}, url={:?}, status={:?}",
                word.id,
                word.word,
                word.url,
                word.status
            );
            word
        }
        Err(error) => {
            log::error!("update_word failed: id={id}, error={error}");
            return Err(error);
        }
    };

    Ok(lookup_missing_dictionary_data(&state, updated).await)
}

async fn lookup_missing_dictionary_data(state: &AppState, word: VocabularyWord) -> VocabularyWord {
    if word.phonetic.is_some() && !word.parts_of_speech.is_empty() {
        return word;
    }

    log::info!(
        "dictionary metadata lookup requested: id={}, word={:?}",
        word.id,
        word.word
    );
    let pronunciation = match pronunciation::lookup(&word.word).await {
        Ok(pronunciation) => pronunciation,
        Err(error) => {
            log::warn!(
                "dictionary metadata lookup skipped: id={}, word={:?}, error={error}",
                word.id,
                word.word
            );
            return word;
        }
    };
    let phonetic = word.phonetic.clone().or(pronunciation.phonetic);
    let parts_of_speech = if word.parts_of_speech.is_empty() {
        pronunciation.parts_of_speech
    } else {
        word.parts_of_speech.clone()
    };
    if phonetic.is_none() && parts_of_speech.is_empty() {
        log::info!(
            "dictionary metadata lookup completed without data: id={}, word={:?}",
            word.id,
            word.word
        );
        return word;
    }

    match access_repository(state, |repository| {
        repository.save_dictionary_data(&word.id, phonetic, parts_of_speech)
    }) {
        Ok(saved) => {
            log::info!(
                "dictionary metadata saved: id={}, word={:?}, phonetic={:?}, parts_of_speech={:?}",
                saved.id,
                saved.word,
                saved.phonetic,
                saved.parts_of_speech
            );
            saved
        }
        Err(error) => {
            log::error!(
                "dictionary metadata save failed: id={}, word={:?}, error={error}",
                word.id,
                word.word
            );
            word
        }
    }
}

#[tauri::command]
fn delete_word(state: State<'_, AppState>, id: String) -> Result<(), String> {
    log::info!("delete_word requested: id={id}");
    let result = access_repository(&state, |repository| repository.delete(&id));

    match &result {
        Ok(()) => log::info!("delete_word completed: id={id}"),
        Err(error) => log::error!("delete_word failed: id={id}, error={error}"),
    }

    result
}

#[tauri::command]
fn open_source_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    log::info!("open_source_url requested: url={url:?}");
    if let Err(error) = parse_source_url(&url) {
        log::error!("open_source_url rejected: url={url:?}, error={error}");
        return Err(error);
    }

    let url_for_log = url.clone();
    let result = app
        .opener()
        .open_url(url, None::<String>)
        .map_err(|error| error.to_string());

    match &result {
        Ok(()) => log::info!("open_source_url dispatched successfully: url={url_for_log:?}"),
        Err(error) => log::error!("open_source_url failed: url={url_for_log:?}, error={error}"),
    }

    result
}

#[tauri::command]
fn get_data_directory(state: State<'_, AppState>) -> Result<DataDirectory, String> {
    let database_path = state
        .database_path
        .lock()
        .map_err(|_| "Vocabulary database is temporarily unavailable.".to_string())?;
    let directory = database_path
        .parent()
        .ok_or_else(|| "Database directory is unavailable.".to_string())?;

    Ok(DataDirectory {
        directory: directory.to_string_lossy().into_owned(),
    })
}

#[tauri::command]
async fn choose_data_directory(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    title: String,
) -> Result<Option<String>, String> {
    let current_directory = get_data_directory(state)?.directory;
    log::info!("data directory picker opened: current_directory={current_directory:?}");

    let selected_directory = app
        .dialog()
        .file()
        .set_title(title)
        .set_directory(current_directory)
        .blocking_pick_folder();

    selected_directory
        .map(|file_path| {
            file_path.into_path().map(|path| {
                let directory = path.to_string_lossy().into_owned();
                log::info!("data directory picker selected: directory={directory:?}");
                directory
            })
        })
        .transpose()
        .map_err(|error| {
            log::error!("data directory picker returned an unusable path: {error}");
            "Selected directory is unavailable.".to_string()
        })
}

#[tauri::command]
fn set_data_directory(
    state: State<'_, AppState>,
    directory: String,
) -> Result<DataDirectory, String> {
    let directory = PathBuf::from(directory.trim());
    if !directory.is_absolute() {
        return Err("Data directory must be an absolute path.".to_string());
    }

    fs::create_dir_all(&directory)
        .map_err(|error| format!("Failed to create data directory: {error}"))?;
    let target_database_path = directory.join("vocabulary.db");
    let mut database_path = state
        .database_path
        .lock()
        .map_err(|_| "Vocabulary database is temporarily unavailable.".to_string())?;

    if *database_path == target_database_path {
        return Ok(DataDirectory {
            directory: directory.to_string_lossy().into_owned(),
        });
    }
    if target_database_path.exists() {
        return Err("The selected directory already contains vocabulary.db.".to_string());
    }
    log::info!("data directory migration requested: from={database_path:?}, to={directory:?}");
    let source_database_path = database_path.clone();
    let mut repository = state
        .repository
        .lock()
        .map_err(|_| "Vocabulary database is temporarily unavailable.".to_string())?;
    let previous_repository = repository
        .take()
        .ok_or_else(|| "Vocabulary database is temporarily unavailable.".to_string())?;
    drop(previous_repository);

    if let Err(error) = fs::copy(&source_database_path, &target_database_path) {
        restore_repository(&mut repository, &source_database_path);
        log::error!(
            "data directory migration copy failed: from={source_database_path:?}, to={target_database_path:?}, error={error}"
        );
        return Err(format!("Failed to copy vocabulary database: {error}"));
    }

    let next_repository = match VocabularyRepository::open(&target_database_path) {
        Ok(repository) => repository,
        Err(error) => {
            let _ = fs::remove_file(&target_database_path);
            restore_repository(&mut repository, &source_database_path);
            log::error!(
                "data directory migration validation failed: target={target_database_path:?}, error={error}"
            );
            return Err("The copied vocabulary database could not be opened.".to_string());
        }
    };

    if let Err(error) = write_storage_configuration(&state.settings_path, &directory) {
        drop(next_repository);
        let _ = fs::remove_file(&target_database_path);
        restore_repository(&mut repository, &source_database_path);
        log::error!("data directory configuration save failed: error={error}");
        return Err("Failed to save the data directory setting.".to_string());
    }

    *repository = Some(next_repository);
    *database_path = target_database_path;
    if let Err(error) = fs::remove_file(&source_database_path) {
        log::warn!(
            "data directory migration completed but old database could not be removed: path={source_database_path:?}, error={error}"
        );
    }
    log::info!("data directory migration completed: directory={directory:?}");

    Ok(DataDirectory {
        directory: directory.to_string_lossy().into_owned(),
    })
}

fn access_repository<T>(
    state: &AppState,
    operation: impl FnOnce(&VocabularyRepository) -> Result<T, String>,
) -> Result<T, String> {
    let repository = state
        .repository
        .lock()
        .map_err(|_| "Vocabulary database is temporarily unavailable.".to_string())?;
    let repository = repository
        .as_ref()
        .ok_or_else(|| "Vocabulary database is temporarily unavailable.".to_string())?;
    operation(repository)
}

fn restore_repository(repository: &mut Option<VocabularyRepository>, database_path: &Path) {
    match VocabularyRepository::open(database_path) {
        Ok(restored_repository) => *repository = Some(restored_repository),
        Err(error) => log::error!(
            "failed to restore database after data directory migration failure: path={database_path:?}, error={error}"
        ),
    }
}

fn load_storage_configuration(settings_path: &Path) -> Result<StorageConfiguration, String> {
    match fs::read_to_string(settings_path) {
        Ok(contents) => serde_json::from_str(&contents)
            .map_err(|error| format!("Failed to read data directory setting: {error}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(StorageConfiguration::default())
        }
        Err(error) => Err(format!("Failed to read data directory setting: {error}")),
    }
}

fn write_storage_configuration(settings_path: &Path, directory: &Path) -> Result<(), String> {
    let parent = settings_path
        .parent()
        .ok_or_else(|| "Settings directory is unavailable.".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Failed to create settings directory: {error}"))?;
    let configuration = StorageConfiguration {
        data_directory: Some(directory.to_string_lossy().into_owned()),
    };
    let contents = serde_json::to_string_pretty(&configuration)
        .map_err(|error| format!("Failed to serialize data directory setting: {error}"))?;
    fs::write(settings_path, contents)
        .map_err(|error| format!("Failed to write data directory setting: {error}"))
}

fn parse_source_url(value: &str) -> Result<url::Url, String> {
    let parsed = url::Url::parse(value).map_err(|_| "来源链接无效。".to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        return Err("来源链接必须是 HTTP 或 HTTPS 地址。".to_string());
    }
    Ok(parsed)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir {
                        file_name: Some("vocabulary-builder".into()),
                    }),
                ])
                .level(log::LevelFilter::Info)
                .max_file_size(1_000_000)
                .rotation_strategy(RotationStrategy::KeepAll)
                .timezone_strategy(TimezoneStrategy::UseLocal)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            log::info!("Vocabulary Builder starting");
            let default_data_dir = app.path().app_data_dir().map_err(|error| {
                log::error!("Failed to resolve application data directory: {error}");
                error
            })?;
            let settings_path = app
                .path()
                .app_config_dir()
                .map_err(|error| {
                    log::error!("Failed to resolve application settings directory: {error}");
                    error
                })?
                .join("storage-settings.json");
            let configuration =
                load_storage_configuration(&settings_path).map_err(std::io::Error::other)?;
            let data_dir = configuration
                .data_directory
                .map(PathBuf::from)
                .unwrap_or(default_data_dir);
            fs::create_dir_all(&data_dir).map_err(|error| {
                log::error!("Failed to create application data directory {data_dir:?}: {error}");
                error
            })?;
            let database_path = data_dir.join("vocabulary.db");
            let repository = VocabularyRepository::open(&database_path).map_err(|error| {
                log::error!("Failed to initialize database {database_path:?}: {error}");
                std::io::Error::other(error)
            })?;
            app.manage(AppState {
                repository: Mutex::new(Some(repository)),
                database_path: Mutex::new(database_path.clone()),
                settings_path,
            });
            log::info!("Database initialized: {database_path:?}");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_words,
            create_word,
            update_word,
            delete_word,
            open_source_url,
            get_data_directory,
            choose_data_directory,
            set_data_directory
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| log::error!("Application stopped with an error: {error}"));
}
