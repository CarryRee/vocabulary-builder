#![allow(non_snake_case)]

use dioxus::prelude::*;
use js_sys::Date;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

static CSS: Asset = asset!("/assets/styles.css");
const WORDS_PER_PAGE: u32 = 10;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "log"], js_name = info)]
    async fn log_info(message: &str) -> Result<(), JsValue>;

    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "log"], js_name = error)]
    async fn log_error(message: &str) -> Result<(), JsValue>;
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum WordStatus {
    Unfamiliar,
    Known,
    Familiar,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Language {
    Zh,
    En,
    Ja,
}

impl Language {
    fn from_value(value: &str) -> Self {
        match value {
            "en" => Self::En,
            "ja" => Self::Ja,
            _ => Self::Zh,
        }
    }

    fn value(self) -> &'static str {
        match self {
            Self::Zh => "zh",
            Self::En => "en",
            Self::Ja => "ja",
        }
    }
}

#[derive(Clone, Copy)]
struct Text {
    app_title: &'static str,
    archive_label: &'static str,
    headline: &'static str,
    subtitle: &'static str,
    current_catalog: &'static str,
    word_count_suffix: &'static str,
    language_label: &'static str,
    new_entry: &'static str,
    edit_entry: &'static str,
    new_card: &'static str,
    edit_card: &'static str,
    word: &'static str,
    source_url: &'static str,
    familiarity: &'static str,
    word_placeholder: &'static str,
    unfamiliar_option: &'static str,
    known_option: &'static str,
    familiar_option: &'static str,
    add: &'static str,
    save: &'static str,
    cancel: &'static str,
    catalog: &'static str,
    status_filter: &'static str,
    search: &'static str,
    search_placeholder: &'static str,
    clear_search: &'static str,
    all: &'static str,
    loading: &'static str,
    empty_title: &'static str,
    empty_copy: &'static str,
    filtered_empty_title: &'static str,
    filtered_empty_copy: &'static str,
    search_empty_title: &'static str,
    search_empty_copy: &'static str,
    search_filtered_empty_title: &'static str,
    search_filtered_empty_copy: &'static str,
    pagination: &'static str,
    previous: &'static str,
    next: &'static str,
    preview: &'static str,
    parts_of_speech: &'static str,
    phonetic: &'static str,
    edit: &'static str,
    delete: &'static str,
    delete_eyebrow: &'static str,
    delete_title: &'static str,
    delete_copy: &'static str,
    keep: &'static str,
    delete_success: &'static str,
    add_success: &'static str,
    save_success: &'static str,
    load_error: &'static str,
    added_at: &'static str,
    settings: &'static str,
    storage_eyebrow: &'static str,
    storage_title: &'static str,
    storage_copy: &'static str,
    storage_directory: &'static str,
    storage_choose: &'static str,
    storage_empty: &'static str,
    storage_hint: &'static str,
    storage_save: &'static str,
    storage_success: &'static str,
}

