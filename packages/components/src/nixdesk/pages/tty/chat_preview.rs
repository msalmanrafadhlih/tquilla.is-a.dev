use crate::api::{self, ChatMessage, Page};
use crate::guestbook::tab_hidden;
use crate::languages::use_strings;
use crate::markdown::plain_text;
use crate::platform::TimeoutFuture;
use crate::time::relative_time;
use dioxus::prelude::*;

/// Time each message stays on screen.
const CYCLE_MS: u32 = 3000;
/// The preview shows plain text, cut here (CSS clamps the lines as well).
const PREVIEW_CHARS: usize = 240;
/// How many of the latest messages the preview rotates through.
const PREVIEW_COUNT: u32 = 10;
/// The preview is decoration, not the guestbook: it looks for news every 2 minutes
/// (never while the tab is in the background).
const REFRESH_MS: u32 = 120_000;

/// The messages the preview rotates through and which one is showing.
/// Created once in `TerminalMode` and shared by the desktop panel and the
/// mobile "SHELL 4" tab, so they stay in sync and the API is asked once.
#[derive(Clone, Copy, PartialEq)]
pub struct ChatFeed {
    index: Signal<usize>,
    messages: Signal<Vec<ChatMessage>>,
}

/// Loads the latest messages (a failure just leaves the panel on its "no
/// entries" text) and cycles the active index every 3s — mirrors the "shell
/// 4 - chat preview transition interval 3s" layer from the Terminal Figma frame.
pub fn use_chat_feed() -> ChatFeed {
    let mut index = use_signal(|| 0usize);
    let mut messages = use_signal(Vec::<ChatMessage>::new);

    use_future(move || async move {
        let mut first = true;
        loop {
            if first || !tab_hidden().await {
                if let Ok(latest) = api::messages(Page::Latest, Some(PREVIEW_COUNT)).await {
                    // Newest first looks right for a ticker.
                    let mut latest = latest;
                    latest.reverse();
                    messages.set(latest);
                }
            }
            first = false;
            TimeoutFuture::new(REFRESH_MS).await;
        }
    });

    use_future(move || async move {
        loop {
            TimeoutFuture::new(CYCLE_MS).await;
            let len = messages.peek().len().max(1);
            index.set((index() + 1) % len);
        }
    });

    ChatFeed { index, messages }
}

/// The message to show for `index`, wrapping around; `None` when there are none.
fn pick(messages: &[ChatMessage], index: usize) -> Option<&ChatMessage> {
    if messages.is_empty() {
        None
    } else {
        messages.get(index % messages.len())
    }
}

/// "shell 4" — the auto-cycling Guestbook preview.
#[component]
pub fn ChatPreviewPanel(feed: ChatFeed) -> Element {
    let s = use_strings();
    let list = (feed.messages)();
    let current = pick(&list, (feed.index)());

    rsx! {
        div { class: "flex flex-col h-full justify-center gap-2 border border-white/15 px-5 py-6 overflow-hidden",
            if let Some(m) = current {
                div { key: "{m.id}", class: "chat-cycle-in flex flex-col gap-1.5",
                    div { class: "flex items-baseline justify-between gap-3",
                        span { class: "text-sm font-semibold text-white truncate", "{m.username}" }
                        span { class: "text-[11px] text-white/40 whitespace-nowrap",
                            "{relative_time(&m.created_at, s)}"
                        }
                    }
                    p { class: "text-sm text-white/70 leading-snug line-clamp-3", "{plain_text(&m.message, PREVIEW_CHARS)}" }
                }
            } else {
                p { class: "text-white/30 text-sm", "{s.terminal.no_entries}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(id: u32) -> ChatMessage {
        ChatMessage {
            id,
            username: "u".into(),
            url: None,
            message: "m".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn pick_wraps_around_and_handles_an_empty_list() {
        assert!(pick(&[], 0).is_none());
        assert!(pick(&[], 7).is_none());
        let list = [msg(1), msg(2), msg(3)];
        assert_eq!(pick(&list, 0).map(|m| m.id), Some(1));
        assert_eq!(pick(&list, 2).map(|m| m.id), Some(3));
        assert_eq!(pick(&list, 3).map(|m| m.id), Some(1));
        assert_eq!(pick(&list, 100).map(|m| m.id), Some(2));
    }
}
