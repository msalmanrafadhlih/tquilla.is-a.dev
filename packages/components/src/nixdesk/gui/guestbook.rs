//! The Guestbook window: a list of entries (newest first, rendered from
//! Markdown), a Write/Preview editor to sign it, and a way to report an entry.
//!
//! The logic that can be tested without a browser lives in
//! `crate::guestbook`; the Markdown renderer in `crate::markdown`.

use api_types::chat::{clean_link, clean_message, clean_username, Reject};
use dioxus::prelude::*;

use crate::api::{self, ApiError, ChatMessage, NewMessage, Page};
use crate::guestbook::{
    chars_left, editor_script, may_have_older, merge, newest_id, oldest_id, preview_source,
    reject_message, Format, EDITOR_ID, MAX_LINK_CHARS, MAX_USERNAME_CHARS, PAGE_SIZE,
};
use crate::languages::use_strings;
use crate::markdown::Markdown;
use crate::nixdesk::js_util::eval_js;
use crate::remote::{LoadFailed, Loading};
use crate::time::relative_time;

const AVATAR_1: Asset = asset!("/assets/profile_default_01.svg");
const AVATAR_2: Asset = asset!("/assets/profile_default_02.svg");
const AVATAR_3: Asset = asset!("/assets/profile_default_03.svg");
const AVATAR_4: Asset = asset!("/assets/profile_default_04.svg");
const AVATAR_5: Asset = asset!("/assets/profile_default_05.svg");

/// id of the scrolling list (the scripts below find it by this).
const LIST_ID: &str = "guestbook-scroll";
/// Most rounds a "refresh" may take when many entries arrived at once.
const MAX_REFRESH_ROUNDS: usize = 5;
/// Rows of the message box.
const EDITOR_ROWS: &str = "5";

const FIELD_CLASS: &str = "w-full min-w-0 appearance-none border-0 border-b border-solid border-[var(--fg-secondary)] bg-transparent p-1 text-xs text-[var(--fg-main)] outline-none placeholder:text-[var(--fg-secondary)] focus:border-[var(--fg-main)] [font:'JetBrains_Mono-Regular',Helvetica]";
const GHOST_BUTTON: &str = "cursor-pointer border border-solid border-[var(--fg-secondary)] bg-transparent px-2 py-1 text-[10px] text-[var(--fg-main)] hover:bg-[var(--fg-main)] hover:text-[var(--bg-secondary)] disabled:cursor-default disabled:opacity-50 disabled:hover:bg-transparent disabled:hover:text-[var(--fg-main)] [font:'JetBrains_Mono-Regular',Helvetica]";

/// Default avatar, "random" but stable: picked from a hash of the username, so
/// the same person always gets the same avatar on all of their entries.
fn default_avatar_for(username: &str) -> Asset {
    let hash = username
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
    match hash % 5 {
        0 => AVATAR_1,
        1 => AVATAR_2,
        2 => AVATAR_3,
        3 => AVATAR_4,
        _ => AVATAR_5,
    }
}

/// Why an entry was not sent. Kept as data and translated when shown, so
/// switching language updates a message that is already on screen.
#[derive(Debug, Clone, PartialEq)]
enum SendError {
    /// Caught here, before any request (same rules as the server).
    Invalid(Reject),
    /// The server (or the network) said no.
    Failed(ApiError),
}

/// What the form holds -> what is sent. The same cleaning the server does, so
/// the person sees a problem immediately; the server still checks again.
fn build_message(username: &str, url: &str, text: &str) -> Result<NewMessage, Reject> {
    Ok(NewMessage {
        username: clean_username(username)?,
        url: clean_link(url)?,
        message: clean_message(text)?,
    })
}

