use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use super::ai_history::{
    self, export_filename, export_json, fresh_id, ChatTurn, Conversation, Role, Summary,
};
use super::side_panel::{Side, SidePanel};
use crate::languages::{use_strings, Strings};
use crate::nixdesk::js_util::{eval_js, set_field};
use crate::platform::Date;

// Penerus resmi `gemini-3.5-flash` dan `gemini-3.7-flash` (Google mengalihkan
// nama lama ke dua ini). Yang pertama jadi bawaan, seperti urutan sebelumnya.
const GEMINI_36_FLASH: &str = "gemini-3.6-flash";
const GEMINI_38_FLASH: &str = "gemini-3.8-flash";

const MODEL_OPTIONS: [(&str, &str); 2] = [
    (GEMINI_36_FLASH, "3.6 Flash"),
    (GEMINI_38_FLASH, "3.8 Flash"),
];

/// Textarea pertanyaan: mulai 1 baris, tumbuh sampai 5 baris (`js/autosize.js`).
const QUESTION_ID: &str = "assistant-question";
const QUESTION_MAX_ROWS: u32 = 5;
const AUTOSIZE_JS: &str = include_str!("../../../js/autosize.js");

/// Pembungkus jendela: tempat panel riwayat menempel (di dalam area jendela,
/// bukan viewport; `overflow-hidden` memotong panel di tepi jendela).
const OUTER_CLASS: &str = "ai";
/// Panel riwayat = `SidePanel` (scrim, geser buka/tutup) + gaya isi milik AI.
const HISTORY_PANEL_ID: &str = "ai-history-panel";
const PANEL_CLASS: &str = "ai__history";
/// Tombol buka/tutup riwayat: ikon `ai__icon-btn` yang diposisikan absolute di
/// pojok kiri atas `ai__root` (bukan lagi di footer form).
const HISTORY_TOGGLE_CLASS: &str = "ai__icon-btn ai__history-toggle";
const PANEL_HEADER_CLASS: &str = "ai__history-head";
const PANEL_TITLE_CLASS: &str = "ai__history-title";
const NEW_CHAT_CLASS: &str = "ai__new-chat";
const NOTE_CLASS: &str = "ai__note";
const LIST_CLASS: &str = "ai__history-list";
const ROW_CLASS: &str = "ai__row ai__row--idle";
const ROW_ACTIVE_CLASS: &str = "ai__row ai__row--active";
const ROW_OPEN_CLASS: &str = "ai__row-open";
const ROW_TITLE_CLASS: &str = "ai__row-title";
const ROW_DATE_CLASS: &str = "ai__row-date";
const ROW_ACTIONS_CLASS: &str = "ai__row-actions";
const TEXT_BUTTON_CLASS: &str = "ai__text-btn";

const ROOT_CLASS: &str = "ai__root";
const CONVERSATION_CLASS: &str = "ai__conversation";
const USER_BUBBLE_CLASS: &str = "ai__user";
const USER_TEXT_CLASS: &str = "ai__user-text";
const ASSISTANT_ARTICLE_CLASS: &str = "ai__reply";
const ASSISTANT_ICON_CLASS: &str = "ai__reply-icon";
const ASSISTANT_BODY_CLASS: &str = "ai__reply-body";
const ASSISTANT_TEXT_CLASS: &str = "ai__reply-text";
const CODE_FIGURE_CLASS: &str = "ai__code";
const CODE_GUTTER_CLASS: &str = "ai__code-gutter";
const CODE_NUMBERS_CLASS: &str = "ai__code-lines";
const CODE_PRE_CLASS: &str = "ai__code-pre";
const FOOTER_CLASS: &str = "ai__footer";
const FORM_CLASS: &str = "ai__form";
// Tinggi diatur `js/autosize.js` (1 sampai 5 baris); padding 3px + 19.6px baris
// = 25.6px, setara tombol ikon di sampingnya.
const QUESTION_INPUT_CLASS: &str = "ai__question";
const ICON_BUTTON_CLASS: &str = "ai__icon-btn";
const KEY_ROW_CLASS: &str = "ai__key-row";
const KEY_GROUP_CLASS: &str = "ai__key-group";
const KEY_LABEL_CLASS: &str = "ai__key-label";
const KEY_FIELD_CLASS: &str = "ai__key-field";
const KEY_INPUT_CLASS: &str = "ai__key-input";
const SHOW_BUTTON_CLASS: &str = "ai__key-show";
const INFO_BUTTON_CLASS: &str = "ai__info-btn";
const SETTINGS_WRAP_CLASS: &str = "ai__settings";
const SETTINGS_TOGGLE_CLASS: &str = "ai__settings-toggle";
const MODEL_MENU_CLASS: &str = "ai__model-menu";

