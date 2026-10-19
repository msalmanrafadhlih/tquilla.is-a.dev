//! Logic of the Guestbook window and the Terminal preview, kept free of UI so
//! it can be unit-tested without a browser:
//!
//! - how pages of entries are merged into the list on screen;
//! - the Write/Preview editor (what the preview shows, how many characters
//!   are left) and its toolbar script;
//! - turning a local validation failure into text in the current language.
//!
//! The guestbook does not poll. Entries are loaded when the window opens,
//! refreshed by a button (which only asks for entries newer than the newest
//! one on screen), and older pages load on demand. Nothing runs while nobody
//! is looking, which also keeps the Worker free plan (100k requests a day)
//! out of the picture.

use api_types::chat::{clean_message, Reject};
pub use api_types::limits::{MAX_LINK_CHARS, MAX_MESSAGE_CHARS, MAX_USERNAME_CHARS};
use dioxus::prelude::*;

use crate::api::ChatMessage;
use crate::languages::Strings;

/// Entries per page, for the first load and for each "load older".
pub const PAGE_SIZE: u32 = 20;

/// id of the message textarea (the toolbar script finds it by this).
pub const EDITOR_ID: &str = "guestbook-message";

/// Merges `incoming` into `list`: ids already present are ignored (a refresh
/// and the answer to our own POST can overlap) and the result stays ordered
/// by id, oldest first. Returns whether anything was added.
pub fn merge(list: &mut Vec<ChatMessage>, incoming: Vec<ChatMessage>) -> bool {
    let before = list.len();
    for m in incoming {
        if !list.iter().any(|known| known.id == m.id) {
            list.push(m);
        }
    }
    let added = list.len() != before;
    if added {
        list.sort_by_key(|m| m.id);
    }
    added
}

/// The newest id on screen: the cursor for "refresh".
pub fn newest_id(list: &[ChatMessage]) -> Option<u32> {
    list.iter().map(|m| m.id).max()
}

/// The oldest id on screen: the cursor for "load older".
pub fn oldest_id(list: &[ChatMessage]) -> Option<u32> {
    list.iter().map(|m| m.id).min()
}

/// A full page means there may be more behind it; a short one is the end.
/// (If the total happens to be an exact multiple of the page size, the next
/// click finds nothing and the button goes away then.)
pub fn may_have_older(page_len: usize) -> bool {
    page_len >= PAGE_SIZE as usize
}

/// What the Preview tab renders: the text as the server will store it
/// (tabs become spaces, blank lines are squeezed, ...). If the text is not
/// valid yet (too long, a forbidden character) the raw text is shown anyway,
/// so the preview keeps working while the person fixes it.
pub fn preview_source(draft: &str) -> String {
    clean_message(draft).unwrap_or_else(|_| draft.trim().to_owned())
}

/// Characters still available (negative once over the limit).
pub fn chars_left(draft: &str) -> i64 {
    let used = draft.chars().count() as i64;
    MAX_MESSAGE_CHARS as i64 - used
}

/// `true` while the tab is in the background (`document.hidden`). If the
/// browser cannot say, the tab counts as visible.
pub async fn tab_hidden() -> bool {
    let mut probe = document::eval("dioxus.send(document.hidden === true);");
    probe.recv::<bool>().await.unwrap_or(false)
}

/// The text for a local validation failure, in the current language.
pub fn reject_message(reject: Reject, s: &Strings) -> &'static str {
    let t = &s.guestbook;
    match reject {
        Reject::UsernameLength => t.reject_username_length,
        Reject::UsernameChars => t.reject_username_chars,
        Reject::MessageLength => t.reject_message_length,
        Reject::MessageChars => t.reject_message_chars,
        Reject::Link => t.reject_link,
        Reject::Blocked => t.reject_blocked,
    }
}

// ---------------------------------------------------------------------------
// Editor toolbar
// ---------------------------------------------------------------------------

/// Which Markdown syntax a toolbar button inserts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Bold,
    Italic,
    Strike,
    Code,
    Link,
    Image,
    Quote,
    List,
    Tasks,
    Heading,
}

impl Format {
    /// Toolbar order.
    pub const ALL: [Format; 10] = [
        Format::Heading,
        Format::Bold,
        Format::Italic,
        Format::Strike,
        Format::Code,
        Format::Link,
        Format::Image,
        Format::Quote,
        Format::List,
        Format::Tasks,
    ];

    /// The `KIND` the script understands (`js/guestbook_editor.js`).
    pub fn kind(self) -> &'static str {
        match self {
            Format::Bold => "bold",
            Format::Italic => "italic",
            Format::Strike => "strike",
            Format::Code => "code",
            Format::Link => "link",
            Format::Image => "image",
            Format::Quote => "quote",
            Format::List => "list",
            Format::Tasks => "tasks",
            Format::Heading => "heading",
        }
    }

    /// Tooltip and accessible name.
    pub fn label(self, s: &Strings) -> &'static str {
        let t = &s.guestbook;
        match self {
            Format::Bold => t.fmt_bold,
            Format::Italic => t.fmt_italic,
            Format::Strike => t.fmt_strike,
            Format::Code => t.fmt_code,
            Format::Link => t.fmt_link,
            Format::Image => t.fmt_image,
            Format::Quote => t.fmt_quote,
            Format::List => t.fmt_list,
            Format::Tasks => t.fmt_tasks,
            Format::Heading => t.fmt_heading,
        }
    }
}

