use api_types::chat::{clean_link, clean_message, clean_username, Reject};
use dioxus::prelude::*;

use crate::api::{self, ApiError, ChatMessage, NewMessage};
use crate::chat::{
    last_id, merge, near_bottom, poll_delay, reject_message, tab_hidden, MAX_LINK_CHARS,
    MAX_MESSAGE_CHARS, MAX_USERNAME_CHARS, TICK_MS,
};
use crate::languages::use_strings;
use crate::nixdesk::js_util::eval_js;
use crate::platform::TimeoutFuture;
use crate::remote::{LoadFailed, Loading};
use crate::time::relative_time;

const AVATAR_1: Asset = asset!("/assets/profile_default_01.svg");
const AVATAR_2: Asset = asset!("/assets/profile_default_02.svg");
const AVATAR_3: Asset = asset!("/assets/profile_default_03.svg");
const AVATAR_4: Asset = asset!("/assets/profile_default_04.svg");
const AVATAR_5: Asset = asset!("/assets/profile_default_05.svg");

/// id of the scrolling message list (also used by the JS snippets below).
const LIST_ID: &str = "livechat-scroll";
/// Two failed polls in a row before the "connection lost" notice appears, so a
/// single hiccup does not flash a warning.
const OFFLINE_AFTER_FAILURES: u32 = 2;

const ARTICLE_BASE_CLASS: &str = "flex items-start gap-5 p-2.5 relative self-stretch w-full flex-[0_0_auto] border-b [border-bottom-style:solid] border-[var(--fg-secondary)]";
const AUTHOR_CLASS: &str = "relative flex items-end w-fit mt-[-1.00px] [font:'JetBrains_Mono-Regular',Helvetica] font-normal text-[var(--fg-main)] opacity-70 hover:opacity-100 text-xs tracking-[0] leading-[normal]";
const BG_BLACK_TO_GRAY: &str = "bg-[image:var(--linear-l)] border-r";
const BG_GRAY_TO_BLACK: &str = "bg-[image:var(--linear-r)] border-l";
const NOTICE_CLASS: &str = "w-full px-2.5 text-[10px] [font:'JetBrains_Mono-Regular',Helvetica]";
const RESIZE_MESSAGE_BOX_JS: &str = "const el = document.getElementById('message'); if (el) { el.style.height = 'auto'; const h = el.scrollHeight; el.style.height = h + 'px'; const max = parseFloat(getComputedStyle(el).maxHeight); el.style.overflowY = h > max ? 'auto' : 'hidden'; }";
const AFTER_SEND_JS: &str = "setTimeout(() => { const list = document.getElementById('livechat-scroll'); if (list) list.scrollTop = list.scrollHeight; const el = document.getElementById('message'); if (el) { el.style.height = 'auto'; el.style.height = el.scrollHeight + 'px'; el.style.overflowY = 'hidden'; } }, 0);";
const SCROLL_BOTTOM_JS: &str = "const list = document.getElementById('livechat-scroll'); if (list) list.scrollTop = list.scrollHeight;";

/// Default avatar, "random" but stable: picked from a hash of the username, so
/// the same person always gets the same avatar on all of their messages.
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

/// Why a message was not sent. Kept as data and translated when shown, so
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

/// Everything the form and the poll loop share. All fields are signals, so the
/// struct is `Copy` and can move into any closure.
#[derive(Clone, Copy)]
struct Chat {
    messages: Signal<Vec<ChatMessage>>,
    username: Signal<String>,
    url: Signal<String>,
    draft: Signal<String>,
    sending: Signal<bool>,
    error: Signal<Option<SendError>>,
    /// Polls in a row that found nothing new (drives [`poll_delay`]).
    idle: Signal<u32>,
}

