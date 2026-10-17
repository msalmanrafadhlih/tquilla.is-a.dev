//! Logic shared by the Live Chat window and the Terminal chat preview, kept
//! free of UI so it can be unit-tested without a browser:
//!
//! - how a polled batch is merged into the list that is on screen;
//! - how often to poll (the Worker free plan allows 100k requests a day, so
//!   an open tab must get cheaper the quieter the chat is, and a hidden tab
//!   must cost nothing);
//! - turning a local validation failure into text in the current language.

use api_types::chat::Reject;
pub use api_types::limits::{MAX_LINK_CHARS, MAX_MESSAGE_CHARS, MAX_USERNAME_CHARS};
use dioxus::prelude::*;

use crate::api::ChatMessage;
use crate::languages::Strings;

/// How many messages a list keeps; older ones scroll away for good. The
/// server returns at most `MESSAGES_MAX_LIMIT` per request anyway.
pub const MAX_KEPT: usize = 200;

/// The poll loop wakes up this often to look at the clock and at whether the
/// tab is visible. Waking up costs nothing; only a fetch is a request.
pub const TICK_MS: u32 = 2_500;

/// Merges `incoming` into `list`: ids already present are ignored (a polled
/// batch and the answer to our own POST can overlap), the result is ordered
/// by id (the server's insertion order) and trimmed to the newest
/// [`MAX_KEPT`]. Returns whether anything was added.
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
        if list.len() > MAX_KEPT {
            list.drain(..list.len() - MAX_KEPT);
        }
    }
    added
}

/// The newest id on screen: what to pass as `after` on the next poll.
pub fn last_id(list: &[ChatMessage]) -> Option<u32> {
    list.iter().map(|m| m.id).max()
}

/// Milliseconds to wait before the next poll, given how many polls in a row
/// found nothing new (`idle`) and how many in a row failed (`failures`).
///
/// A lively chat is polled every 10 s; after a quiet while it slows down to
/// once a minute (about 1,400 requests a day for a tab left open all day,
/// 1.4% of the free plan). Failures back off further, up to 2 minutes, so a
/// server that is struggling is not hammered by every open tab.
pub fn poll_delay(idle: u32, failures: u32) -> u32 {
    let quiet = match idle {
        0..=2 => 10_000,
        3..=5 => 15_000,
        6..=9 => 30_000,
        _ => 60_000,
    };
    let backoff = match failures {
        0 => 0,
        1 => 20_000,
        2 => 40_000,
        3 => 80_000,
        _ => 120_000,
    };
    quiet.max(backoff)
}

/// `true` while the tab is in the background (`document.hidden`). If the
/// browser cannot say, the tab counts as visible.
pub async fn tab_hidden() -> bool {
    let mut probe = document::eval("dioxus.send(document.hidden === true);");
    probe.recv::<bool>().await.unwrap_or(false)
}

/// `true` when the message list is scrolled to (nearly) the bottom, or when it
/// cannot be measured. Asked before new messages are added, so a reader who
/// scrolled up is not yanked down.
pub async fn near_bottom(element_id: &str) -> bool {
    let js = format!(
        "const el = document.getElementById('{element_id}'); \
         dioxus.send(!el || el.scrollHeight - el.scrollTop - el.clientHeight < 80);"
    );
    let mut probe = document::eval(&js);
    probe.recv::<bool>().await.unwrap_or(true)
}

/// The text for a local validation failure, in the current language.
pub fn reject_message(reject: Reject, s: &Strings) -> &'static str {
    let t = &s.livechat;
    match reject {
        Reject::UsernameLength => t.reject_username_length,
        Reject::UsernameChars => t.reject_username_chars,
        Reject::MessageLength => t.reject_message_length,
        Reject::MessageChars => t.reject_message_chars,
        Reject::Link => t.reject_link,
        Reject::Blocked => t.reject_blocked,
    }
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
            avatar_url: None,
            message: "m".into(),
            attachment_url: None,
            created_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    fn ids(list: &[ChatMessage]) -> Vec<u32> {
        list.iter().map(|m| m.id).collect()
    }

    #[test]
    fn merge_appends_new_ids_in_order_and_ignores_known_ones() {
        let mut list = vec![msg(1), msg(2)];
        assert!(merge(&mut list, vec![msg(4), msg(2), msg(3)]));
        assert_eq!(ids(&list), [1, 2, 3, 4]);
        assert!(!merge(&mut list, vec![msg(3), msg(4)]));
        assert!(!merge(&mut list, vec![]));
        assert_eq!(ids(&list), [1, 2, 3, 4]);
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
    fn merge_keeps_only_the_newest_messages() {
        let mut list: Vec<_> = (1..=MAX_KEPT as u32).map(msg).collect();
        let top = MAX_KEPT as u32;
        assert!(merge(&mut list, vec![msg(top + 1), msg(top + 2)]));
        assert_eq!(list.len(), MAX_KEPT);
        assert_eq!(list.first().map(|m| m.id), Some(3));
        assert_eq!(list.last().map(|m| m.id), Some(top + 2));
    }

    #[test]
    fn last_id_is_the_maximum_not_the_last_position() {
        assert_eq!(last_id(&[]), None);
        assert_eq!(last_id(&[msg(5), msg(9), msg(7)]), Some(9));
    }

    #[test]
    fn polling_slows_down_when_quiet_and_never_gets_faster() {
        let mut previous = 0;
        for idle in 0..40 {
            let d = poll_delay(idle, 0);
            assert!(d >= previous, "idle {idle}: {d} < {previous}");
            assert!((10_000..=60_000).contains(&d));
            previous = d;
        }
        assert_eq!(poll_delay(0, 0), 10_000);
        assert_eq!(poll_delay(99, 0), 60_000);
    }

    #[test]
    fn failures_back_off_but_stay_bounded() {
        assert!(poll_delay(0, 1) > poll_delay(0, 0));
        assert!(poll_delay(0, 3) > poll_delay(0, 2));
        assert_eq!(poll_delay(0, 50), 120_000);
        // A long quiet spell is not made shorter by a single failure.
        assert_eq!(poll_delay(99, 1), 60_000);
    }

    #[test]
    fn the_tick_divides_every_delay() {
        for idle in [0, 3, 6, 10] {
            for failures in 0..6 {
                assert_eq!(poll_delay(idle, failures) % TICK_MS, 0);
            }
        }
    }

    #[test]
    fn english_texts_match_the_shared_contract() {
        // The numbers in the English strings come from `api_types`; if a limit
        // changes there, this fails until `language.rs` is updated.
        let en = Language::En.strings();
        for r in [
            Reject::UsernameLength,
            Reject::UsernameChars,
            Reject::MessageLength,
            Reject::MessageChars,
            Reject::Link,
            Reject::Blocked,
        ] {
            assert_eq!(reject_message(r, en), r.message());
        }
    }

    #[test]
    fn every_reject_has_text_in_every_language() {
        for lang in Language::ALL {
            for r in [
                Reject::UsernameLength,
                Reject::UsernameChars,
                Reject::MessageLength,
                Reject::MessageChars,
                Reject::Link,
                Reject::Blocked,
            ] {
                assert!(!reject_message(r, lang.strings()).trim().is_empty());
            }
        }
    }
}