impl Format {
    /// What the toolbar button shows. Symbols and single letters, not words,
    /// so they need no translation (the tooltip carries the translated name).
    pub fn glyph(self) -> &'static str {
        match self {
            Format::Heading => "H",
            Format::Bold => "B",
            Format::Italic => "I",
            Format::Strike => "S",
            Format::Code => "<>",
            Format::Link => "@",
            Format::Image => "\u{25A3}",
            Format::Quote => "\u{275D}",
            Format::List => "\u{2022}",
            Format::Tasks => "\u{2611}",
        }
    }
}

const EDITOR_JS: &str = include_str!("../js/guestbook_editor.js");

/// The script for one toolbar click. The three inputs are written as JSON
/// string literals (the placeholder is UI text and may hold any character).
pub fn editor_script(format: Format, placeholder: &str) -> String {
    let literal = |s: &str| serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_owned());
    format!(
        "const ID = {}; const KIND = {}; const PLACEHOLDER = {};\n{EDITOR_JS}",
        literal(EDITOR_ID),
        literal(format.kind()),
        literal(placeholder),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::languages::Language;

    fn msg(id: u32) -> ChatMessage {
        ChatMessage {
            id,
            username: format!("u{id}"),
            url: None,
            message: "m".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    fn ids(list: &[ChatMessage]) -> Vec<u32> {
        list.iter().map(|m| m.id).collect()
    }

    #[test]
    fn merge_keeps_ids_ordered_and_ignores_known_ones() {
        let mut list = vec![msg(4), msg(5)];
        assert!(merge(&mut list, vec![msg(2), msg(5), msg(3)]));
        assert_eq!(ids(&list), [2, 3, 4, 5]);
        assert!(merge(&mut list, vec![msg(9)]));
        assert!(!merge(&mut list, vec![msg(3), msg(9)]));
        assert!(!merge(&mut list, vec![]));
        assert_eq!(ids(&list), [2, 3, 4, 5, 9]);
    }

    #[test]
    fn merge_keeps_the_first_copy_of_an_id() {
        let mut list = vec![msg(1)];
        let mut copy = msg(1);
        copy.message = "changed".into();
        merge(&mut list, vec![copy]);
        assert_eq!(list[0].message, "m");
    }

    #[test]
    fn cursors_are_the_extremes_not_the_positions() {
        assert_eq!((newest_id(&[]), oldest_id(&[])), (None, None));
        let list = [msg(5), msg(9), msg(7)];
        assert_eq!(newest_id(&list), Some(9));
        assert_eq!(oldest_id(&list), Some(5));
    }

    #[test]
    fn only_a_full_page_suggests_there_is_more() {
        assert!(!may_have_older(0));
        assert!(!may_have_older(PAGE_SIZE as usize - 1));
        assert!(may_have_older(PAGE_SIZE as usize));
    }

    #[test]
    fn the_preview_shows_the_text_as_the_server_will_store_it() {
        assert_eq!(
            preview_source("  hi\r\n\r\n\r\n\tthere  "),
            "hi\n\n    there"
        );
        // Not valid (too long): still previewed, as typed.
        let long = "x".repeat(MAX_MESSAGE_CHARS + 5);
        assert_eq!(preview_source(&long), long);
        assert_eq!(preview_source("   "), "");
    }

    #[test]
    fn the_counter_counts_characters_not_bytes_and_can_go_negative() {
        assert_eq!(chars_left(""), MAX_MESSAGE_CHARS as i64);
        assert_eq!(chars_left("héllo"), MAX_MESSAGE_CHARS as i64 - 5);
        assert_eq!(chars_left(&"x".repeat(MAX_MESSAGE_CHARS + 3)), -3);
    }

    #[test]
    fn english_texts_match_the_shared_contract() {
        // The numbers in the English strings come from `api_types`; if a limit
        // changes there, this fails until `language.rs` is updated.
        let en = Language::En.strings();
        for r in ALL_REJECTS {
            assert_eq!(reject_message(r, en), r.message());
        }
    }

    const ALL_REJECTS: [Reject; 6] = [
        Reject::UsernameLength,
        Reject::UsernameChars,
        Reject::MessageLength,
        Reject::MessageChars,
        Reject::Link,
        Reject::Blocked,
    ];

    #[test]
    fn every_reject_has_text_in_every_language() {
        for lang in Language::ALL {
            for r in ALL_REJECTS {
                assert!(!reject_message(r, lang.strings()).trim().is_empty());
            }
        }
    }

    #[test]
    fn every_toolbar_button_has_a_distinct_kind_and_a_label_in_every_language() {
        let mut kinds: Vec<_> = Format::ALL.iter().map(|f| f.kind()).collect();
        kinds.sort_unstable();
        kinds.dedup();
        assert_eq!(kinds.len(), Format::ALL.len());
        for lang in Language::ALL {
            for f in Format::ALL {
                assert!(!f.label(lang.strings()).trim().is_empty());
            }
        }
    }

    #[test]
    fn the_editor_script_gets_its_inputs_as_safe_literals() {
        let js = editor_script(Format::Bold, "te\"xt\\ '</script>\n");
        assert!(js.starts_with("const ID = \"guestbook-message\"; const KIND = \"bold\";"));
        assert!(js.contains(r#"const PLACEHOLDER = "te\"xt\\ '</script>\n";"#));
        assert!(js.contains("function edit("));
    }
}
