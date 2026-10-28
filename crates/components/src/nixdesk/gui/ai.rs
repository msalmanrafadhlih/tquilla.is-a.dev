use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use super::ai_history::{
    self, export_filename, export_json, fresh_id, ChatTurn, Conversation, Role, Summary,
};
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
const OUTER_CLASS: &str = "relative w-full h-full overflow-hidden";
const SCRIM_CLASS: &str = "absolute inset-0 z-10 bg-black/40";
const PANEL_CLASS: &str = "absolute inset-y-0 left-0 z-20 flex w-[min(18rem,85%)] flex-col gap-2 p-2.5 border-r border-solid border-[var(--fg-secondary)] bg-[var(--bg-secondary)]";
const PANEL_HEADER_CLASS: &str = "flex items-center justify-between gap-2";
const PANEL_TITLE_CLASS: &str =
    "m-0 text-xs font-normal text-[var(--fg-main)] [font:'JetBrains_Mono-Regular',Helvetica]";
const NEW_CHAT_CLASS: &str = "flex items-center gap-2 px-2 py-1.5 text-xs text-[var(--fg-main)] bg-[var(--bg-secondary)] border border-solid border-[var(--fg-secondary)] cursor-pointer hover:border-[var(--fg-main)] disabled:opacity-40 disabled:cursor-default";
const NOTE_CLASS: &str = "m-0 text-[10px] text-[var(--fg-secondary)] leading-snug";
const LIST_CLASS: &str = "m-0 p-0 list-none flex flex-col gap-1 flex-1 min-h-0 overflow-y-auto";
const ROW_CLASS: &str = "flex flex-col gap-0.5 p-1.5 border border-solid border-transparent";
const ROW_ACTIVE_CLASS: &str =
    "flex flex-col gap-0.5 p-1.5 border border-solid border-[var(--fg-secondary)]";
const ROW_OPEN_CLASS: &str = "flex flex-col items-start gap-0.5 min-w-0 p-0 text-left bg-transparent border-0 cursor-pointer disabled:cursor-default";
const ROW_TITLE_CLASS: &str = "w-full truncate text-xs text-[var(--fg-main)]";
const ROW_DATE_CLASS: &str = "text-[10px] text-[var(--fg-secondary)]";
const ROW_ACTIONS_CLASS: &str = "flex items-center gap-3";
const TEXT_BUTTON_CLASS: &str = "p-0 text-[10px] text-[var(--fg-secondary)] hover:text-[var(--fg-main)] bg-transparent border-0 cursor-pointer disabled:opacity-40 disabled:cursor-default";

const ROOT_CLASS: &str =
    "flex flex-col items-center justify-center gap-[25px] relative w-full h-full overflow-y-scroll";
const CONVERSATION_CLASS: &str = "flex flex-col-reverse items-end gap-[25px] relative flex-1 w-full max-w-[48rem] h-full grow overflow-y-scroll";
const USER_BUBBLE_CLASS: &str = "flex flex-col rounded-b-xl rounded-tl-xl items-start w-full max-w-max p-5 relative flex-[0_0_auto] bg-[var(--fg-main)]";
const USER_TEXT_CLASS: &str = "relative self-stretch whitespace-pre-wrap [font:'JetBrains_Mono-Regular',Helvetica] text-[var(--bg-secondary)] text-xs text-left tracking-[0] leading-[normal]";
const ASSISTANT_ARTICLE_CLASS: &str = "flex items-start gap-[var(--size-space-200)] relative self-stretch w-full flex-[0_0_auto] rounded-lg overflow-hidden";
const ASSISTANT_ICON_CLASS: &str = "relative w-5 h-5 aspect-[1]";
const ASSISTANT_BODY_CLASS: &str = "flex flex-col gap-3 flex-1 min-w-0";
const ASSISTANT_TEXT_CLASS: &str = "relative whitespace-pre-wrap [font:'JetBrains_Mono-Regular',Helvetica] text-[var(--fg-main)] text-xs tracking-[0] leading-[normal]";
const CODE_FIGURE_CLASS: &str = "flex items-start bg-white relative self-stretch w-full flex-[0_0_auto] rounded-xl overflow-hidden border border-solid border-color-border-default-default";
const CODE_GUTTER_CLASS: &str = "flex flex-col w-12 items-end pt-[var(--size-space-400)] pr-[var(--size-space-200)] pb-[var(--size-space-400)] pl-[var(--size-space-200)] relative self-stretch bg-color-background-default-default border-r [border-right-style:solid] border-color-border-default-default";
const CODE_NUMBERS_CLASS: &str = "relative w-fit mt-[-1.00px] whitespace-pre font-body-code font-[number:var(--body-code-font-weight)] text-color-text-default-tertiary text-[length:var(--body-code-font-size)] text-right tracking-[var(--body-code-letter-spacing)] leading-[var(--body-code-line-height)] [font-style:var(--body-code-font-style)]";
const CODE_PRE_CLASS: &str = "inline-flex flex-col items-start pt-[var(--size-space-400)] pr-[var(--size-space-200)] pb-[var(--size-space-400)] pl-[var(--size-space-200)] relative flex-1 min-w-0 overflow-auto bg-color-background-default-secondary border-r [border-right-style:solid] border-color-border-default-default whitespace-pre font-body-code font-[number:var(--body-code-font-weight)] text-[length:var(--body-code-font-size)] tracking-[var(--body-code-letter-spacing)] leading-[var(--body-code-line-height)] [font-style:var(--body-code-font-style)]";
const FOOTER_CLASS: &str = "flex flex-col items-start relative w-full max-w-[48rem] flex-[0_0_auto] border border-solid border-[var(--fg-secondary)]";
const FORM_CLASS: &str =
    "flex items-end gap-5 p-2.5 relative self-stretch w-full h-max flex-[0_0_auto]";
