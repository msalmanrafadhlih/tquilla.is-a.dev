use dioxus::prelude::*;
use crate::platform::Date;
use serde::Deserialize;

use crate::nixdesk::js_util::eval_js;

const CHAT_JSON: &str = include_str!("../../../data/chat_sample.json");

const AVATAR_1: Asset = asset!("/assets/profile_default_01.svg");
const AVATAR_2: Asset = asset!("/assets/profile_default_02.svg");
const AVATAR_3: Asset = asset!("/assets/profile_default_03.svg");
const AVATAR_4: Asset = asset!("/assets/profile_default_04.svg");
const AVATAR_5: Asset = asset!("/assets/profile_default_05.svg");
const ICON_ADD: Asset = asset!("/assets/icon-add.svg");

/// Class dasar setiap baris pesan, sama dengan `<article>` di HTML.
const ARTICLE_BASE_CLASS: &str = "flex items-start gap-5 p-2.5 relative self-stretch w-full flex-[0_0_auto] border-b [border-bottom-style:solid] border-[var(--variable-collection-fg-main)]";
/// Gradasi selang-seling: pesan ganjil hitam -> abu, genap abu -> hitam.
const BG_BLACK_TO_GRAY: &str = "[background:linear-gradient(90deg,rgba(0,0,0,1)_0%,rgba(37,37,37,1)_100%)]";
const BG_GRAY_TO_BLACK: &str = "[background:linear-gradient(90deg,rgba(37,37,37,1)_0%,rgba(0,0,0,1)_100%)]";

/// Menyesuaikan tinggi textarea dengan isinya (max 5 baris, lalu scroll).
const RESIZE_MESSAGE_BOX_JS: &str = "const el = document.getElementById('message'); if (el) { el.style.height = 'auto'; const h = el.scrollHeight; el.style.height = h + 'px'; const max = parseFloat(getComputedStyle(el).maxHeight); el.style.overflowY = h > max ? 'auto' : 'hidden'; }";

/// Setelah kirim: scroll list ke bawah dan reset tinggi textarea.
const AFTER_SEND_JS: &str = "setTimeout(() => { const list = document.getElementById('livechat-scroll'); if (list) list.scrollTop = list.scrollHeight; const el = document.getElementById('message'); if (el) { el.style.height = 'auto'; el.style.height = el.scrollHeight + 'px'; el.style.overflowY = 'hidden'; } }, 0);";

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct SeedMessage {
    message_id: u32,
    username: String,
    img_profile: Option<String>,
    message: String,
    timestamp: String,
}

#[derive(Debug, Clone, PartialEq)]
struct ChatMessage {
    id: u32,
    username: String,
    img: Option<String>,
    message: String,
    timestamp: String,
}

fn load_seed_messages() -> Vec<ChatMessage> {
    // Jaga-jaga kalau file JSON-nya dikasih baris komentar `//` di awal,
    // sama kayak pola di loader embience.json.
    let cleaned: String = CHAT_JSON
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    let seeds: Vec<SeedMessage> = serde_json::from_str(&cleaned).unwrap_or_default();
    seeds
        .into_iter()
        .map(|s| ChatMessage {
            id: s.message_id,
            username: s.username,
            img: s.img_profile,
            message: s.message,
            timestamp: s.timestamp,
        })
        .collect()
}

fn avatar_for(id: u32) -> Asset {
    match id % 5 {
        0 => AVATAR_1,
        1 => AVATAR_2,
        2 => AVATAR_3,
        3 => AVATAR_4,
        _ => AVATAR_5,
    }
}