/// Feedback under the list header.
#[derive(Debug, Clone, PartialEq)]
enum Notice {
    UpToDate,
    Failed(ApiError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Write,
    Preview,
}

/// Loads the latest entries, then hands them to [`Book`].
#[component]
pub fn GuestbookWindowContent() -> Element {
    let s = use_strings();
    let mut resource = use_resource(|| api::messages(Page::Latest, Some(PAGE_SIZE)));
    let state = resource.read_unchecked();
    match &*state {
        None => rsx! {
            Loading { label: s.guestbook.loading }
        },
        Some(Err(e)) => rsx! {
            LoadFailed {
                error: e.clone(),
                headline: s.guestbook.load_failed,
                on_retry: move |_| resource.restart(),
            }
        },
        Some(Ok(list)) => rsx! {
            Book { initial: list.clone() }
        },
    }
}

#[component]
fn Book(initial: Vec<ChatMessage>) -> Element {
    let s = use_strings();
    let older_at_start = may_have_older(initial.len());
    let mut entries = use_signal(move || initial);
    let mut has_older = use_signal(move || older_at_start);
    let mut refreshing = use_signal(|| false);
    let mut loading_older = use_signal(|| false);
    let mut notice = use_signal(|| None::<Notice>);

    // Entries newer than the newest one on screen. A full page may mean more
    // are waiting, so ask again (a few times at most).
    let refresh = move |_| {
        if *refreshing.peek() {
            return;
        }
        refreshing.set(true);
        notice.set(None);
        spawn(async move {
            let mut added = false;
            for _ in 0..MAX_REFRESH_ROUNDS {
                let page = newest_id(&entries.peek()).map_or(Page::Latest, Page::After);
                match api::messages(page, Some(PAGE_SIZE)).await {
                    Ok(batch) => {
                        let full = batch.len() >= PAGE_SIZE as usize;
                        added |= merge(&mut entries.write(), batch);
                        if !full {
                            break;
                        }
                    }
                    Err(e) => {
                        notice.set(Some(Notice::Failed(e)));
                        refreshing.set(false);
                        return;
                    }
                }
            }
            if !added {
                notice.set(Some(Notice::UpToDate));
            }
            refreshing.set(false);
        });
    };

    let load_older = move |_| {
        if *loading_older.peek() {
            return;
        }
        let Some(before) = oldest_id(&entries.peek()) else {
            return;
        };
        loading_older.set(true);
        notice.set(None);
        spawn(async move {
            match api::messages(Page::Before(before), Some(PAGE_SIZE)).await {
                Ok(batch) => {
                    has_older.set(may_have_older(batch.len()));
                    merge(&mut entries.write(), batch);
                }
                Err(e) => notice.set(Some(Notice::Failed(e))),
            }
            loading_older.set(false);
        });
    };

    let list = entries();
    let count = list.len();

    rsx! {
        div { class: "flex h-full w-full flex-col bg-[var(--bg-secondary)] text-sm",

            // LIST HEADER
            div { class: "flex flex-none items-center justify-between gap-2.5 border-t border-solid border-[var(--fg-secondary)] px-2.5 py-2",
                h2 { class: "m-0 text-xs font-normal text-[var(--fg-main)] [font:'JetBrains_Mono-Regular',Helvetica]", "{s.guestbook.entries_title}" }
                button {
                    r#type: "button",
                    class: GHOST_BUTTON,
                    disabled: refreshing(),
                    onclick: refresh,
                    if refreshing() {
                        "{s.guestbook.refreshing}"
                    } else {
                        "{s.guestbook.refresh}"
                    }
                }
            }
            match notice() {
                Some(Notice::UpToDate) => rsx! {
                    p { role: "status", class: "m-0 flex-none px-2.5 pb-1 text-[10px] text-[var(--fg-secondary)] [font:'JetBrains_Mono-Regular',Helvetica]",
                        "{s.guestbook.up_to_date}"
                    }
                },
                Some(Notice::Failed(e)) => rsx! {
                    p { role: "alert", class: "m-0 flex-none px-2.5 pb-1 text-[10px] text-[var(--red)] [font:'JetBrains_Mono-Regular',Helvetica]",
                        {e.message(s)}
                    }
                },
                None => rsx! {},
            }

            // ENTRIES (newest first)
            section {
                id: LIST_ID,
                aria_label: s.guestbook.entries_title,
                class: "flex min-h-0 flex-1 flex-col overflow-y-scroll border-y border-solid border-[var(--fg-secondary)] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden",
                if count == 0 {
                    p { class: "p-5 text-center text-xs text-[var(--fg-secondary)] [font:'JetBrains_Mono-Regular',Helvetica]",
                        "{s.guestbook.empty}"
                    }
                }
                for msg in list.into_iter().rev() {
                    Entry { key: "{msg.id}", msg }
                }
                if has_older() {
                    div { class: "flex justify-center p-2.5",
                        button {
                            r#type: "button",
                            class: GHOST_BUTTON,
                            disabled: loading_older(),
                            onclick: load_older,
                            if loading_older() {
                                "{s.guestbook.loading_older}"
                            } else {
                                "{s.guestbook.load_older}"
                            }
                        }
                    }
                } else if count > 0 {
                    p { class: "m-0 p-2.5 text-center text-[10px] text-[var(--fg-secondary)] [font:'JetBrains_Mono-Regular',Helvetica]",
                        "{s.guestbook.first_entry}"
                    }
                }
            }

            Composer { entries }
        }
    }
}

/// One entry: avatar, name (a link if the author gave a website), time, the
/// rendered Markdown and the report button.
#[component]
fn Entry(msg: ChatMessage) -> Element {
    let s = use_strings();
    rsx! {
        article {
            class: "flex items-start gap-3 border-b border-solid border-[var(--fg-secondary)] p-2.5 last:border-b-0",
            aria_labelledby: "guestbook-author-{msg.id}",
            div { class: "flex h-[40px] w-[40px] flex-none items-center justify-center overflow-hidden border border-solid border-[var(--fg-main)]",
                img {
                    class: "h-full w-full object-cover",
                    src: "{default_avatar_for(&msg.username)}",
                    alt: "{s.guestbook.avatar_alt(&msg.username)}",
                }
            }
            div { class: "flex min-w-0 flex-1 flex-col gap-1.5",
                header { class: "flex items-baseline justify-between gap-2.5",
                    if let Some(link) = &msg.url {
                        a {
                            id: "guestbook-author-{msg.id}",
                            href: "{link}",
                            target: "_blank",
                            rel: "noopener noreferrer nofollow ugc",
                            title: "{link}",
                            class: "min-w-0 truncate text-xs text-[var(--fg-main)] underline decoration-dotted decoration-[var(--fg-secondary)] underline-offset-4 hover:decoration-solid hover:decoration-[var(--fg-main)] [font:'JetBrains_Mono-Regular',Helvetica]",
                            "{msg.username}"
                        }
                    } else {
                        h3 {
                            id: "guestbook-author-{msg.id}",
                            class: "m-0 min-w-0 truncate text-xs font-normal text-[var(--fg-main)] [font:'JetBrains_Mono-Regular',Helvetica]",
                            "{msg.username}"
                        }
                    }
                    time {
                        datetime: "{msg.created_at}",
                        class: "flex-none text-[10px] text-[var(--fg-secondary)] [font:'JetBrains_Mono-Regular',Helvetica]",
                        "{relative_time(&msg.created_at, s)}"
                    }
                }
                Markdown { source: msg.message.clone() }
                ReportButton { id: msg.id }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ReportState {
    Idle,
    /// First click done: asks to click again, so one stray click reports nothing.
    Confirm,
    Sending,
    Done,
    Failed(ApiError),
}

#[component]
fn ReportButton(id: u32) -> Element {
    let s = use_strings();
    let mut state = use_signal(|| ReportState::Idle);
    let send = move |_| match state() {
        ReportState::Idle | ReportState::Failed(_) => state.set(ReportState::Confirm),
        ReportState::Confirm => {
            state.set(ReportState::Sending);
            spawn(async move {
                match api::report_message(id).await {
                    Ok(()) => state.set(ReportState::Done),
                    Err(e) => state.set(ReportState::Failed(e)),
                }
            });
        }
        ReportState::Sending | ReportState::Done => {}
    };
    rsx! {
        div { class: "flex items-center justify-end gap-2",
            match state() {
                ReportState::Done => rsx! {
                    span { role: "status", class: "text-[10px] text-[var(--fg-secondary)] [font:'JetBrains_Mono-Regular',Helvetica]", "{s.guestbook.reported}" }
                },
                ReportState::Failed(e) => rsx! {
                    span { role: "alert", class: "text-[10px] text-[var(--red)] [font:'JetBrains_Mono-Regular',Helvetica]", {e.message(s)} }
                    button {
                        r#type: "button",
                        class: "cursor-pointer border-0 bg-transparent p-0 text-[10px] text-[var(--fg-secondary)] underline hover:text-[var(--fg-main)] [font:'JetBrains_Mono-Regular',Helvetica]",
                        onclick: send,
                        "{s.guestbook.report}"
                    }
                },
                other => rsx! {
                    button {
                        r#type: "button",
                        disabled: other == ReportState::Sending,
                        class: if other == ReportState::Confirm { "cursor-pointer border-0 bg-transparent p-0 text-[10px] text-[var(--yellow)] underline [font:'JetBrains_Mono-Regular',Helvetica]" } else { "cursor-pointer border-0 bg-transparent p-0 text-[10px] text-[var(--fg-secondary)] underline hover:text-[var(--fg-main)] disabled:cursor-default [font:'JetBrains_Mono-Regular',Helvetica]" },
                        onclick: send,
                        match other {
                            ReportState::Confirm => "{s.guestbook.report_confirm}",
                            ReportState::Sending => "{s.guestbook.reporting}",
                            _ => "{s.guestbook.report}",
                        }
                    }
                },
            }
        }
    }
}

/// The form: name, website, the Write/Preview editor and the Sign button.
#[component]
fn Composer(entries: Signal<Vec<ChatMessage>>) -> Element {
    let s = use_strings();
    let mut username = use_signal(String::new);
    let mut url = use_signal(String::new);
    let mut draft = use_signal(String::new);
    let mut tab = use_signal(|| Tab::Write);
    let mut sending = use_signal(|| false);
    let mut error = use_signal(|| None::<SendError>);
    let mut sent = use_signal(|| false);

    let mut submit = move || {
        if *sending.peek() || draft.peek().trim().is_empty() {
            return;
        }
        let new = match build_message(&username.peek(), &url.peek(), &draft.peek()) {
            Ok(new) => new,
            Err(reason) => {
                sent.set(false);
                error.set(Some(SendError::Invalid(reason)));
                return;
            }
        };
        sending.set(true);
        error.set(None);
        sent.set(false);
        spawn(async move {
            match api::post_message(&new).await {
                Ok(entry) => {
                    merge(&mut entries.write(), vec![entry]);
                    draft.set(String::new());
                    tab.set(Tab::Write);
                    sent.set(true);
                    // Newest first: the new entry is at the top.
                    eval_js(format!(
                        "const list = document.getElementById('{LIST_ID}'); if (list) list.scrollTop = 0;"
                    ));
                }
                Err(e) => error.set(Some(SendError::Failed(e))),
            }
            sending.set(false);
        });
    };

    let shown_name = username();
    let text = draft();
    let left = chars_left(&text);
    let preview = preview_source(&text);
    let current = tab();

    let tab_class = |active: bool| {
        if active {
            "cursor-pointer border-0 border-b border-solid border-[var(--fg-main)] bg-transparent px-2 py-1 text-[10px] text-[var(--fg-main)] [font:'JetBrains_Mono-Regular',Helvetica]"
        } else {
            "cursor-pointer border-0 border-b border-solid border-transparent bg-transparent px-2 py-1 text-[10px] text-[var(--fg-secondary)] hover:text-[var(--fg-main)] [font:'JetBrains_Mono-Regular',Helvetica]"
        }
    };

    rsx! {
        form {
            aria_label: s.guestbook.form_label,
            class: "flex flex-none flex-col gap-2 bg-[var(--bg-secondary)] p-2.5",
            onsubmit: move |evt: FormEvent| {
                evt.prevent_default();
                submit();
            },

            div { class: "flex items-center gap-2.5",
                div { class: "flex h-[40px] w-[40px] flex-none items-center justify-center overflow-hidden border border-solid border-[var(--fg-main)]",
                    img {
                        class: "h-full w-full object-cover",
                        src: "{default_avatar_for(&shown_name)}",
                        alt: s.guestbook.your_avatar_alt,
                    }
                }
                div { class: "flex min-w-0 flex-1 gap-2.5",
                    label { class: "sr-only", r#for: "guestbook-name", "{s.guestbook.username_label}" }
                    input {
                        class: FIELD_CLASS,
                        id: "guestbook-name",
                        name: "username",
                        r#type: "text",
                        maxlength: "{MAX_USERNAME_CHARS}",
                        placeholder: s.guestbook.username_placeholder,
                        autocomplete: "username",
                        value: "{username}",
                        oninput: move |evt: FormEvent| username.set(evt.value()),
                    }
                    label { class: "sr-only", r#for: "guestbook-url", "{s.guestbook.url_label}" }
                    input {
                        class: FIELD_CLASS,
                        id: "guestbook-url",
                        name: "url",
                        r#type: "text",
                        inputmode: "url",
                        maxlength: "{MAX_LINK_CHARS}",
                        placeholder: s.guestbook.url_placeholder,
                        autocomplete: "url",
                        value: "{url}",
                        oninput: move |evt: FormEvent| url.set(evt.value()),
                    }
                }
            }

            // Write / Preview tabs and the counter
            div { class: "flex items-end justify-between gap-2.5",
                div {
                    role: "tablist",
                    aria_label: s.guestbook.editor_modes,
                    class: "flex",
                    button {
                        r#type: "button",
                        role: "tab",
                        aria_selected: current == Tab::Write,
                        class: tab_class(current == Tab::Write),
                        onclick: move |_| tab.set(Tab::Write),
                        "{s.guestbook.tab_write}"
                    }
                    button {
                        r#type: "button",
                        role: "tab",
                        aria_selected: current == Tab::Preview,
                        class: tab_class(current == Tab::Preview),
                        onclick: move |_| tab.set(Tab::Preview),
                        "{s.guestbook.tab_preview}"
                    }
                }
                span {
                    aria_live: "polite",
                    class: if left < 0 { "text-[10px] text-[var(--red)] [font:'JetBrains_Mono-Regular',Helvetica]" } else { "text-[10px] text-[var(--fg-secondary)] [font:'JetBrains_Mono-Regular',Helvetica]" },
                    if left < 0 {
                        "{s.guestbook.chars_over(left.unsigned_abs() as u32)}"
                    } else {
                        "{s.guestbook.chars_left(left as u32)}"
                    }
                }
            }

            if current == Tab::Write {
                div {
                    role: "toolbar",
                    aria_label: s.guestbook.toolbar_label,
                    class: "flex flex-wrap gap-1",
                    for format in Format::ALL {
                        button {
                            key: "{format.kind()}",
                            r#type: "button",
                            title: format.label(s),
                            aria_label: format.label(s),
                            class: "h-6 min-w-6 cursor-pointer border border-solid border-[var(--fg-secondary)] bg-transparent px-1.5 text-[11px] text-[var(--fg-main)] hover:bg-[var(--fg-main)] hover:text-[var(--bg-secondary)] [font:'JetBrains_Mono-Regular',Helvetica]",
                            onclick: move |_| {
                                eval_js(editor_script(format, s.guestbook.fmt_placeholder));
                            },
                            "{format.glyph()}"
                        }
                    }
                }
                label { class: "sr-only", r#for: EDITOR_ID, "{s.guestbook.message_label}" }
                textarea {
                    class: "{FIELD_CLASS} h-28 resize-none border border-solid p-2 leading-relaxed",
                    id: EDITOR_ID,
                    name: "message",
                    rows: EDITOR_ROWS,
                    placeholder: s.guestbook.message_placeholder,
                    value: "{draft}",
                    oninput: move |evt: FormEvent| draft.set(evt.value()),
                    // Ctrl/Cmd+Enter signs. Plain Enter is a new line (Markdown needs lines).
                    onkeydown: move |evt: KeyboardEvent| {
                        let m = evt.modifiers();
                        if evt.key() == Key::Enter && (m.ctrl() || m.meta()) {
                            evt.prevent_default();
                            submit();
                        }
                    },
                }
            } else {
                div {
                    class: "h-[calc(1.5rem+7rem)] overflow-y-auto border border-solid border-[var(--fg-secondary)] p-2 [scrollbar-width:thin]",
                    aria_label: s.guestbook.tab_preview,
                    if preview.trim().is_empty() {
                        p { class: "m-0 text-xs text-[var(--fg-secondary)] [font:'JetBrains_Mono-Regular',Helvetica]",
                            "{s.guestbook.preview_empty}"
                        }
                    } else {
                        Markdown { source: preview }
                    }
                }
            }

            if let Some(err) = error() {
                p { role: "alert", class: "m-0 text-[10px] text-[var(--red)] [font:'JetBrains_Mono-Regular',Helvetica]",
                    {
                        match &err {
                            SendError::Invalid(reason) => reject_message(*reason, s),
                            SendError::Failed(e) => e.message(s),
                        }
                    }
                }
            } else if sent() {
                p { role: "status", class: "m-0 text-[10px] text-[var(--green)] [font:'JetBrains_Mono-Regular',Helvetica]",
                    "{s.guestbook.sent}"
                }
            }

            div { class: "flex items-center justify-between gap-2.5",
                span { class: "text-[10px] text-[var(--fg-secondary)] [font:'JetBrains_Mono-Regular',Helvetica]", "{s.guestbook.markdown_hint}" }
                button {
                    r#type: "submit",
                    disabled: sending() || text.trim().is_empty(),
                    class: "{GHOST_BUTTON} px-3 py-1.5 text-xs",
                    if sending() {
                        "{s.guestbook.sending}"
                    } else {
                        "{s.guestbook.send}"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guestbook::MAX_MESSAGE_CHARS;

    #[test]
    fn default_avatar_is_stable_per_username() {
        assert_eq!(
            default_avatar_for("tquilla").to_string(),
            default_avatar_for("tquilla").to_string()
        );
    }

    #[test]
    fn build_message_cleans_the_same_way_the_server_does() {
        let m = build_message("  Moch   T ", " a.dev ", "hi\r\n\r\n\r\nthere  ").unwrap();
        assert_eq!(m.username, "Moch T");
        assert_eq!(m.url.as_deref(), Some("https://a.dev"));
        assert_eq!(m.message, "hi\n\nthere");
        assert_eq!(build_message("a", "", "b").unwrap().url, None);
    }

    #[test]
    fn build_message_keeps_markdown_as_typed() {
        let md = "# Hi\n\n- **a**\n- [b](https://b.dev)\n\n```\ncode\n```";
        assert_eq!(build_message("a", "", md).unwrap().message, md);
    }

    #[test]
    fn build_message_reports_the_first_problem() {
        assert_eq!(
            build_message("   ", "", "hi").unwrap_err(),
            Reject::UsernameLength
        );
        assert_eq!(
            build_message("a", "bad url", "hi").unwrap_err(),
            Reject::Link
        );
        assert_eq!(
            build_message("a", "", &"x".repeat(MAX_MESSAGE_CHARS + 1)).unwrap_err(),
            Reject::MessageLength
        );
        assert_eq!(
            build_message("a", "", "hi\u{202E}").unwrap_err(),
            Reject::MessageChars
        );
    }

    #[test]
    fn build_message_never_yields_a_dangerous_link() {
        for raw in ["javascript:alert(1)", "data:text/html,x", "JaVaScRiPt:x"] {
            if let Ok(m) = build_message("a", raw, "hi") {
                let link = m.url.unwrap().to_ascii_lowercase();
                assert!(link.starts_with("https://") || link.starts_with("http://"));
            }
        }
    }
}