const ICON_ASSISTANT: Asset = asset!("/assets/logo-Ai-Assistent.svg");
const ICON_ADD: Asset = asset!("/assets/icon-add.svg");
const ICON_CLOSE: Asset = asset!("/assets/icon-close.svg");
const ICON_SIDEBAR: Asset = asset!("/assets/icon-sidebar.svg");
const ICON_SEND: Asset = asset!("assets/icon-send.svg");
const ICON_INFO: Asset = asset!("assets/icon-info.svg");
const ICON_TOGGLE: Asset = asset!("assets/icon-toggle-popup.svg");

/// Kenapa permintaan ke Gemini gagal. Diterjemahkan saat ditampilkan
/// (`AiError::message`); teks dari Google atau dari jaringan ditampilkan apa adanya.
#[derive(Clone, PartialEq, Debug)]
enum AiError {
    MissingKey,
    HttpStatus(u16),
    /// Pesan dari Google (`error.message`) atau dari lapisan jaringan.
    Remote(String),
}

impl AiError {
    fn message(&self, s: &Strings) -> String {
        match self {
            AiError::MissingKey => s.ai.missing_key.to_string(),
            AiError::HttpStatus(status) => s.ai.http_failed(*status),
            AiError::Remote(text) => text.clone(),
        }
    }
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize, Clone)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize, Default)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
    error: Option<GeminiError>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContentResp,
}

#[derive(Deserialize)]
struct GeminiContentResp {
    parts: Vec<GeminiPart>,
}

#[derive(Deserialize)]
struct GeminiError {
    message: String,
}

async fn call_gemini(
    api_key: String,
    model: String,
    history: Vec<ChatTurn>,
    s: &'static Strings,
) -> Result<String, AiError> {
    let contents: Vec<GeminiContent> = history
        .iter()
        .map(|turn| GeminiContent {
            role: match turn.role {
                Role::User => "user".to_string(),
                Role::Assistant => "model".to_string(),
            },
            parts: vec![GeminiPart {
                text: turn.text.clone(),
            }],
        })
        .collect();

    // API key dikirim lewat header `x-goog-api-key`, BUKAN query string
    // (`?key=...`): URL ikut tercatat di log proxy/server, riwayat browser,
    // dan pesan error jaringan, sedangkan header tidak.
    let url =
        format!("https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent");
    let body = GeminiRequest { contents };

    let payload = serde_json::to_string(&body).map_err(|e| AiError::Remote(e.to_string()))?;

    let response = crate::platform::http::post_json_with_headers(
        &url,
        payload,
        &[("x-goog-api-key", api_key.trim())],
    )
    .await
    .map_err(AiError::Remote)?;
    let status = response.status;
    let parsed: GeminiResponse = serde_json::from_str(&response.body).unwrap_or_default();

    if let Some(err) = parsed.error {
        return Err(AiError::Remote(err.message));
    }
    if !(200..300).contains(&status) {
        return Err(AiError::HttpStatus(status));
    }

    let text = parsed
        .candidates
        .unwrap_or_default()
        .first()
        .and_then(|c| c.content.parts.first())
        .map(|p| p.text.clone())
        .unwrap_or_else(|| s.ai.empty_response.to_string());
    Ok(text)
}

/// Chat yang sedang tampil. Id dibuat saat jendela dibuka atau "New chat";
/// chat baru disimpan ke riwayat baru setelah pesan pertama terkirim.
#[derive(Clone, PartialEq)]
struct ActiveChat {
    id: String,
    created_at: i64,
}

impl ActiveChat {
    fn fresh() -> Self {
        Self {
            id: fresh_id(),
            created_at: now_ms(),
        }
    }
}

fn now_ms() -> i64 {
    Date::now() as i64
}

