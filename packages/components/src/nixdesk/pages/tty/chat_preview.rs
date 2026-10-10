use std::sync::OnceLock;

use crate::platform::TimeoutFuture;
use crate::time::relative_time;
use dioxus::prelude::*;
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
    pub timestamp: String,
}

/// Parsed once per session. `use_chat_cycle` and `ChatPreviewPanel` call this
/// on every render (and the cycle re-renders every `CYCLE_MS`), so parsing
/// the JSON each time was pure waste.
fn load_messages() -> &'static [ChatMessage] {
    static MESSAGES: OnceLock<Vec<ChatMessage>> = OnceLock::new();
    MESSAGES
        .get_or_init(|| serde_json::from_str(CHAT_JSON).unwrap_or_default())
        .as_slice()
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
    let username = current
        .as_ref()
        .map(|m| m.username.clone())
        .unwrap_or_default();
    let relative = current
        .as_ref()
        .map(|m| relative_time(&m.timestamp))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_messages_parse_and_are_cached() {
        let a = load_messages();
        assert!(
            !a.is_empty(),
            "chat_sample.json failed to parse or is empty"
        );
        // OnceLock: the same allocation every call, not a re-parse.
        assert!(std::ptr::eq(a.as_ptr(), load_messages().as_ptr()));
    }
}