/// Duplicated from `tty/chat_preview.rs` — small enough that a shared
/// module across Terminal and Desktop wasn't worth the cross-feature
/// coupling.
fn relative_time(iso: &str) -> String {
    let then = Date::from_iso(iso).get_time();
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

/// Appends a new local message (session-only — there's no backend here,
/// same honest scope as the rest of this portfolio simulation) and
/// scrolls the list down. A plain function taking signals as explicit
/// parameters, not a shared closure, so it can be called from both the
/// Enter-key handler and the form submit without any capture sharing.
fn send_message(
    mut messages: Signal<Vec<ChatMessage>>,
    mut draft: Signal<String>,
    mut next_id: Signal<u32>,
    username: String,
) {
    let text = draft();
    if text.trim().is_empty() || username.trim().is_empty() {
        return;
    }
    let now = Date::new_0().to_iso_string();
    let id = next_id();
    next_id.set(id + 1);
    let mut list = messages();
    list.push(ChatMessage { id, username, img: None, message: text, timestamp: now });
    messages.set(list);
    draft.set(String::new());
    eval_js(AFTER_SEND_JS.to_string());
}

#[component]
pub fn LiveChatWindowContent() -> Element {
    let messages = use_signal(load_seed_messages);
    let mut username = use_signal(String::new);
    let mut draft = use_signal(String::new);
    let mut next_id = use_signal(|| 1000u32);

    rsx! {
        div { class: "flex flex-col items-center gap-5 h-full w-full bg-transparent text-sm",

            // MAIN CONTENTS : LIVE CHAT
            form {
                aria_label: "Message submission form",
                class: "flex w-full items-center justify-center relative",
                onsubmit: move |evt: FormEvent| {
                    evt.prevent_default();
                    send_message(messages, draft, next_id, username());
                },

                // Tombol tambah gambar (input file disembunyikan, dipicu oleh label)
                label {
                    r#for: "image-upload",
                    aria_label: "Add image file",
                    class: "group flex flex-col w-[50px] h-[50px] ml-2.5 mr-5 items-center justify-center gap-2.5 relative border border-solid text-[var(--variable-collection-fg-main)] bg-transparent border-[var(--variable-collection-fg-main)] aspect-[1] cursor-pointer hover:text-[var(--variable-collection-bg-main)] hover:bg-[var(--variable-collection-fg-main)]",
                    img {
                        class: "group-hover:invert ",
                        title: "Add Icon",
                        "aria-hidden": "true",
                        src: ICON_ADD,
                    }
                }
                // TODO: handler upload gambar belum ada di versi Rust
                input {
                    r#type: "file",
                    id: "image-upload",
                    accept: "image/*",
                    class: "hidden",
                }

                div { class: "flex flex-col items-start gap-2.5 pb-2.5 flex-1 grow border-b border-solid border-[var(--variable-collection-fg-secondary)] justify-center relative",
                    label { class: "sr-only", r#for: "username", "Username" }
                    input {
                        class: "relative w-full flex items-center self-stretch mt-[-1.00px] opacity-50 [font:'JetBrains_Mono-ExtraLight',Helvetica] font-extralight text-[var(--variable-collection-fg-main)] text-xs tracking-[0] leading-[normal] appearance-none bg-transparent border-0 outline-none p-0",
                        id: "username",
                        name: "username",
                        r#type: "text",
                        placeholder: "Set username...",
                        autocomplete: "username",
                        value: "{username}",
                        oninput: move |evt: FormEvent| username.set(evt.value()),
                    }

                    label { class: "sr-only", r#for: "message", "Message" }
                    textarea {
                        class: "relative w-full flex items-center self-stretch opacity-50 [font:'JetBrains_Mono-ExtraLight',Helvetica] font-extralight text-[var(--variable-collection-fg-main)] text-xs tracking-[0] leading-[normal] appearance-none bg-transparent border-0 outline-none p-0 resize-none max-h-[calc(1.4em*5)] overflow-y-hidden",
                        id: "message",
                        name: "message",
                        rows: "1",
                        placeholder: "Type your message here...",
                        value: "{draft}",
                        oninput: move |evt: FormEvent| {
                            draft.set(evt.value());
                            eval_js(RESIZE_MESSAGE_BOX_JS.to_string());
                        },
                        // Enter = kirim, Shift+Enter = baris baru
                        onkeydown: move |evt: KeyboardEvent| match evt.key() {
                            Key::Enter if !evt.modifiers().shift() => {
                                evt.prevent_default();
                                send_message(messages, draft, next_id, username());
                            }
                            _ => {}
                        },
                    }
                }

                button {
                    r#type: "Send",
                    class: "group text-[10px] sm:text-xs inline-flex bg-transparent border-0 hover:bg-[var(--variable-collection-fg-main)] items-center text-[var(--variable-collection-fg-secondary)] hover:text-[var(--variable-collection-bg-main)] gap-2.5 sm:ml-5 p-2.5 flex-[0_0_auto] justify-center relative cursor-pointer",
                    "Send"
                }
            }

            // CHAT ENVIRONMENTS
            section {
                id: "livechat-scroll",
                class: "flex flex-col items-center relative flex-1 min-h-0 w-full overflow-y-scroll [scrollbar-width:none] [&::-webkit-scrollbar]:hidden border-t border-solid border-[var(--variable-collection-fg-secondary)]",

                for (idx, msg) in messages().into_iter().enumerate() {
                    {
                        let bg = if idx % 2 == 0 { BG_BLACK_TO_GRAY } else { BG_GRAY_TO_BLACK };
                        let row_class = format!("{ARTICLE_BASE_CLASS} {bg}");
                        rsx! {
                            article {
                                key: "{msg.id}",
                                class: "{row_class}",
                                aria_labelledby: "post-author-{msg.id}",

                                div { class: "flex flex-col w-[50px] h-[50px] items-center justify-center gap-2.5 relative border border-solid border-[var(--variable-collection-fg-main)] aspect-[1]",
                                    img {
                                        class: "relative self-stretch w-full aspect-[1]",
                                        src: if let Some(path) = &msg.img { path.clone() } else { avatar_for(msg.id).to_string() },
                                        alt: "{msg.username}",
                                    }
                                }

                                div { class: "flex-col pt-0 pb-2.5 px-0 flex-1 grow flex items-center gap-2.5 relative self-stretch",
                                    header { class: "justify-center w-full flex-[0_0_auto] flex items-center gap-2.5 relative self-stretch",
                                        h2 {
                                            id: "post-author-{msg.id}",
                                            class: "relative flex items-end w-fit mt-[-1.00px] [font:'JetBrains_Mono-Regular',Helvetica] font-normal text-[var(--variable-collection-fg-main)] text-xs tracking-[0] leading-[normal]",
                                            "{msg.username}"
                                        }
                                        time {
                                            datetime: "{msg.timestamp}",
                                            class: "relative flex items-end flex-1 [font:'JetBrains_Mono-Regular',Helvetica] font-normal text-[var(--variable-collection-fg-secondary)] text-[10px] tracking-[0] leading-[normal]",
                                            "{relative_time(&msg.timestamp)}"
                                        }
                                    }
                                    p { class: "relative flex items-end self-stretch opacity-50 [font:'JetBrains_Mono-Regular',Helvetica] font-normal text-[var(--variable-collection-fg-main)] text-xs tracking-[0] leading-[normal]",
                                        "{msg.message}"
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