fn text(language: Language) -> Text {
    match language {
        Language::Zh => Text {
            app_title: "词汇标本室",
            archive_label: "词汇标本室 · LOCAL ARCHIVE",
            headline: "把生词，放进记忆。",
            subtitle: "记录来源、判断熟悉度；下一次遇见时不再陌生。",
            current_catalog: "当前目录",
            word_count_suffix: "个单词",
            language_label: "语言",
            new_entry: "新建词条",
            edit_entry: "编辑词条",
            new_card: "收下一张词卡",
            edit_card: "修订这张词卡",
            word: "单词",
            source_url: "来源链接",
            familiarity: "此刻的熟悉度",
            word_placeholder: "例如 ephemeral",
            unfamiliar_option: "陌生 · 还需要认识",
            known_option: "了解 · 已有印象",
            familiar_option: "熟悉 · 可以复述",
            add: "加入档案",
            save: "保存修改",
            cancel: "取消",
            catalog: "词汇目录",
            status_filter: "按熟悉度筛选",
            search: "检索词条",
            search_placeholder: "输入单词后按 Enter",
            clear_search: "清除搜索",
            all: "全部",
            loading: "正在翻阅档案…",
            empty_title: "这里还没有词卡",
            empty_copy: "从左侧记下今天遇到的第一个陌生单词。",
            filtered_empty_title: "这个状态下还没有词卡",
            filtered_empty_copy: "切换状态筛选，或在左侧新增一张词卡。",
            search_empty_title: "没有找到匹配的单词",
            search_empty_copy: "换一个单词，或清除搜索后查看全部词卡。",
            search_filtered_empty_title: "当前筛选下没有匹配的单词",
            search_filtered_empty_copy: "调整熟悉度筛选，或清除搜索后继续浏览。",
            pagination: "词汇分页",
            previous: "← 上一页",
            next: "下一页 →",
            preview: "预览",
            parts_of_speech: "词性",
            phonetic: "音标",
            edit: "编辑",
            delete: "删除",
            delete_eyebrow: "删除词卡",
            delete_title: "确认移除这张词卡？",
            delete_copy: "删除后无法恢复，但不会影响来源网页。",
            keep: "保留词卡",
            delete_success: "词卡已删除。",
            add_success: "已加入词汇档案。",
            save_success: "已保存修改。",
            load_error: "无法读取单词：",
            added_at: "添加于",
            settings: "数据目录设置",
            storage_eyebrow: "本地存储",
            storage_title: "词汇数据库保存位置",
            storage_copy: "保存后会复制现有词汇到新目录，并立即使用新数据库。",
            storage_directory: "数据目录",
            storage_choose: "选择文件夹",
            storage_empty: "尚未选择目录",
            storage_hint: "请选择可写入的文件夹；目录中不能已有 vocabulary.db。",
            storage_save: "保存并迁移",
            storage_success: "词汇数据库已迁移到新目录。",
        },
        Language::En => Text {
            app_title: "Vocabulary Archive",
            archive_label: "VOCABULARY ARCHIVE · LOCAL",
            headline: "Make new words familiar.",
            subtitle: "Keep their source and track how well you know them.",
            current_catalog: "Current catalog",
            word_count_suffix: "words",
            language_label: "Language",
            new_entry: "New entry",
            edit_entry: "Edit entry",
            new_card: "Add a word card",
            edit_card: "Revise this card",
            word: "Word",
            source_url: "Source URL",
            familiarity: "Familiarity",
            word_placeholder: "e.g. ephemeral",
            unfamiliar_option: "Unfamiliar · still learning",
            known_option: "Known · rings a bell",
            familiar_option: "Familiar · can recall it",
            add: "Add to archive",
            save: "Save changes",
            cancel: "Cancel",
            catalog: "Word catalog",
            status_filter: "Filter by familiarity",
            search: "Find a word",
            search_placeholder: "Type a word and press Enter",
            clear_search: "Clear search",
            all: "All",
            loading: "Opening the archive…",
            empty_title: "No word cards yet",
            empty_copy: "Add the first new word from the panel on the left.",
            filtered_empty_title: "No cards in this status",
            filtered_empty_copy: "Change the filter or add a word card from the left.",
            search_empty_title: "No matching words",
            search_empty_copy: "Try another word or clear the search to see every card.",
            search_filtered_empty_title: "No matching words in this status",
            search_filtered_empty_copy: "Change the familiarity filter or clear the search.",
            pagination: "Word pagination",
            previous: "← Previous",
            next: "Next →",
            preview: "Open",
            parts_of_speech: "Part of speech",
            phonetic: "IPA",
            edit: "Edit",
            delete: "Delete",
            delete_eyebrow: "Delete word card",
            delete_title: "Remove this word card?",
            delete_copy: "This cannot be undone and will not affect the source page.",
            keep: "Keep card",
            delete_success: "Word card deleted.",
            add_success: "Added to the archive.",
            save_success: "Changes saved.",
            load_error: "Could not load words: ",
            added_at: "Added",
            settings: "Data directory settings",
            storage_eyebrow: "LOCAL STORAGE",
            storage_title: "Vocabulary database location",
            storage_copy: "Saving copies existing words to the new directory and switches to it immediately.",
            storage_directory: "Data directory",
            storage_choose: "Choose folder",
            storage_empty: "No folder selected",
            storage_hint: "Choose a writable folder. It cannot already contain vocabulary.db.",
            storage_save: "Save and migrate",
            storage_success: "The vocabulary database has moved to the new directory.",
        },
        Language::Ja => Text {
            app_title: "単語アーカイブ",
            archive_label: "単語アーカイブ · LOCAL ARCHIVE",
            headline: "知らない単語を、覚えていく。",
            subtitle: "出典を記録し、理解度を整理します。",
            current_catalog: "現在の一覧",
            word_count_suffix: "語",
            language_label: "言語",
            new_entry: "新しい単語",
            edit_entry: "単語を編集",
            new_card: "単語カードを追加",
            edit_card: "このカードを修正",
            word: "単語",
            source_url: "出典 URL",
            familiarity: "現在の理解度",
            word_placeholder: "例：ephemeral",
            unfamiliar_option: "知らない · これから覚える",
            known_option: "知っている · 見覚えがある",
            familiar_option: "身についている · 思い出せる",
            add: "アーカイブに追加",
            save: "変更を保存",
            cancel: "キャンセル",
            catalog: "単語一覧",
            status_filter: "理解度で絞り込む",
            search: "単語を検索",
            search_placeholder: "単語を入力して Enter",
            clear_search: "検索をクリア",
            all: "すべて",
            loading: "アーカイブを開いています…",
            empty_title: "単語カードはまだありません",
            empty_copy: "左側から最初の単語を記録しましょう。",
            filtered_empty_title: "この状態のカードはありません",
            filtered_empty_copy: "絞り込みを変更するか、左側から単語を追加してください。",
            search_empty_title: "一致する単語がありません",
            search_empty_copy: "別の単語で検索するか、検索をクリアしてすべてのカードを表示してください。",
            search_filtered_empty_title: "この状態に一致する単語はありません",
            search_filtered_empty_copy: "理解度の絞り込みを変更するか、検索をクリアしてください。",
            pagination: "単語ページ送り",
            previous: "← 前へ",
            next: "次へ →",
            preview: "開く",
            parts_of_speech: "品詞",
            phonetic: "発音記号",
            edit: "編集",
            delete: "削除",
            delete_eyebrow: "単語カードを削除",
            delete_title: "この単語カードを削除しますか？",
            delete_copy: "削除すると元に戻せません。出典ページには影響しません。",
            keep: "カードを残す",
            delete_success: "単語カードを削除しました。",
            add_success: "アーカイブに追加しました。",
            save_success: "変更を保存しました。",
            load_error: "単語を読み込めません：",
            added_at: "追加日時",
            settings: "データ保存先の設定",
            storage_eyebrow: "LOCAL STORAGE",
            storage_title: "単語データベースの保存先",
            storage_copy: "保存すると既存の単語を新しいフォルダーにコピーし、すぐに切り替えます。",
            storage_directory: "データフォルダー",
            storage_choose: "フォルダーを選択",
            storage_empty: "フォルダーが選択されていません",
            storage_hint: "書き込み可能なフォルダーを選択してください。vocabulary.db があるフォルダーは選べません。",
            storage_save: "保存して移行",
            storage_success: "単語データベースを新しいフォルダーへ移行しました。",
        },
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct VocabularyWord {
    id: String,
    word: String,
    url: String,
    status: WordStatus,
    phonetic: Option<String>,
    parts_of_speech: Vec<String>,
    created_at: i64,
    updated_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WordPage {
    words: Vec<VocabularyWord>,
    total: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WordInput {
    word: String,
    url: String,
    status: WordStatus,
}

#[derive(Serialize)]
struct ListArgs {
    status: Option<WordStatus>,
    query: Option<String>,
    page: u32,
}

#[derive(Serialize)]
struct CreateArgs {
    input: WordInput,
}

#[derive(Serialize)]
struct UpdateArgs {
    id: String,
    input: WordInput,
}

#[derive(Serialize)]
struct IdArgs {
    id: String,
}

#[derive(Serialize)]
struct UrlArgs {
    url: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DataDirectory {
    directory: String,
}

#[derive(Serialize)]
struct DataDirectoryArgs {
    directory: String,
}

#[derive(Serialize)]
struct DirectoryPickerArgs {
    title: String,
}

pub fn App() -> Element {
    let mut words = use_signal(Vec::<VocabularyWord>::new);
    let mut total_words = use_signal(|| 0_u32);
    let mut language = use_signal(|| Language::Zh);
    let mut status_filter = use_signal(|| "all".to_string());
    let mut search_draft = use_signal(String::new);
    let mut search_query = use_signal(String::new);
    let mut current_page = use_signal(|| 1_u32);
    let mut refresh_key = use_signal(|| 0_u32);
    let mut is_loading = use_signal(|| true);
    let mut notice = use_signal(String::new);
    let mut pending_delete = use_signal(|| Option::<VocabularyWord>::None);
    let mut editing_id = use_signal(|| Option::<String>::None);
    let mut draft_word = use_signal(String::new);
    let mut draft_url = use_signal(String::new);
    let mut draft_status = use_signal(|| WordStatus::Unfamiliar);
    let mut settings_open = use_signal(|| false);
    let mut data_directory = use_signal(String::new);

    use_effect(move || {
        spawn(async move {
            if let Ok(settings) = get_data_directory().await {
                data_directory.set(settings.directory);
            }
        });
    });

    use_effect(move || {
        let _ = refresh_key();
        let selected_status = filter_to_status(&status_filter());
        let selected_status_for_log = selected_status.clone();
        let selected_query = normalized_search_query(&search_query());
        let selected_query_for_log = selected_query.clone();
        let requested_page = current_page();
        let selected_language = language();
        spawn(async move {
            is_loading.set(true);
            frontend_info(format!(
                "frontend list request: status={selected_status:?}, query={selected_query_for_log:?}, page={requested_page}"
            ))
            .await;
            match list_words(selected_status, selected_query, requested_page).await {
                Ok(result) => {
                    let last_page = total_pages(result.total);
                    if result.total > 0 && requested_page > last_page {
                        frontend_info(format!(
                            "frontend list page corrected: requested_page={requested_page}, last_page={last_page}, total={}",
                            result.total
                        ))
                        .await;
                        current_page.set(last_page);
                    } else {
                        frontend_info(format!(
                            "frontend list completed: query={selected_query_for_log:?}, page={requested_page}, returned={}, total={}",
                            result.words.len(),
                            result.total
                        ))
                        .await;
                        words.set(result.words);
                        total_words.set(result.total);
                    }
                }
                Err(error) => {
                    frontend_error(format!(
                        "frontend list failed: status={selected_status_for_log:?}, query={selected_query_for_log:?}, page={requested_page}, error={error}"
                    ))
                    .await;
                    notice.set(format!(
                        "{}{}",
                        text(selected_language).load_error,
                        localized_error(&error, selected_language)
                    ))
                }
            }
            is_loading.set(false);
        });
    });

    let current_status = draft_status();
    let editing = editing_id();
    let active_language = language();
    let ui = text(active_language);
    let word_count = total_words();
    let page_count = total_pages(word_count);
    let active_page = current_page();
    let active_search_query = normalized_search_query(&search_query());
    let catalog_heading = filter_label(&status_filter(), active_language);
    let pagination_summary = page_summary(active_language, word_count, active_page, page_count);
    let deletion_candidate = pending_delete();

    rsx! {
        document::Title { "{ui.app_title}" }
        link { rel: "stylesheet", href: CSS }

        main { class: "app-shell",
            section { class: "masthead",
                div { class: "brand-mark", "Aa" }
                div { class: "brand-copy",
                    p { class: "eyebrow", "{ui.archive_label}" }
                    h1 { "{ui.headline}" }
                    p { class: "subtitle", "{ui.subtitle}" }
                }
                div { class: "header-tools",
                    div { class: "archive-count",
                        span { "{ui.current_catalog}" }
                        strong { "{word_count}" }
                        small { "{ui.word_count_suffix}" }
                    }
                    label { class: "language-picker", r#for: "language",
                        select {
                            id: "language",
                            "aria-label": "{ui.language_label}",
                            value: active_language.value(),
                            onchange: move |event| {
                                let selected_language = Language::from_value(&event.value());
                                language.set(selected_language);
                                spawn(async move {
                                    frontend_info(format!(
                                        "frontend language changed: language={}",
                                        selected_language.value()
                                    ))
                                    .await;
                                });
                            },
                            option { value: "zh", selected: active_language == Language::Zh, "中文" }
                            option { value: "en", selected: active_language == Language::En, "English" }
                            option { value: "ja", selected: active_language == Language::Ja, "日本語" }
                        }
                    }
                    button {
                        class: "header-icon-button",
                        r#type: "button",
                        title: "{ui.settings}",
                        "aria-label": "{ui.settings}",
                        onclick: move |_| settings_open.set(true),
                        "⚙"
                    }
                }
            }

            section { class: "workspace",
                aside { class: "entry-panel",
                    div { class: "section-heading",
                        p { class: "eyebrow", if editing.is_some() { "{ui.edit_entry}" } else { "{ui.new_entry}" } }
                        h2 { if editing.is_some() { "{ui.edit_card}" } else { "{ui.new_card}" } }
                    }
                    form {
                        class: "word-form",
                        onsubmit: move |event| {
                            event.prevent_default();
                            let input = WordInput {
                                word: draft_word(),
                                url: draft_url(),
                                status: draft_status(),
                            };
                            let target_id = editing_id();
                            spawn(async move {
                                let operation = if target_id.is_some() { "update" } else { "create" };
                                frontend_info(format!(
                                    "frontend {operation} submitted: id={target_id:?}, word={:?}, url={:?}, status={:?}",
                                    input.word,
                                    input.url,
                                    input.status
                                ))
                                .await;
                                let result = if let Some(id) = target_id {
                                    update_word(id, input).await.map(|_| ui.save_success)
                                } else {
                                    create_word(input).await.map(|_| ui.add_success)
                                };
                                match result {
                                    Ok(message) => {
                                        frontend_info(format!(
                                            "frontend {operation} completed successfully"
                                        ))
                                        .await;
                                        notice.set(message.to_string());
                                        draft_word.set(String::new());
                                        draft_url.set(String::new());
                                        draft_status.set(WordStatus::Unfamiliar);
                                        editing_id.set(None);
                                        refresh_key += 1;
                                    }
                                    Err(error) => {
                                        frontend_error(format!(
                                            "frontend {operation} failed: error={error}"
                                        ))
                                        .await;
                                        notice.set(localized_error(&error, active_language));
                                    }
                                }
                            });
                        },
                        label { r#for: "word", "{ui.word}" }
                        input {
                            id: "word",
                            placeholder: "{ui.word_placeholder}",
                            value: "{draft_word}",
                            maxlength: "120",
                            required: true,
                            oninput: move |event| draft_word.set(event.value()),
                        }
                        label { r#for: "source-url", "{ui.source_url}" }
                        input {
                            id: "source-url",
                            r#type: "url",
                            placeholder: "https://...",
                            value: "{draft_url}",
                            required: true,
                            oninput: move |event| draft_url.set(event.value()),
                        }
                        label { r#for: "status", "{ui.familiarity}" }
                        select {
                            id: "status",
                            value: status_value(&current_status),
                            onchange: move |event| draft_status.set(status_from_value(&event.value())),
                            option { value: "unfamiliar", selected: current_status == WordStatus::Unfamiliar, "{ui.unfamiliar_option}" }
                            option { value: "known", selected: current_status == WordStatus::Known, "{ui.known_option}" }
                            option { value: "familiar", selected: current_status == WordStatus::Familiar, "{ui.familiar_option}" }
                        }
                        div { class: "form-actions",
                            button { class: "primary-button", r#type: "submit",
                                if editing.is_some() { "{ui.save}" } else { "{ui.add}" }
                            }
                            if editing.is_some() {
                                button {
                                    class: "quiet-button",
                                    r#type: "button",
                                    onclick: move |_| {
                                        editing_id.set(None);
                                        draft_word.set(String::new());
                                        draft_url.set(String::new());
                                        draft_status.set(WordStatus::Unfamiliar);
                                    },
                                    "{ui.cancel}"
                                }
                            }
                        }
                    }
                    if !notice().is_empty() {
                        p { class: "notice", role: "status", "{notice}" }
                    }
                }

                section { class: "library-panel",
                        div { class: "library-toolbar",
                        div {
                            p { class: "eyebrow", "{ui.catalog}" }
                            h2 { "{catalog_heading}" }
                        }
                        div { class: "library-controls",
                            form {
                                class: "word-search",
                                role: "search",
                                onsubmit: move |event| {
                                    event.prevent_default();
                                    let query = search_draft().trim().to_string();
                                    if query == search_query() {
                                        return;
                                    }
                                    search_query.set(query.clone());
                                    current_page.set(1);
                                    spawn(async move {
                                        frontend_info(format!(
                                            "frontend word search submitted: query={query:?}, page=1"
                                        ))
                                        .await;
                                    });
                                },
                                label { class: "sr-only", r#for: "word-search", "{ui.search}" }
                                input {
                                    id: "word-search",
                                    r#type: "search",
                                    placeholder: "{ui.search_placeholder}",
                                    value: "{search_draft}",
                                    oninput: move |event| search_draft.set(event.value()),
                                }
                                if active_search_query.is_some() {
                                    button {
                                        class: "clear-search-button",
                                        r#type: "button",
                                        "aria-label": "{ui.clear_search}",
                                        title: "{ui.clear_search}",
                                        onclick: move |_| {
                                            search_draft.set(String::new());
                                            search_query.set(String::new());
                                            current_page.set(1);
                                            spawn(async move {
                                                frontend_info("frontend word search cleared: page=1".to_string()).await;
                                            });
                                        },
                                        "×"
                                    }
                                }
                            }
                            div { class: "filter-tabs", role: "group", "aria-label": "{ui.status_filter}",
                                FilterButton { value: "all", label: ui.all, active: status_filter() == "all", on_select: move |value| { log_filter_change(value, &mut status_filter, &mut current_page); } }
                                FilterButton { value: "unfamiliar", label: status_label(&WordStatus::Unfamiliar, active_language), active: status_filter() == "unfamiliar", on_select: move |value| { log_filter_change(value, &mut status_filter, &mut current_page); } }
                                FilterButton { value: "known", label: status_label(&WordStatus::Known, active_language), active: status_filter() == "known", on_select: move |value| { log_filter_change(value, &mut status_filter, &mut current_page); } }
                                FilterButton { value: "familiar", label: status_label(&WordStatus::Familiar, active_language), active: status_filter() == "familiar", on_select: move |value| { log_filter_change(value, &mut status_filter, &mut current_page); } }
                            }
                        }
                    }

                    if is_loading() {
                        div { class: "empty-state", "{ui.loading}" }
                    } else if words().is_empty() {
                        div { class: "empty-state",
                            span { class: "empty-glyph", "＋" }
                            if word_count == 0 && status_filter() == "all" && active_search_query.is_none() {
                                h3 { "{ui.empty_title}" }
                                p { "{ui.empty_copy}" }
                            } else if active_search_query.is_some() && status_filter() == "all" {
                                h3 { "{ui.search_empty_title}" }
                                p { "{ui.search_empty_copy}" }
                            } else if active_search_query.is_some() {
                                h3 { "{ui.search_filtered_empty_title}" }
                                p { "{ui.search_filtered_empty_copy}" }
                            } else if word_count == 0 {
                                h3 { "{ui.filtered_empty_title}" }
                                p { "{ui.filtered_empty_copy}" }
                            }
                        }
                    } else {
                        div { class: "word-list",
                            for record in words() {
                                WordCard {
                                    key: "{record.id}",
                                    record: record.clone(),
                                    language: active_language,
                                    on_preview: move |record: VocabularyWord| {
                                        spawn(async move {
                                            frontend_info(format!(
                                                "frontend source open requested: word={:?}, url={:?}, status={:?}",
                                                record.word,
                                                record.url,
                                                record.status
                                            ))
                                            .await;
                                            match open_source_url(record.url).await {
                                                Ok(()) => frontend_info("frontend source open dispatched".to_string()).await,
                                                Err(error) => {
                                                    frontend_error(format!(
                                                        "frontend source open failed: error={error}"
                                                    ))
                                                    .await;
                                                    notice.set(localized_error(&error, active_language));
                                                }
                                            }
                                        });
                                    },
                                    on_edit: move |record: VocabularyWord| {
                                        editing_id.set(Some(record.id));
                                        draft_word.set(record.word);
                                        draft_url.set(record.url);
                                        draft_status.set(record.status);
                                    },
                                    on_delete: move |record: VocabularyWord| {
                                        pending_delete.set(Some(record));
                                    },
                                }
                            }
                        }
                    }
                    if !is_loading() && word_count > 0 {
                        nav { class: "pagination", "aria-label": "{ui.pagination}",
                            button {
                                class: "page-button",
                                r#type: "button",
                                disabled: active_page <= 1,
                                onclick: move |_| {
                                    if active_page > 1 {
                                        let next_page = active_page - 1;
                                        current_page.set(next_page);
                                        spawn(async move {
                                            frontend_info(format!(
                                                "frontend page changed: page={next_page}"
                                            ))
                                            .await;
                                        });
                                    }
                                },
                                "{ui.previous}"
                            }
                            p { class: "page-summary", "{pagination_summary}" }
                            button {
                                class: "page-button",
                                r#type: "button",
                                disabled: active_page >= page_count,
                                onclick: move |_| {
                                    if active_page < page_count {
                                        let next_page = active_page + 1;
                                        current_page.set(next_page);
                                        spawn(async move {
                                            frontend_info(format!(
                                                "frontend page changed: page={next_page}"
                                            ))
                                            .await;
                                        });
                                    }
                                },
                                "{ui.next}"
                            }
                        }
                    }
                }
            }
            if let Some(record) = deletion_candidate {
                DeleteDialog {
                    record,
                    language: active_language,
                    on_cancel: move |_| pending_delete.set(None),
                    on_confirm: move |record: VocabularyWord| {
                        pending_delete.set(None);
                        let id = record.id;
                        let word = record.word;
                        let url = record.url;
                        let status = record.status;
                        spawn(async move {
                            frontend_info(format!(
                                "frontend delete submitted: id={id}, word={word:?}, url={url:?}, status={status:?}"
                            ))
                            .await;
                            match delete_word(id).await {
                                Ok(()) => {
                                    frontend_info("frontend delete completed successfully".to_string()).await;
                                    notice.set(ui.delete_success.to_string());
                                    refresh_key += 1;
                                }
                                Err(error) => {
                                    frontend_error(format!("frontend delete failed: error={error}"))
                                        .await;
                                    notice.set(localized_error(&error, active_language));
                                }
                            }
                        });
                    },
                }
            }
            if settings_open() {
                DataDirectoryDialog {
                    directory: data_directory(),
                    language: active_language,
                    on_close: move |_| settings_open.set(false),
                    on_saved: move |directory| {
                        data_directory.set(directory);
                        settings_open.set(false);
                        notice.set(ui.storage_success.to_string());
                    },
                }
            }
        }
    }
}

#[component]
fn FilterButton(
    value: &'static str,
    label: &'static str,
    active: bool,
    on_select: EventHandler<String>,
) -> Element {
    rsx! {
        button {
            class: if active { "filter-button active" } else { "filter-button" },
            r#type: "button",
            onclick: move |_| on_select.call(value.to_string()),
            "{label}"
        }
    }
}

#[component]
fn WordCard(
    record: VocabularyWord,
    language: Language,
    on_preview: EventHandler<VocabularyWord>,
    on_edit: EventHandler<VocabularyWord>,
    on_delete: EventHandler<VocabularyWord>,
) -> Element {
    let ui = text(language);
    let record_for_preview = record.clone();
    let record_for_edit = record.clone();
    let record_for_delete = record.clone();
    let status = record.status.clone();
    let added_at = format_added_at(record.created_at, language);
    rsx! {
        article { class: "word-card",
            div { class: "word-card-main",
                span { class: "status-dot {status_value(&status)}" }
                div {
                    div { class: "word-title-line",
                        h3 { "{record.word}" }
                        if let Some(phonetic) = record.phonetic.as_deref() {
                            span { class: "phonetic-inline", "aria-label": "{ui.phonetic}",
                                span { class: "phonetic-value", "{phonetic}" }
                            }
                        }
                    }
                    if !record.parts_of_speech.is_empty() {
                        p { class: "part-of-speech-line", "aria-label": "{ui.parts_of_speech}",
                            for part_of_speech in &record.parts_of_speech {
                                span { class: "part-of-speech-tag", "{part_of_speech}" }
                            }
                        }
                    }
                    p { class: "source-host", "{host_label(&record.url)}" }
                    p { class: "added-at", "{ui.added_at} · {added_at}" }
                }
            }
            span { class: "status-pill {status_value(&status)}", "{status_label(&status, language)}" }
            div { class: "card-actions",
                button { class: "card-button", r#type: "button", onclick: move |_| on_preview.call(record_for_preview.clone()), "{ui.preview}" }
                button { class: "card-button", r#type: "button", onclick: move |_| on_edit.call(record_for_edit.clone()), "{ui.edit}" }
                button { class: "card-button danger", r#type: "button", onclick: move |_| on_delete.call(record_for_delete.clone()), "{ui.delete}" }
            }
        }
    }
}

#[component]
fn DeleteDialog(
    record: VocabularyWord,
    language: Language,
    on_cancel: EventHandler<MouseEvent>,
    on_confirm: EventHandler<VocabularyWord>,
) -> Element {
    let ui = text(language);
    let record_for_delete = record.clone();
    rsx! {
        div { class: "delete-overlay", role: "presentation",
            section {
                class: "delete-dialog",
                role: "alertdialog",
                "aria-modal": "true",
                "aria-labelledby": "delete-dialog-title",
                div { class: "delete-mark", "×" }
                p { class: "eyebrow", "{ui.delete_eyebrow}" }
                h2 { id: "delete-dialog-title", "{ui.delete_title}" }
                p { class: "delete-word", "{record.word}" }
                p { class: "delete-copy", "{ui.delete_copy}" }
                div { class: "delete-actions",
                    button {
                        class: "quiet-button",
                        r#type: "button",
                        autofocus: true,
                        onclick: move |event| on_cancel.call(event),
                        "{ui.keep}"
                    }
                    button {
                        class: "delete-button",
                        r#type: "button",
                        onclick: move |_| on_confirm.call(record_for_delete.clone()),
                        "{ui.delete}"
                    }
                }
            }
        }
    }
}

#[component]
fn DataDirectoryDialog(
    directory: String,
    language: Language,
    on_close: EventHandler<MouseEvent>,
    on_saved: EventHandler<String>,
) -> Element {
    let ui = text(language);
    let mut directory_input = use_signal(|| directory);
    let mut feedback = use_signal(String::new);
    let mut is_saving = use_signal(|| false);

    rsx! {
        div { class: "delete-overlay", role: "presentation",
            section {
                class: "settings-dialog",
                role: "dialog",
                "aria-modal": "true",
                "aria-labelledby": "settings-dialog-title",
                p { class: "eyebrow", "{ui.storage_eyebrow}" }
                h2 { id: "settings-dialog-title", "{ui.storage_title}" }
                p { class: "settings-copy", "{ui.storage_copy}" }
                form {
                    class: "settings-form",
                    onsubmit: move |event| {
                        event.prevent_default();
                        let directory = directory_input();
                        is_saving.set(true);
                        feedback.set(String::new());
                        spawn(async move {
                            frontend_info(format!(
                                "frontend data directory migration submitted: directory={directory:?}"
                            ))
                            .await;
                            match set_data_directory(directory).await {
                                Ok(settings) => {
                                    frontend_info(format!(
                                        "frontend data directory migration completed: directory={:?}",
                                        settings.directory
                                    ))
                                    .await;
                                    on_saved.call(settings.directory);
                                }
                                Err(error) => {
                                    frontend_error(format!(
                                        "frontend data directory migration failed: error={error}"
                                    ))
                                    .await;
                                    feedback.set(localized_error(&error, language));
                                }
                            }
                            is_saving.set(false);
                        });
                    },
                    label { "{ui.storage_directory}" }
                    p {
                        class: if directory_input().is_empty() { "directory-display empty" } else { "directory-display" },
                        if directory_input().is_empty() { "{ui.storage_empty}" } else { "{directory_input}" }
                    }
                    button {
                        class: "choose-directory-button",
                        r#type: "button",
                        disabled: is_saving(),
                        onclick: move |_| {
                            let title = ui.storage_choose.to_string();
                            spawn(async move {
                                frontend_info("frontend data directory picker opened".to_string()).await;
                                match choose_data_directory(title).await {
                                    Ok(Some(directory)) => {
                                        directory_input.set(directory);
                                        feedback.set(String::new());
                                    }
                                    Ok(None) => {
                                        frontend_info("frontend data directory picker cancelled".to_string()).await;
                                    }
                                    Err(error) => {
                                        frontend_error(format!(
                                            "frontend data directory picker failed: error={error}"
                                        ))
                                        .await;
                                        feedback.set(localized_error(&error, language));
                                    }
                                }
                            });
                        },
                        "{ui.storage_choose}"
                    }
                    p { class: "settings-hint", "{ui.storage_hint}" }
                    if !feedback().is_empty() {
                        p { class: "settings-feedback", role: "status", "{feedback}" }
                    }
                    div { class: "settings-actions",
                        button {
                            class: "quiet-button",
                            r#type: "button",
                            disabled: is_saving(),
                            onclick: move |event| on_close.call(event),
                            "{ui.cancel}"
                        }
                        button {
                            class: "primary-button",
                            r#type: "submit",
                            disabled: is_saving() || directory_input().is_empty(),
                            if is_saving() { "…" } else { "{ui.storage_save}" }
                        }
                    }
                }
            }
        }
    }
}

fn log_filter_change(
    value: String,
    status_filter: &mut Signal<String>,
    current_page: &mut Signal<u32>,
) {
    status_filter.set(value.clone());
    current_page.set(1);
    spawn(async move {
        frontend_info(format!(
            "frontend status filter changed: status={value}, page=1"
        ))
        .await;
    });
}

async fn list_words(
    status: Option<WordStatus>,
    query: Option<String>,
    page: u32,
) -> Result<WordPage, String> {
    invoke_json(
        "list_words",
        &ListArgs {
            status,
            query,
            page,
        },
    )
    .await
}

async fn create_word(input: WordInput) -> Result<VocabularyWord, String> {
    invoke_json("create_word", &CreateArgs { input }).await
}

async fn update_word(id: String, input: WordInput) -> Result<VocabularyWord, String> {
    invoke_json("update_word", &UpdateArgs { id, input }).await
}

async fn delete_word(id: String) -> Result<(), String> {
    invoke_void("delete_word", &IdArgs { id }).await
}

async fn open_source_url(url: String) -> Result<(), String> {
    invoke_void("open_source_url", &UrlArgs { url }).await
}

async fn get_data_directory() -> Result<DataDirectory, String> {
    invoke_json("get_data_directory", &()).await
}

async fn choose_data_directory(title: String) -> Result<Option<String>, String> {
    invoke_json("choose_data_directory", &DirectoryPickerArgs { title }).await
}

async fn set_data_directory(directory: String) -> Result<DataDirectory, String> {
    invoke_json("set_data_directory", &DataDirectoryArgs { directory }).await
}

async fn frontend_info(message: String) {
    let _ = log_info(&message).await;
}

async fn frontend_error(message: String) {
    let _ = log_error(&message).await;
}

async fn invoke_json<T: for<'de> Deserialize<'de>, A: Serialize>(
    command: &str,
    args: &A,
) -> Result<T, String> {
    let args = serde_wasm_bindgen::to_value(args).map_err(|error| error.to_string())?;
    let response = invoke(command, args).await.map_err(js_error_message)?;
    serde_wasm_bindgen::from_value(response).map_err(|error| error.to_string())
}

async fn invoke_void<A: Serialize>(command: &str, args: &A) -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(args).map_err(|error| error.to_string())?;
    let _ = invoke(command, args).await.map_err(js_error_message)?;
    Ok(())
}

fn js_error_message(error: JsValue) -> String {
    error
        .as_string()
        .unwrap_or_else(|| "无法连接到 Tauri 后端，请完全重启应用。".to_string())
}

fn localized_error(error: &str, language: Language) -> String {
    let message = match error {
        "词汇数据库暂时不可用。" | "Vocabulary database is temporarily unavailable." => {
            match language {
                Language::Zh => "词汇数据库暂时不可用。",
                Language::En => "The vocabulary database is temporarily unavailable.",
                Language::Ja => "単語データベースは一時的に利用できません。",
            }
        }
        "Page number must start at 1." => match language {
            Language::Zh => "页码必须从 1 开始。",
            Language::En => "The page number must start at 1.",
            Language::Ja => "ページ番号は 1 から指定してください。",
        },
        "Page number is out of range." => match language {
            Language::Zh => "页码超出范围。",
            Language::En => "The page number is out of range.",
            Language::Ja => "ページ番号が範囲外です。",
        },
        "A word is required." => match language {
            Language::Zh => "请输入单词。",
            Language::En => "Enter a word.",
            Language::Ja => "単語を入力してください。",
        },
        "A word cannot exceed 120 characters." => match language {
            Language::Zh => "单词不能超过 120 个字符。",
            Language::En => "A word cannot exceed 120 characters.",
            Language::Ja => "単語は 120 文字以内にしてください。",
        },
        "A valid source URL is required." | "来源链接无效。" => match language {
            Language::Zh => "请输入有效的来源链接。",
            Language::En => "Enter a valid source URL.",
            Language::Ja => "有効な出典 URL を入力してください。",
        },
        "Source URL must start with http:// or https://."
        | "来源链接必须是 HTTP 或 HTTPS 地址。" => match language {
            Language::Zh => "来源链接必须以 http:// 或 https:// 开头。",
            Language::En => "The source URL must start with http:// or https://.",
            Language::Ja => "出典 URL は http:// または https:// で始まる必要があります。",
        },
        "Word to edit was not found." | "Updated word was not found." => match language {
            Language::Zh => "找不到要编辑的单词。",
            Language::En => "The word to edit could not be found.",
            Language::Ja => "編集する単語が見つかりません。",
        },
        "Word to delete was not found." => match language {
            Language::Zh => "找不到要删除的单词。",
            Language::En => "The word to delete could not be found.",
            Language::Ja => "削除する単語が見つかりません。",
        },
        "Word count exceeds the supported range." => match language {
            Language::Zh => "词汇数量超出支持范围。",
            Language::En => "The number of words exceeds the supported range.",
            Language::Ja => "単語数が対応範囲を超えています。",
        },
        "System time is before the Unix epoch." => match language {
            Language::Zh => "系统时间设置无效。",
            Language::En => "The system time is invalid.",
            Language::Ja => "システム時刻が無効です。",
        },
        "无法连接到 Tauri 后端，请完全重启应用。" => match language {
            Language::Zh => "无法连接到应用服务，请完全重启应用。",
            Language::En => {
                "Cannot connect to the application service. Restart the app completely."
            }
            Language::Ja => "アプリサービスに接続できません。アプリを完全に再起動してください。",
        },
        "Data directory must be an absolute path." => match language {
            Language::Zh => "请输入绝对路径，例如 D:\\VocabularyBuilderData。",
            Language::En => "Enter an absolute path, for example D:\\VocabularyBuilderData.",
            Language::Ja => "D:\\VocabularyBuilderData のような絶対パスを入力してください。",
        },
        "The selected directory already contains vocabulary.db." => match language {
            Language::Zh => "该目录已包含 vocabulary.db，请选择新的空目录。",
            Language::En => {
                "This directory already contains vocabulary.db. Choose a new empty directory."
            }
            Language::Ja => {
                "このフォルダーには vocabulary.db が既にあります。新しい空のフォルダーを選択してください。"
            }
        },
        "The copied vocabulary database could not be opened." => match language {
            Language::Zh => "无法验证复制后的词汇数据库，原数据库保持不变。",
            Language::En => {
                "The copied vocabulary database could not be verified. The original database was kept."
            }
            Language::Ja => {
                "コピーした単語データベースを確認できませんでした。元のデータベースは保持されています。"
            }
        },
        "Failed to save the data directory setting." => match language {
            Language::Zh => "无法保存数据目录设置，原数据库保持不变。",
            Language::En => {
                "The data directory setting could not be saved. The original database was kept."
            }
            Language::Ja => {
                "データ保存先を保存できませんでした。元のデータベースは保持されています。"
            }
        },
        _ => match language {
            Language::Zh => "操作未完成，请稍后重试。",
            Language::En => "The operation could not be completed. Please try again.",
            Language::Ja => "操作を完了できませんでした。もう一度お試しください。",
        },
    };

    message.to_string()
}

fn filter_to_status(value: &str) -> Option<WordStatus> {
    match value {
        "unfamiliar" => Some(WordStatus::Unfamiliar),
        "known" => Some(WordStatus::Known),
        "familiar" => Some(WordStatus::Familiar),
        _ => None,
    }
}

fn normalized_search_query(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn total_pages(total: u32) -> u32 {
    total.div_ceil(WORDS_PER_PAGE).max(1)
}

fn status_from_value(value: &str) -> WordStatus {
    filter_to_status(value).unwrap_or(WordStatus::Unfamiliar)
}

fn status_value(status: &WordStatus) -> &'static str {
    match status {
        WordStatus::Unfamiliar => "unfamiliar",
        WordStatus::Known => "known",
        WordStatus::Familiar => "familiar",
    }
}

fn status_label(status: &WordStatus, language: Language) -> &'static str {
    match status {
        WordStatus::Unfamiliar => match language {
            Language::Zh => "陌生",
            Language::En => "Unfamiliar",
            Language::Ja => "知らない",
        },
        WordStatus::Known => match language {
            Language::Zh => "了解",
            Language::En => "Known",
            Language::Ja => "知っている",
        },
        WordStatus::Familiar => match language {
            Language::Zh => "熟悉",
            Language::En => "Familiar",
            Language::Ja => "身についている",
        },
    }
}

fn filter_label(value: &str, language: Language) -> &'static str {
    let ui = text(language);
    match value {
        "unfamiliar" => match language {
            Language::Zh => "陌生词汇",
            Language::En => "Unfamiliar words",
            Language::Ja => "知らない単語",
        },
        "known" => match language {
            Language::Zh => "已有印象",
            Language::En => "Known words",
            Language::Ja => "見覚えのある単語",
        },
        "familiar" => match language {
            Language::Zh => "熟悉词汇",
            Language::En => "Familiar words",
            Language::Ja => "身についている単語",
        },
        _ => ui.all,
    }
}

fn page_summary(language: Language, total: u32, page: u32, pages: u32) -> String {
    match language {
        Language::Zh => format!("共 {total} 条 · 第 {page} / {pages} 页"),
        Language::En => format!("{total} words · Page {page} / {pages}"),
        Language::Ja => format!("全 {total} 語 · {page} / {pages} ページ"),
    }
}

fn format_added_at(timestamp: i64, language: Language) -> String {
    let date = Date::new(&JsValue::from_f64(timestamp as f64 * 1_000.0));
    let year = date.get_full_year();
    let month = date.get_month() + 1;
    let day = date.get_date();
    let hour = date.get_hours();
    let minute = date.get_minutes();

    match language {
        Language::Zh => format!("{year}年{month:02}月{day:02}日 {hour:02}:{minute:02}"),
        Language::En => format!("{year}-{month:02}-{day:02} {hour:02}:{minute:02}"),
        Language::Ja => format!("{year}年{month}月{day}日 {hour:02}:{minute:02}"),
    }
}

fn host_label(value: &str) -> String {
    value
        .split("//")
        .nth(1)
        .unwrap_or(value)
        .split('/')
        .next()
        .unwrap_or(value)
        .to_string()
}