// Tinggi diatur `js/autosize.js` (1 sampai 5 baris); padding 3px + 19.6px baris
// = 25.6px, setara tombol ikon di sampingnya.
const QUESTION_INPUT_CLASS: &str = "relative flex-1 min-w-0 min-h-[25px] py-[3px] resize-none overflow-y-hidden bg-[var(--bg-secondary)] border-0 outline-none [font:'JetBrains_Mono-Light',Helvetica] font-light text-[var(--fg-main)] text-sm tracking-[0] leading-[19.6px]";
const ICON_BUTTON_CLASS: &str = "flex relative h-max opacity-70 hover:opacity-100 aspect-[1] bg-[var(--bg-secondary)] border-0 items-center justify-center cursor-pointer";
const KEY_ROW_CLASS: &str =
    "w-full h-12 flex items-center justify-center p-2.5 relative max-w-full";
const KEY_GROUP_CLASS: &str = "flex items-center gap-2.5 relative flex-1 self-stretch grow";
const KEY_LABEL_CLASS: &str = "relative flex text-left w-fit [font:'JetBrains_Mono-Thin',Helvetica] font-thin text-[var(--fg-secondary)] text-xs text-center tracking-[0] leading-[normal] whitespace-nowrap";
const KEY_FIELD_CLASS: &str =
    "w-full flex items-center gap-[5px] p-[5px] relative flex-1 self-stretch grow";
const KEY_INPUT_CLASS: &str = "flex-1 min-w-0 w-full text-xs bg-[var(--bg-secondary)] outline-none border-0 border-b border-[var(--fg-secondary)] text-[var(--fg-main)] placeholder-[var(--fg-secondary)] opacity-50 focus:opacity-100 transition-opacity duration-300";
const SHOW_BUTTON_CLASS: &str = "text-[var(--fg-secondary)] hover:text-[var(--fg-main)] transition-colors duration-150 text-xs bg-[var(--bg-secondary)] border-0 cursor-pointer";
const INFO_BUTTON_CLASS: &str = "h-full relative aspect-[1] border-0 bg-[var(--bg-secondary)] p-[1px] opacity-70 hover:opacity-100 cursor-pointer";
const SETTINGS_WRAP_CLASS: &str = "relative flex-[0_0_auto] h-full";
const SETTINGS_TOGGLE_CLASS: &str = "inline-flex h-full p-[5px] items-center relative bg-[var(--bg-secondary)] border-0 opacity-70 hover:opacity-100 cursor-pointer";
const MODEL_MENU_CLASS: &str = "absolute bottom-full right-0 mb-2 flex flex-col gap-1 p-1 border border-solid border-[var(--fg-secondary)] bg-[var(--bg-secondary)]";

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
        "px-2 py-1 text-left whitespace-nowrap text-[10px] bg-[var(--fg-main)] text-black border-0"
    } else {
        "px-2 py-1 text-left whitespace-nowrap text-[10px] text-[var(--fg-secondary)] hover:text-[var(--fg-main)] bg-[var(--bg-secondary)] border-0"
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

            // conversation — flex-col-reverse, so the DOM runs bottom to top:
            // the status indicators come first, then turns from newest to oldest
            section {
                "aria-label": s.ai.conversation,
                class: CONVERSATION_CLASS,

                if sending {
                    p { class: "text-[var(--fg-secondary)] text-xs", "{s.ai.thinking}" }
                }
                if has_error {
                    p { role: "alert", class: "text-[var(--red)] text-xs [overflow-wrap:anywhere]", "{error_message}" }
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
                                                            figcaption { class: "sr-only", "{s.ai.code_block}" }
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
                        class: "text-[var(--fg-secondary)] text-xs",
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

                    // Membuka dan menutup panel riwayat chat.
                    button {
                        r#type: "button",
                        "aria-label": history_toggle_aria,
                        "aria-expanded": "{history_expanded}",
                        "aria-controls": "ai-history-panel",
                        title: s.ai.history_toggle_title,
                        class: ICON_BUTTON_CLASS,
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

                    label {
                        r#for: QUESTION_ID,
                        class: "sr-only",
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
                        class: "sr-only",
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
                                class: "flex items-center justify-center relative flex-1 grow",
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
                                    class: "h-full",
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
                                class: "relative aspect-[1] h-full",
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
                    class: "sr-only",
                    "{s.ai.api_key_help}"
                }
            }
        }

        // Panel riwayat: menempel di dalam jendela (absolute di `OUTER_CLASS`),
        // bukan di viewport. Klik di luar panel menutupnya.
        if history_open {
            div {
                class: SCRIM_CLASS,
                onclick: move |_| {
                    show_history.set(false);
                    confirm_delete.set(None);
                },
            }
            aside {
                id: "ai-history-panel",
                "aria-label": s.ai.history_panel_label,
                class: PANEL_CLASS,

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
                        img { src: ICON_CLOSE, alt: "", "aria-hidden": "true", class: "h-3 w-3" }
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
                    img { src: ICON_ADD, alt: "", "aria-hidden": "true", class: "h-4 w-4" }
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
}
