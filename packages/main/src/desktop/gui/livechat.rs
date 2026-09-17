use dioxus::prelude::*;
use js_sys::Date;
use serde::Deserialize;
use wasm_bindgen::JsValue;

use crate::desktop::js_util::eval_js;

const CHAT_JSON: &str = include_str!("../../../data/chat_sample.json");

const AVATAR_1: Asset = asset!("/assets/profile_default_1.svg");
const AVATAR_2: Asset = asset!("/assets/profile_default_2.svg");
const AVATAR_3: Asset = asset!("/assets/profile_default_3.svg");
const AVATAR_4: Asset = asset!("/assets/profile_default_4.svg");
const AVATAR_5: Asset = asset!("/assets/profile_default_5.svg");

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

/// Appends a new local message (session-only — there's no backend here,
/// same honest scope as the rest of this portfolio simulation) and
/// scrolls the list down. A plain function taking signals as explicit
/// parameters, not a shared closure, so it can be called from both the
/// Enter-key handler and the Submit button without any capture sharing.
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
    let now = Date::new_0().to_iso_string().as_string().unwrap_or_default();
    let id = next_id();
    next_id.set(id + 1);
    let mut list = messages();
    list.push(ChatMessage { id, username, img: None, message: text, timestamp: now });
    messages.set(list);
    draft.set(String::new());
    eval_js(
        "const el = document.getElementById('livechat-scroll'); if (el) el.scrollTop = el.scrollHeight;"
            .to_string(),
    );
}

#[component]
pub fn LiveChatWindowContent() -> Element {
    let messages = use_signal(load_seed_messages);
    let mut username = use_signal(String::new);
    let mut draft = use_signal(String::new);
    let mut next_id = use_signal(|| 1000u32);

    rsx! {
        div { class: "flex flex-col h-full text-[11px]",
            div {
                id: "livechat-scroll",
                class: "flex-1 min-h-0 overflow-y-auto px-3 py-3 flex flex-col gap-3",
                for msg in messages() {
                    div { key: "{msg.id}", class: "flex items-start gap-2",
                        img {
                            src: if let Some(path) = &msg.img { path.clone() } else { avatar_for(msg.id).to_string() },
                            alt: "",
                            class: "w-7 h-7 shrink-0 rounded-full border border-white/15",
                        }
                        div { class: "min-w-0",
                            div { class: "flex items-baseline gap-2",
                                span { class: "font-semibold text-white", "{msg.username}" }
                                span { class: "text-white/30 text-[9px]", "{relative_time(&msg.timestamp)}" }
                            }
                            p { class: "text-white/70 leading-snug break-words", "{msg.message}" }
                        }
                    }
                }
            }
            div { class: "shrink-0 border-t border-white/15 px-3 py-2 flex flex-col gap-1.5",
                input {
                    r#type: "text",
                    placeholder: "Set username...",
                    class: "bg-transparent outline-none border-b border-white/15 pb-1 text-white placeholder-white/30",
                    value: "{username}",
                    oninput: move |evt: FormEvent| username.set(evt.value()),
                }
                div { class: "flex items-center gap-2",
                    input {
                        r#type: "text",
                        placeholder: "Type your message here...",
                        class: "flex-1 bg-transparent outline-none text-white placeholder-white/30",
                        value: "{draft}",
                        onkeydown: move |evt: KeyboardEvent| match evt.key() {
                            Key::Enter => {
                                evt.prevent_default();
                                send_message(messages, draft, next_id, username());
                            }
                            _ => {}
                        },
                        oninput: move |evt: FormEvent| draft.set(evt.value()),
                    }
                    button {
                        r#type: "button",
                        class: "text-white/50 hover:text-white transition-colors duration-150",
                        onclick: move |_| send_message(messages, draft, next_id, username()),
                        "submit"
                    }
                }
            }
        }
    }
}
