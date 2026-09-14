use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use js_sys::Date;
use wasm_bindgen::JsValue;
use serde::Deserialize;

const CHAT_JSON: &str = include_str!("../../../../data/chat_sample.json");
const CYCLE_MS: u32 = 3000;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub message_id: u32,
    pub username: String,
    #[allow(dead_code)]
    pub img_profile: Option<String>,
    pub message: String,
    // NOTE: the source data file spells this field "timestanp" (typo for
    // "timestamp") — kept as-is so serde matches the real key.
    pub timestanp: String,
}

fn load_messages() -> Vec<ChatMessage> {
    serde_json::from_str(CHAT_JSON).unwrap_or_default()
}

/// "3 minutes ago" / "2 hours ago" / "5 days ago" / "2 weeks ago" /
/// "1 month ago" / "2 years ago" — bucketed against the real current time.
fn relative_time(iso: &str) -> String {
    let then = Date::new(&JsValue::from_str(iso)).get_time();
    let now = Date::now();
    let diff_secs = ((now - then) / 1000.0).max(0.0);

    let minutes = diff_secs / 60.0;
    let hours = minutes / 60.0;
    let days = hours / 24.0;
    let weeks = days / 7.0;
    let months = days / 30.0;
    let years = days / 365.0;

    fn plural(n: u32) -> &'static str {
        if n == 1 { "" } else { "s" }
    }

    if minutes < 1.0 {
        "just now".to_string()
    } else if minutes < 60.0 {
        let n = minutes as u32;
        format!("{n} minute{} ago", plural(n))
    } else if hours < 24.0 {
        let n = hours as u32;
        format!("{n} hour{} ago", plural(n))
    } else if days < 7.0 {
        let n = days as u32;
        format!("{n} day{} ago", plural(n))
    } else if weeks < 5.0 {
        let n = weeks as u32;
        format!("{n} week{} ago", plural(n))
    } else if months < 12.0 {
        let n = months as u32;
        format!("{n} month{} ago", plural(n))
    } else {
        let n = years as u32;
        format!("{n} year{} ago", plural(n))
    }
}

/// Cycles the active message index every 3s — mirrors the "shell 4 - chat
/// preview transition interval 3s" layer from the Terminal Figma frame.
/// Called once from `TerminalMode` so the desktop panel and the mobile
/// "SHELL 4" tab stay in sync instead of running independent timers.
pub fn use_chat_cycle() -> Signal<usize> {
    let messages_len = load_messages().len().max(1);
    let mut index = use_signal(|| 0usize);

    use_effect(move || {
        spawn(async move {
            loop {
                TimeoutFuture::new(CYCLE_MS).await;
                index.set((index() + 1) % messages_len);
            }
        });
    });

    index
}

/// "shell 4" — the auto-cycling Live Chat preview.
#[component]
pub fn ChatPreviewPanel(index: Signal<usize>) -> Element {
    let messages = load_messages();
    let total = messages.len();
    let current = if total > 0 {
        Some(messages[index() % total].clone())
    } else {
        None
    };

    let has_message = current.is_some();
    let msg_id = current.as_ref().map(|m| m.message_id).unwrap_or(0);
    let username = current.as_ref().map(|m| m.username.clone()).unwrap_or_default();
    let relative = current
        .as_ref()
        .map(|m| relative_time(&m.timestanp))
        .unwrap_or_default();
    let message_text = current
        .as_ref()
        .map(|m| m.message.clone())
        .unwrap_or_else(|| "No messages yet.".to_string());

    rsx! {
        div { class: "flex flex-col h-full justify-center gap-2 border border-white/15 px-5 py-6 overflow-hidden",
            if has_message {
                div {
                    key: "{msg_id}",
                    class: "chat-cycle-in flex flex-col gap-1.5",
                    div { class: "flex items-baseline justify-between gap-3",
                        span { class: "text-sm font-semibold text-white truncate", "{username}" }
                        span { class: "text-[11px] text-white/40 whitespace-nowrap", "{relative}" }
                    }
                    p { class: "text-sm text-white/70 leading-snug line-clamp-3", "{message_text}" }
                }
            } else {
                p { class: "text-white/30 text-sm", "No messages yet." }
            }
        }
    }
}