impl Chat {
    /// Validates, then posts. Enter key and the Send button both land here.
    fn submit(mut self) {
        if *self.sending.peek() || self.draft.peek().trim().is_empty() {
            return;
        }
        let new = match build_message(&self.username.peek(), &self.url.peek(), &self.draft.peek()) {
            Ok(new) => new,
            Err(reason) => {
                self.error.set(Some(SendError::Invalid(reason)));
                return;
            }
        };
        self.sending.set(true);
        self.error.set(None);
        spawn(async move {
            match api::post_message(&new).await {
                Ok(sent) => {
                    merge(&mut self.messages.write(), vec![sent]);
                    self.draft.set(String::new());
                    // A conversation is going on: poll quickly again.
                    self.idle.set(0);
                    eval_js(AFTER_SEND_JS.to_string());
                }
                Err(e) => self.error.set(Some(SendError::Failed(e))),
            }
            self.sending.set(false);
        });
    }
}

/// Loads the latest messages, then hands them to [`ChatRoom`].
#[component]
pub fn LiveChatWindowContent() -> Element {
    let s = use_strings();
    let mut resource = use_resource(|| api::messages(None, None));
    let state = resource.read_unchecked();
    match &*state {
        None => rsx! {
            Loading { label: s.livechat.loading }
        },
        Some(Err(e)) => rsx! {
            LoadFailed {
                error: e.clone(),
                headline: s.livechat.load_failed,
                on_retry: move |_| resource.restart(),
            }
        },
        Some(Ok(list)) => rsx! {
            ChatRoom { initial: list.clone() }
        },
    }
}