/// Semua keadaan jendela chat. Semua isinya `Signal` (Copy), jadi struct ini
/// bisa diteruskan ke fungsi dan ke dalam `spawn` tanpa clone.
#[derive(Clone, Copy)]
struct ChatState {
    history: Signal<Vec<ChatTurn>>,
    draft: Signal<String>,
    sending: Signal<bool>,
    error: Signal<Option<AiError>>,
    active: Signal<ActiveChat>,
    chats: Signal<Vec<Summary>>,
    /// `false` bila browser menolak IndexedDB (mode privat tertentu, dll.).
    /// Chat tetap jalan, hanya tidak tersimpan.
    store_ok: Signal<bool>,
}

/// Menyimpan satu chat lalu memperbarui daftar di panel. Kegagalan hanya
/// menandai penyimpanan tidak tersedia; percakapannya sendiri tidak terganggu.
async fn persist(chat: Conversation, mut chats: Signal<Vec<Summary>>, mut store_ok: Signal<bool>) {
    let summary = chat.summary();
    match ai_history::save(&chat).await {
        Ok(()) => {
            store_ok.set(true);
            let mut list = chats();
            let dropped = ai_history::upsert(&mut list, summary);
            chats.set(list);
            if !dropped.is_empty() && ai_history::delete(&dropped).await.is_err() {
                store_ok.set(false);
            }
        }
        Err(_) => store_ok.set(false),
    }
}

/// A plain function (not a shared closure) so it can be called from both
/// the form submit handler and the Enter key without any closure-capture
/// sharing concerns.
fn send_prompt(state: ChatState, api_key: String, model: String, s: &'static Strings) {
    let ChatState {
        mut history,
        mut draft,
        mut sending,
        mut error,
        active,
        chats,
        store_ok,
    } = state;
    if sending() {
        return;
    }
    // Isi mentah dipegang untuk membersihkan kolom: hanya dikosongkan bila
    // DOM masih memuat persis teks ini (ketikan yang menyusul tidak hilang).
    let raw = draft();
    let text = raw.trim().to_string();
    if text.is_empty() {
        return;
    }
    if api_key.trim().is_empty() {
        error.set(Some(AiError::MissingKey));
        return;
    }
    error.set(None);
    let mut list = history();
    list.push(ChatTurn {
        role: Role::User,
        text,
    });
    history.set(list.clone());
    draft.set(String::new());
    set_field(QUESTION_ID, "", Some(&raw));
    sending.set(true);

    let chat = active();
    spawn(async move {
        // Pesan pengguna disimpan dulu: chat muncul di riwayat segera, dan
        // tidak hilang kalau permintaan gagal atau halaman ditutup.
        let saved =
            Conversation::assemble(&chat.id, chat.created_at, &model, list.clone(), now_ms());
        persist(saved, chats, store_ok).await;

        match call_gemini(api_key, model.clone(), list.clone(), s).await {
            Ok(reply) => {
                list.push(ChatTurn {
                    role: Role::Assistant,
                    text: reply,
                });
                history.set(list.clone());
                let saved =
                    Conversation::assemble(&chat.id, chat.created_at, &model, list, now_ms());
                persist(saved, chats, store_ok).await;
            }
            Err(e) => error.set(Some(e)),
        }
        sending.set(false);
    });
}

/// Mulai percakapan kosong. Yang lama sudah tersimpan, jadi tidak ada yang hilang.
fn start_new_chat(mut state: ChatState) {
    state.history.set(vec![]);
    state.error.set(None);
    state.active.set(ActiveChat::fresh());
}

/// Membuka chat dari riwayat. Selama permintaan berjalan, jawabannya akan masuk
/// ke chat yang sedang tampil, jadi berpindah chat ditolak sampai selesai.
fn open_chat(
    state: ChatState,
    id: String,
    mut model: Signal<&'static str>,
    mut show_history: Signal<bool>,
) {
    if (state.sending)() {
        return;
    }
    spawn(async move {
        let mut state = state;
        match ai_history::load(&id).await {
            Ok(Some(chat)) => {
                if let Some((known, _)) = MODEL_OPTIONS.iter().find(|(m, _)| *m == chat.model) {
                    model.set(known);
                }
                state.history.set(chat.turns);
                state.active.set(ActiveChat {
                    id: chat.id,
                    created_at: chat.created_at,
                });
                state.error.set(None);
                state.store_ok.set(true);
            }
            // Sudah dihapus (mis. dari tab lain): buang dari daftar.
            Ok(None) => state.chats.with_mut(|list| list.retain(|c| c.id != id)),
            Err(_) => state.store_ok.set(false),
        }
        show_history.set(false);
    });
}

