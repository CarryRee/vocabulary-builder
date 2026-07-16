mod vocabulary;

use std::{fs, sync::Mutex};
use tauri::{Manager, State};
use tauri_plugin_log::{RotationStrategy, Target, TargetKind, TimezoneStrategy};
use tauri_plugin_opener::OpenerExt;
use vocabulary::{
    VocabularyRepository, VocabularyWord, WordInput, WordListRequest, WordPage, WordStatus,
};

struct AppState {
    repository: Mutex<VocabularyRepository>,
}

#[tauri::command]
fn list_words(
    state: State<'_, AppState>,
    status: Option<WordStatus>,
    page: u32,
) -> Result<WordPage, String> {
    log::info!("list_words requested: status={status:?}, page={page}");
    let requested_status = status.clone();
    let result = match state.repository.lock() {
        Ok(repository) => repository.list(WordListRequest { status, page }),
        Err(_) => Err("词汇数据库暂时不可用。".to_string()),
    };

    match &result {
        Ok(word_page) => log::info!(
            "list_words completed: page={page}, returned={}, total={}",
            word_page.words.len(),
            word_page.total
        ),
        Err(error) => {
            log::error!(
                "list_words failed: status={requested_status:?}, page={page}, error={error}"
            )
        }
    }

    result
}

#[tauri::command]
fn create_word(state: State<'_, AppState>, input: WordInput) -> Result<VocabularyWord, String> {
    log::info!(
        "create_word requested: word={:?}, url={:?}, status={:?}",
        input.word,
        input.url,
        input.status
    );
    let result = match state.repository.lock() {
        Ok(repository) => repository.create(input),
        Err(_) => Err("词汇数据库暂时不可用。".to_string()),
    };

    match &result {
        Ok(word) => log::info!(
            "create_word completed: id={}, word={:?}, url={:?}, status={:?}",
            word.id,
            word.word,
            word.url,
            word.status
        ),
        Err(error) => log::error!("create_word failed: error={error}"),
    }

    result
}

#[tauri::command]
fn update_word(
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
    let result = match state.repository.lock() {
        Ok(repository) => repository.update(&id, input),
        Err(_) => Err("词汇数据库暂时不可用。".to_string()),
    };

    match &result {
        Ok(word) => log::info!(
            "update_word completed: id={}, word={:?}, url={:?}, status={:?}",
            word.id,
            word.word,
            word.url,
            word.status
        ),
        Err(error) => log::error!("update_word failed: id={id}, error={error}"),
    }

    result
}

#[tauri::command]
fn delete_word(state: State<'_, AppState>, id: String) -> Result<(), String> {
    log::info!("delete_word requested: id={id}");
    let result = match state.repository.lock() {
        Ok(repository) => repository.delete(&id),
        Err(_) => Err("词汇数据库暂时不可用。".to_string()),
    };

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
        .setup(|app| {
            log::info!("Vocabulary Builder starting");
            let data_dir = app.path().app_data_dir().map_err(|error| {
                log::error!("Failed to resolve application data directory: {error}");
                error
            })?;
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
                repository: Mutex::new(repository),
            });
            log::info!("Database initialized: {database_path:?}");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_words,
            create_word,
            update_word,
            delete_word,
            open_source_url
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| log::error!("Application stopped with an error: {error}"));
}