#[component]
fn ChatRoom(initial: Vec<ChatMessage>) -> Element {
    let s = use_strings();
    let mut chat = Chat {
        messages: use_signal(move || initial),
        username: use_signal(String::new),
        url: use_signal(String::new),
        draft: use_signal(String::new),
        sending: use_signal(|| false),
        error: use_signal(|| None),
        idle: use_signal(|| 0),
    };
    let mut offline = use_signal(|| false);
    // Bumped by every poll so "3 minutes ago" keeps moving in a quiet chat.
    let mut refresh = use_signal(|| 0u32);

    // Start at the newest message.
    use_effect(|| eval_js(SCROLL_BOTTOM_JS.to_string()));

    // Poll for messages from other people. It wakes up every `TICK_MS` but
    // only calls the server when the next poll is due, never while the tab is
    // in the background, and at once when the tab comes back. The task ends
    // with the window (Dioxus cancels it when the component is dropped).
    use_future(move || async move {
        let mut failures = 0u32;
        let mut waited = 0u32;
        let mut was_hidden = false;
        loop {
            TimeoutFuture::new(TICK_MS).await;
            if tab_hidden().await {
                was_hidden = true;
                continue;
            }
            waited += TICK_MS;
            if !was_hidden && waited < poll_delay(*chat.idle.peek(), failures) {
                continue;
            }
            waited = 0;
            was_hidden = false;

            let after = last_id(&chat.messages.peek());
            match api::messages(after, None).await {
                Ok(batch) => {
                    failures = 0;
                    offline.set(false);
                    let stick = near_bottom(LIST_ID).await;
                    if merge(&mut chat.messages.write(), batch) {
                        chat.idle.set(0);
                        if stick {
                            eval_js(SCROLL_BOTTOM_JS.to_string());
                        }
                    } else {
                        let idle = *chat.idle.peek();
                        chat.idle.set(idle.saturating_add(1));
                        refresh.set(refresh().wrapping_add(1));
                    }
                }
                Err(_) => {
                    failures = failures.saturating_add(1);
                    if failures >= OFFLINE_AFTER_FAILURES {
                        offline.set(true);
                    }
                }
            }
        }
    });

    let _ = refresh();
    let messages = (chat.messages)();
    let sending = (chat.sending)();
    let shown_name = (chat.username)();

    rsx! {
        div { class: "flex flex-col items-center gap-5 h-full w-full bg-[var(--bg-secondary)] text-sm",

            // CHAT ENVIRONMENTS
            section {
                id: LIST_ID,
                class: "flex flex-col items-center max-w-[500px] relative flex-1 min-h-0 w-full overflow-y-scroll [scrollbar-width:none] [&::-webkit-scrollbar]:hidden border-t border-solid border-[var(--fg-secondary)]",

                if messages.is_empty() {
                    p { class: "p-5 text-center text-xs text-[var(--fg-secondary)] [font:'JetBrains_Mono-Regular',Helvetica]",
                        "{s.livechat.empty}"
                    }
                }
                for (idx, msg) in messages.into_iter().enumerate() {
                    {
                        let bg = if idx % 2 == 0 { BG_BLACK_TO_GRAY } else { BG_GRAY_TO_BLACK };
                        let row_class = format!("{ARTICLE_BASE_CLASS} {bg}");
                        // Avatar pengguna kalau ada, kalau tidak -> avatar default.
                        let avatar_src = match &msg.avatar_url {
                            Some(path) => path.clone(),
                            None => default_avatar_for(&msg.username).to_string(),
                        };
                        rsx! {
                            article {
                                key: "{msg.id}",
                                class: "{row_class}",
                                aria_labelledby: "post-author-{msg.id}",

                                div { class: "flex flex-col w-[50px] h-[50px] items-center justify-center gap-2.5 relative border border-solid border-[var(--fg-main)] aspect-[1]",
                                    img {
                                        class: "relative self-stretch w-full h-full aspect-[1] object-cover",
                                        src: "{avatar_src}",
                                        alt: "{msg.username}",
                                    }
                                }

                                div { class: "flex-col pt-0 pb-2.5 px-0 flex-1 grow flex items-center gap-2.5 relative self-stretch",
                                    header { class: "justify-center w-full flex-[0_0_auto] flex items-center gap-2.5 relative self-stretch",
                                        // Username jadi link kalau pengguna mengisi URL.
                                        if let Some(link) = &msg.url {
                                            a {
                                                id: "post-author-{msg.id}",
                                                href: "{link}",
                                                target: "_blank",
                                                rel: "noopener noreferrer",
                                                class: "{AUTHOR_CLASS} underline decoration-dotted decoration-[var(--fg-secondary)] underline-offset-4 transition-[opacity,text-decoration-color] hover:decoration-solid hover:decoration-[var(--fg-main)]",
                                                "{msg.username}"
                                            }
                                        } else {
                                            h2 {
                                                id: "post-author-{msg.id}",
                                                class: "{AUTHOR_CLASS}",
                                                "{msg.username}"
                                            }
                                        }
                                        time {
                                            datetime: "{msg.created_at}",
                                            class: "relative flex items-end flex-1 [font:'JetBrains_Mono-Regular',Helvetica] font-normal text-[var(--fg-secondary)] text-[10px] tracking-[0] leading-[normal]",
                                            "{relative_time(&msg.created_at, s)}"
                                        }
                                    }
                                    if !msg.message.is_empty() {
                                        p { class: "relative flex items-end self-stretch [font:'JetBrains_Mono-Regular',Helvetica] font-normal text-[var(--fg-main)] text-xs tracking-[0] leading-[normal]",
                                            "{msg.message}"
                                        }
                                    }
                                    // Gambar lampiran (kalau ada)
                                    if let Some(src) = &msg.attachment_url {
                                        img {
                                            class: "self-start max-w-full max-h-60 object-contain border-0",
                                            src: "{src}",
                                            alt: "{s.livechat.attachment_alt(&msg.username)}",
                                            onload: move |_| {
                                                eval_js(SCROLL_BOTTOM_JS.to_string());
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }


            if offline() {
                p {
                    role: "status",
                    class: "{NOTICE_CLASS} text-[var(--fg-secondary)]",
                    "{s.livechat.reconnecting}"
                }
            }
            if let Some(err) = (chat.error)() {
                p {
                    role: "alert",
                    class: "{NOTICE_CLASS} text-[var(--red)]",
                    {
                        match &err {
                            SendError::Invalid(reason) => reject_message(*reason, s),
                            SendError::Failed(e) => e.message(s),
                        }
                    }
                }
            }

            // MAIN CONTENTS : LIVE CHAT
            form {
                aria_label: s.livechat.form_label,
                class: "flex w-full max-w-[500px] items-center justify-center relative",
                onsubmit: move |evt: FormEvent| {
                    evt.prevent_default();
                    chat.submit();
                },

                // The default avatar this name gets in the list (custom avatars come later).
                div { class: "flex flex-col w-[50px] h-[50px] m-2.5 items-center justify-center relative border border-solid border-[var(--fg-main)] bg-[var(--bg-secondary)] aspect-[1] overflow-hidden shrink-0",
                    img {
                        class: "w-full h-full object-cover",
                        src: "{default_avatar_for(&shown_name)}",
                        alt: s.livechat.avatar_alt,
                    }
                }

                div { class: "flex flex-col items-start gap-2.5 pb-2.5 flex-1 grow border-b border-solid border-[var(--fg-secondary)] justify-center relative",
                    div { class: "flex w-full flex-row gap-2.5",
                        label { class: "sr-only", r#for: "username", "{s.livechat.username_label}" }
                        input {
                            class: "relative w-full flex items-center self-stretch opacity-50 focus-within:opacity-100 [font:'JetBrains_Mono-ExtraLight',Helvetica] font-extralight text-[var(--fg-main)] text-xs tracking-[0] leading-[normal] appearance-none bg-[var(--bg-secondary)] border-0 outline-none p-0",
                            id: "username",
                            name: "username",
                            r#type: "text",
                            maxlength: "{MAX_USERNAME_CHARS}",
                            placeholder: s.livechat.username_placeholder,
                            autocomplete: "username",
                            value: "{chat.username}",
                            oninput: move |evt: FormEvent| chat.username.set(evt.value()),
                        }
                        label { class: "sr-only", r#for: "url", "{s.livechat.url_label}" }
                        input {
                            class: "relative w-full flex items-center self-stretch opacity-50 focus-within:opacity-100 [font:'JetBrains_Mono-ExtraLight',Helvetica] font-extralight text-[var(--fg-main)] text-xs tracking-[0] leading-[normal] appearance-none bg-[var(--bg-secondary)] border-0 outline-none p-0",
                            id: "url",
                            name: "url",
                            r#type: "text",
                            inputmode: "url",
                            maxlength: "{MAX_LINK_CHARS}",
                            placeholder: s.livechat.url_placeholder,
                            autocomplete: "url",
                            value: "{chat.url}",
                            oninput: move |evt: FormEvent| chat.url.set(evt.value()),
                        }
                    }

                    div { class: "flex w-full flex-row",
                        label { class: "sr-only", r#for: "message", "{s.livechat.message_label}" }
                        textarea {
                            class: "relative w-full flex items-center self-stretch opacity-50 focus-within:opacity-100 [font:'JetBrains_Mono-ExtraLight',Helvetica] font-extralight text-[var(--fg-main)] text-xs tracking-[0] leading-[normal] appearance-none bg-[var(--bg-secondary)] border-0 outline-none p-0 resize-none max-h-[calc(1.4em*5)] overflow-y-hidden",
                            id: "message",
                            name: "message",
                            rows: "1",
                            maxlength: "{MAX_MESSAGE_CHARS}",
                            placeholder: s.livechat.message_placeholder,
                            value: "{chat.draft}",
                            oninput: move |evt: FormEvent| {
                                chat.draft.set(evt.value());
                                eval_js(RESIZE_MESSAGE_BOX_JS.to_string());
                            },
                            // Enter = send, Shift+Enter = new line
                            onkeydown: move |evt: KeyboardEvent| match evt.key() {
                                Key::Enter if !evt.modifiers().shift() => {
                                    evt.prevent_default();
                                    chat.submit();
                                }
                                _ => {}
                            },
                        }
                    }
                }

                button {
                    r#type: "submit",
                    disabled: sending,
                    class: "group text-[10px] sm:text-xs ml-2.5 inline-flex bg-[var(--bg-secondary)] border-0 hover:bg-[var(--fg-main)] items-center text-[var(--fg-main)] hover:text-[var(--bg-secondary)] gap-2.5 sm:ml-5 p-2.5 flex-[0_0_auto] justify-center relative cursor-pointer disabled:opacity-50 disabled:cursor-default",
                    if sending {
                        "{s.livechat.sending}"
                    } else {
                        "{s.livechat.send}"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