fn remove_chat(state: ChatState, id: String) {
    spawn(async move {
        let mut state = state;
        match ai_history::delete(std::slice::from_ref(&id)).await {
            Ok(()) => {
                state.chats.with_mut(|list| list.retain(|c| c.id != id));
                if (state.active)().id == id {
                    start_new_chat(state);
                }
            }
            Err(_) => state.store_ok.set(false),
        }
    });
}

/// Muat ulang daftar dari IndexedDB (saat jendela dibuka dan saat panel dibuka,
/// jadi perubahan dari tab lain ikut terlihat).
fn reload_list(state: ChatState) {
    spawn(async move {
        let mut state = state;
        match ai_history::list().await {
            Ok(list) => {
                state.chats.set(list);
                state.store_ok.set(true);
            }
            Err(_) => state.store_ok.set(false),
        }
    });
}

/// Unduh satu chat (`Some(id)`) atau semuanya (`None`) sebagai berkas JSON.
fn download_chats(state: ChatState, id: Option<String>) {
    spawn(async move {
        let mut state = state;
        let now = now_ms();
        match id {
            Some(id) => match ai_history::load(&id).await {
                Ok(Some(chat)) => {
                    let name = export_filename(Some(&chat), now);
                    ai_history::download_json(&name, &export_json(&[chat], now));
                }
                Ok(None) => {}
                Err(_) => state.store_ok.set(false),
            },
            None => match ai_history::load_all().await {
                Ok(all) => {
                    let name = export_filename(None, now);
                    ai_history::download_json(&name, &export_json(&all, now));
                }
                Err(_) => state.store_ok.set(false),
            },
        }
    });
}

/// `YYYY-MM-DD` menurut zona waktu pengguna.
fn day_label(ms: i64) -> String {
    let date = Date::from_iso(&ai_history::iso_from_ms(ms));
    format!(
        "{}-{:02}-{:02}",
        date.get_full_year(),
        date.get_month() + 1,
        date.get_date()
    )
}

/// Skrip yang memasang dan menjalankan penyesuaian tinggi textarea.
fn autosize_script() -> String {
    format!("const ID = \"{QUESTION_ID}\"; const MAX_ROWS = {QUESTION_MAX_ROWS};\n{AUTOSIZE_JS}")
}

/// Splits a reply on ``` fences so code blocks render in a monospace box
/// distinct from prose — a light touch, not real syntax highlighting.
/// Returns `(is_code, content)` pairs rather than an enum so the render
/// loop can branch with a plain `if/else` instead of `match` inside rsx.
fn parse_segments(text: &str) -> Vec<(bool, String)> {
    let mut segments = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("```") {
        if start > 0 {
            segments.push((false, rest[..start].to_string()));
        }
        let after_fence = &rest[start + 3..];
        let code_start = after_fence.find('\n').map(|i| i + 1).unwrap_or(0);
        let code_region = &after_fence[code_start..];
        match code_region.find("```") {
            Some(end) => {
                segments.push((true, code_region[..end].to_string()));
                rest = &code_region[end + 3..];
            }
            None => {
                segments.push((true, code_region.to_string()));
                rest = "";
                break;
            }
        }
    }
    if !rest.is_empty() {
        segments.push((false, rest.to_string()));
    }
    segments
}

/// Line numbers for the code gutter, one per line, joined with newlines
/// so a single `whitespace-pre` element can render them.
fn code_gutter(code: &str) -> String {
    let lines = code.lines().count().max(1);
    (1..=lines)
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn model_option_class(active: bool) -> &'static str {
    if active {
        "ai__model-option ai__model-option--on"
    } else {
        "ai__model-option ai__model-option--off"
    }
}

#[component]
pub fn AiAssistantWindowContent() -> Element {
    let s = use_strings();
    let state = ChatState {
        history: use_signal(Vec::new),
        draft: use_signal(String::new),
        sending: use_signal(|| false),
        error: use_signal(|| None),
        active: use_signal(ActiveChat::fresh),
        chats: use_signal(Vec::new),
        store_ok: use_signal(|| true),
    };
    let mut draft = state.draft;
    let mut api_key = use_signal(String::new);
    let mut show_key = use_signal(|| false);
    let mut show_models = use_signal(|| false);
    let mut model = use_signal(|| GEMINI_36_FLASH);
    let mut show_history = use_signal(|| false);
    // Id chat yang sedang diminta konfirmasi hapusnya (klik kedua menghapus).
    let mut confirm_delete: Signal<Option<String>> = use_signal(|| None);

    // Riwayat dimuat sekali saat jendela dibuka.
    use_future(move || async move {
        reload_list(state);
    });

    // Textarea: pasang penyesuaian tinggi saat jendela dibuka, dan hitung ulang
    // setiap isinya dikosongkan dari kode (mengubah `value` tidak memicu
    // `input`). Membaca `draft` membuat efek ini berjalan tiap isinya berubah,
    // tetapi skripnya hanya dikirim saat kosong; selama mengetik, pendengar
    // `input` di browser yang bekerja.
    use_effect(move || {
        if draft().is_empty() {
            eval_js(autosize_script());
        }
    });

    let turns = (state.history)();
    let sending = (state.sending)();
    let chats = (state.chats)();
    let store_ok = (state.store_ok)();
    let active_id = (state.active)().id;
    let key_input_type = if show_key() { "text" } else { "password" };
    let eye_label = if show_key() {
        s.ai.hide_key
    } else {
        s.ai.show_key
    };
    let models_expanded = if show_models() { "true" } else { "false" };
    let history_open = show_history();
    let history_expanded = if history_open { "true" } else { "false" };
    let history_toggle_aria = if history_open {
        s.ai.history_close_aria
    } else {
        s.ai.history_toggle_aria
    };
    let error = (state.error)();
    let has_error = error.is_some();
    let error_message = error.map(|e| e.message(s)).unwrap_or_default();
    let confirming = confirm_delete();

    rsx! {
        div { class: OUTER_CLASS,
        div { class: ROOT_CLASS,
            // Tombol riwayat chat: pojok kiri atas jendela (absolute di `ai__root`).
            button {
                r#type: "button",
                "aria-label": history_toggle_aria,
                "aria-expanded": "{history_expanded}",
                "aria-controls": HISTORY_PANEL_ID,
                title: s.ai.history_toggle_title,
                class: HISTORY_TOGGLE_CLASS,
                onclick: move |_| {
                    let open = !show_history();
                    show_history.set(open);
                    confirm_delete.set(None);
                    if open {
                        reload_list(state);
                    }
                },
                img { src: ICON_SIDEBAR, alt: "", "aria-hidden": "true" }
            }

            // conversation — flex-col-reverse, so the DOM runs bottom to top:
            // the status indicators come first, then turns from newest to oldest
            section {
                "aria-label": s.ai.conversation,
                class: CONVERSATION_CLASS,

                if sending {
                    p { class: "ai__muted", "{s.ai.thinking}" }
                }
                if has_error {
                    p { role: "alert", class: "ai__error", "{error_message}" }
                }

                for (idx, turn) in turns.iter().enumerate().rev() {
                    {
                        let key = format!("turn-{idx}");
                        if turn.role == Role::User {
                            let user_text = turn.text.clone();
                            rsx! {
                                article {
                                    key: "{key}",
                                    class: USER_BUBBLE_CLASS,
                                    p { class: USER_TEXT_CLASS, "{user_text}" }
                                }
                            }
                        } else {
                            let segments = parse_segments(&turn.text);
                            rsx! {
                                article {
                                    key: "{key}",
                                    class: ASSISTANT_ARTICLE_CLASS,
                                    img {
                                        class: ASSISTANT_ICON_CLASS,
                                        src: ICON_ASSISTANT,
                                        alt: "",
                                        "aria-hidden": "true",
                                    }
                                    div {
                                        class: ASSISTANT_BODY_CLASS,
                                        for (j, segment) in segments.iter().enumerate() {
                                            {
                                                let is_code = segment.0;
                                                let code = segment.1.trim_end().to_string();
                                                let gutter = code_gutter(&code);
                                                let text = segment.1.trim().to_string();
                                                rsx! {
                                                    if is_code {
                                                        figure {
                                                            key: "code-{j}",
                                                            class: CODE_FIGURE_CLASS,
                                                            figcaption { class: "visually-hidden", "{s.ai.code_block}" }
                                                            div {
                                                                class: CODE_GUTTER_CLASS,
                                                                "aria-hidden": "true",
                                                                div { class: CODE_NUMBERS_CLASS, "{gutter}" }
                                                            }
                                                            pre {
                                                                class: CODE_PRE_CLASS,
                                                                code { "{code}" }
                                                            }
                                                        }
                                                    } else {
                                                        p {
                                                            key: "text-{j}",
                                                            class: ASSISTANT_TEXT_CLASS,
                                                            "{text}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if turns.is_empty() {
                    p {
                        class: "ai__muted",
                        "{s.ai.empty_prompt}"
                    }
                }
            }

            // footer: question form + API key row
            footer {
                class: FOOTER_CLASS,

                form {
                    "aria-label": s.ai.form_label,
                    class: FORM_CLASS,
                    onsubmit: move |evt: FormEvent| {
                        evt.prevent_default();
                        send_prompt(state, api_key(), model().to_string(), s);
                    },

                    label {
                        r#for: QUESTION_ID,
                        class: "visually-hidden",
                        "{s.ai.question_prompt}"
                    }
                    // Enter mengirim, Shift+Enter baris baru. Tinggi: 1 sampai 5 baris.
                    textarea {
                        id: QUESTION_ID,
                        name: "question",
                        rows: 1,
                        autocomplete: "off",
                        "aria-describedby": "assistant-question-hint",
                        placeholder: s.ai.question_prompt,
                        class: QUESTION_INPUT_CLASS,
                        // Tanpa `value`: DOM pemilik teks, lihat `js_util::set_field`.
                        oninput: move |evt: FormEvent| draft.set(evt.value()),
                        onkeydown: move |evt: KeyboardEvent| {
                            if evt.key() == Key::Enter && !evt.modifiers().shift() && !evt.is_composing() {
                                evt.prevent_default();
                                send_prompt(state, api_key(), model().to_string(), s);
                            }
                        },
                    }
                    p {
                        id: "assistant-question-hint",
                        class: "visually-hidden",
                        "{s.ai.question_hint}"
                    }

                    button {
                        r#type: "submit",
                        "aria-label": s.ai.send_question,
                        class: ICON_BUTTON_CLASS,
                        img { src: ICON_SEND, alt: "", "aria-hidden": "true" }
                    }
                }

                div {
                    class: KEY_ROW_CLASS,

                    div {
                        class: KEY_GROUP_CLASS,
                        label {
                            r#for: "api-key",
                            class: KEY_LABEL_CLASS,
                            "{s.ai.api_key_label}"
                        }

                        div {
                            class: KEY_FIELD_CLASS,
                            div {
                                class: "ai__center",
                                input {
                                    id: "api-key",
                                    "aria-describedby": "api-key-help",
                                    placeholder: "AIza...",
                                    r#type: key_input_type,
                                    class: KEY_INPUT_CLASS,
                                    oninput: move |evt: FormEvent| api_key.set(evt.value()),
                                }
                            }
                            button {
                                r#type: "button",
                                class: SHOW_BUTTON_CLASS,
                                onclick: move |_| show_key.set(!show_key()),
                                "{eye_label}"
                            }
                            a {
                                r#type: "button",
                                "aria-label": s.ai.api_key_info,
                                target: "_blank",
                                href: "https://aistudio.google.com/app/api-keys",
                                title: s.ai.api_key_tooltip,
                                class: INFO_BUTTON_CLASS,
                                img {
                                    src: ICON_INFO,
                                    alt: "",
                                    "aria-hidden": "true",
                                    class: "ai__img-fill",
                                }
                            }
                        }
                    }

                    // settings toggle → opens the model picker
                    div {
                        class: SETTINGS_WRAP_CLASS,
                        button {
                            r#type: "button",
                            "aria-label": s.ai.open_settings,
                            "aria-expanded": "{models_expanded}",
                            class: SETTINGS_TOGGLE_CLASS,
                            onclick: move |_| show_models.set(!show_models()),
                            img {
                                src: ICON_TOGGLE,
                                alt: "",
                                "aria-hidden": "true",
                                class: "ai__img-square",
                            }
                        }

                        if show_models() {
                            div {
                                class: MODEL_MENU_CLASS,
                                for (id, label) in MODEL_OPTIONS {
                                    button {
                                        key: "{id}",
                                        r#type: "button",
                                        class: model_option_class(model() == id),
                                        onclick: move |_| {
                                            model.set(id);
                                            show_models.set(false);
                                        },
                                        "{label}"
                                    }
                                }
                            }
                        }
                    }
                }

                p {
                    id: "api-key-help",
                    class: "visually-hidden",
                    "{s.ai.api_key_help}"
                }
            }
        }

        // Panel riwayat: menempel di dalam jendela (absolute di `OUTER_CLASS`),
        // bukan di viewport. Selalu dirender supaya buka DAN tutupnya beranimasi
        // (`side_panel.rs`); klik di luar panel atau Esc menutupnya.
        SidePanel {
            id: HISTORY_PANEL_ID,
            label: s.ai.history_panel_label,
            side: Side::Left,
            open: history_open,
            class: PANEL_CLASS,
            on_close: move |_| {
                show_history.set(false);
                confirm_delete.set(None);
            },
            div {
                class: PANEL_HEADER_CLASS,
                h2 { class: PANEL_TITLE_CLASS, "{s.ai.history_title}" }
                button {
                    r#type: "button",
                    "aria-label": s.ai.history_close_aria,
                    class: ICON_BUTTON_CLASS,
                    onclick: move |_| {
                        show_history.set(false);
                        confirm_delete.set(None);
                    },
                    img { src: ICON_CLOSE, alt: "", "aria-hidden": "true", class: "ai__img-xs" }
                }
            }

            button {
                r#type: "button",
                class: NEW_CHAT_CLASS,
                disabled: sending,
                onclick: move |_| {
                    start_new_chat(state);
                    show_history.set(false);
                    confirm_delete.set(None);
                },
                img { src: ICON_ADD, alt: "", "aria-hidden": "true", class: "ai__img-sm" }
                span { "{s.ai.new_chat}" }
            }

            if !store_ok {
                p { class: NOTE_CLASS, role: "alert", "{s.ai.history_unavailable}" }
            }

            ul {
                class: LIST_CLASS,
                if chats.is_empty() && store_ok {
                    li { class: NOTE_CLASS, "{s.ai.history_empty}" }
                }
                for chat in chats.iter() {
                    {
                        let is_active = chat.id == active_id;
                        let title = if chat.title.is_empty() {
                            s.ai.untitled_chat.to_string()
                        } else {
                            chat.title.clone()
                        };
                        let date = day_label(chat.updated_at);
                        let asking = confirming.as_deref() == Some(chat.id.as_str());
                        let open_id = chat.id.clone();
                        let download_id = chat.id.clone();
                        let delete_id = chat.id.clone();
                        let download_aria = s.ai.download_chat_aria(&title);
                        let delete_aria = s.ai.delete_chat_aria(&title);
                        let delete_label = if asking {
                            s.ai.delete_chat_confirm
                        } else {
                            s.ai.delete_chat
                        };
                        rsx! {
                            li {
                                key: "{chat.id}",
                                class: if is_active { ROW_ACTIVE_CLASS } else { ROW_CLASS },
                                button {
                                    r#type: "button",
                                    class: ROW_OPEN_CLASS,
                                    disabled: sending,
                                    "aria-current": if is_active { "true" } else { "false" },
                                    onclick: move |_| {
                                        confirm_delete.set(None);
                                        open_chat(state, open_id.clone(), model, show_history);
                                    },
                                    span { class: ROW_TITLE_CLASS, "{title}" }
                                    span { class: ROW_DATE_CLASS, "{date}" }
                                }
                                div {
                                    class: ROW_ACTIONS_CLASS,
                                    button {
                                        r#type: "button",
                                        class: TEXT_BUTTON_CLASS,
                                        "aria-label": download_aria,
                                        onclick: move |_| download_chats(state, Some(download_id.clone())),
                                        "{s.ai.download_chat}"
                                    }
                                    button {
                                        r#type: "button",
                                        class: TEXT_BUTTON_CLASS,
                                        "aria-label": delete_aria,
                                        // Chat yang sedang menunggu jawaban tidak boleh dihapus.
                                        disabled: sending && is_active,
                                        onclick: move |_| {
                                            if confirm_delete().as_deref() == Some(delete_id.as_str()) {
                                                confirm_delete.set(None);
                                                remove_chat(state, delete_id.clone());
                                            } else {
                                                confirm_delete.set(Some(delete_id.clone()));
                                            }
                                        },
                                        "{delete_label}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            button {
                r#type: "button",
                class: NEW_CHAT_CLASS,
                disabled: chats.is_empty(),
                onclick: move |_| download_chats(state, None),
                "{s.ai.download_all}"
            }
            p { class: NOTE_CLASS, "{s.ai.history_note}" }
        }
        }
    }
}
