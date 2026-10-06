use dioxus::html::FileData;
use dioxus::prelude::*;
use crate::platform::Date;
use serde::Deserialize;
use base64::{engine::general_purpose::STANDARD, Engine as _};

use crate::nixdesk::js_util::eval_js;

const CHAT_JSON: &str = include_str!("../../../data/chat_sample.json");

const AVATAR_1: Asset = asset!("/assets/profile_default_01.svg");
const AVATAR_2: Asset = asset!("/assets/profile_default_02.svg");
const AVATAR_3: Asset = asset!("/assets/profile_default_03.svg");
const AVATAR_4: Asset = asset!("/assets/profile_default_04.svg");
const AVATAR_5: Asset = asset!("/assets/profile_default_05.svg");
const ICON_ADD_MEDIA: Asset = asset!("/assets/icon-add-media.svg");
const ICON_ADD: Asset = asset!("/assets/icon-add.svg");

const MAX_AVATAR_BYTES: u64 = 1024 * 1024;
const MAX_ATTACHMENT_BYTES: u64 = 5 * 1024 * 1024;

const ARTICLE_BASE_CLASS: &str = "flex items-start gap-5 p-2.5 relative self-stretch w-full flex-[0_0_auto] border-b [border-bottom-style:solid] border-[var(--fg-main)]";
const AUTHOR_CLASS: &str = "relative flex items-end w-fit mt-[-1.00px] [font:'JetBrains_Mono-Regular',Helvetica] font-normal text-[var(--fg-main)] text-xs tracking-[0] leading-[normal]";
const BG_BLACK_TO_GRAY: &str = "bg-[image:var(--linear-l)]";
const BG_GRAY_TO_BLACK: &str = "bg-[image:var(--linear-r)]";
const RESIZE_MESSAGE_BOX_JS: &str = "const el = document.getElementById('message'); if (el) { el.style.height = 'auto'; const h = el.scrollHeight; el.style.height = h + 'px'; const max = parseFloat(getComputedStyle(el).maxHeight); el.style.overflowY = h > max ? 'auto' : 'hidden'; }";
const AFTER_SEND_JS: &str = "setTimeout(() => { const list = document.getElementById('livechat-scroll'); if (list) list.scrollTop = list.scrollHeight; const el = document.getElementById('message'); if (el) { el.style.height = 'auto'; el.style.height = el.scrollHeight + 'px'; el.style.overflowY = 'hidden'; } }, 0);";
const SCROLL_BOTTOM_JS: &str = "const list = document.getElementById('livechat-scroll'); if (list) list.scrollTop = list.scrollHeight;";

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct SeedMessage {
    message_id: u32,
    username: String,
    url: Option<String>,
    avatar: Option<String>,
    message: String,
    attachment: Option<String>,
    timestamp: String,
}

#[derive(Debug, Clone, PartialEq)]
struct ChatMessage {
    id: u32,
    username: String,
    url: Option<String>,
    avatar: Option<String>,
    message: String,
    timestamp: String,
    attachment: Option<String>,
}

/// Normalisasi URL dari input pengguna. Hasilnya SELALU diawali `http://`
/// atau `https://`, jadi aman dipakai sebagai `href` (tidak mungkin
/// `javascript:` / `data:`). Input kosong atau berisi spasi -> `None`.
fn normalize_url(raw: &str) -> Option<String> {
    let t = raw.trim();
    if t.is_empty() || t.contains(char::is_whitespace) {
        return None;
    }
    let lower = t.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        Some(t.to_string())
    } else {
        Some(format!("https://{t}"))
    }
}

fn load_seed_messages() -> Vec<ChatMessage> {
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
            url: s.url.as_deref().and_then(normalize_url),
            avatar: s.avatar,
            message: s.message,
            timestamp: s.timestamp,
            attachment: s.attachment,
        })
        .collect()
}

/// Avatar default "acak" tapi stabil: dipilih dari hash username, jadi
/// pengguna yang sama selalu dapat avatar yang sama di semua pesannya.
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

/// Baca file gambar -> validasi (tipe & ukuran) -> data URL base64.
async fn read_image_data_url(file: FileData, max_bytes: u64) -> Result<String, String> {
    let mime = file.content_type().unwrap_or_default();
    if !mime.starts_with("image/") {
        return Err("File harus berupa gambar.".to_string());
    }
    if file.size() > max_bytes {
        return Err(format!(
            "Ukuran gambar maksimal {} MB.",
            max_bytes / (1024 * 1024)
        ));
    }
    let bytes = file
        .read_bytes()
        .await
        .map_err(|_| "Gagal membaca file gambar.".to_string())?;
    Ok(format!("data:{mime};base64,{}", STANDARD.encode(&bytes)))
}

/// Kosongkan `<input type=file>` supaya memilih file yang sama dua kali
/// tetap memicu event `change`.
fn clear_file_input_js(id: &str) -> String {
    format!("const el = document.getElementById('{id}'); if (el) el.value = '';")
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
///
/// Pesan boleh berisi teks saja, gambar saja, atau keduanya.
fn send_message(
    mut messages: Signal<Vec<ChatMessage>>,
    mut draft: Signal<String>,
    mut next_id: Signal<u32>,
    mut attachment: Signal<Option<String>>,
    username: String,
    url: String,
    avatar: Option<String>,
) {
    let text = draft();
    let image = attachment();
    if (text.trim().is_empty() && image.is_none()) || username.trim().is_empty() {
        return;
    }
    let now = Date::new_0().to_iso_string();
    let id = next_id();
    next_id.set(id + 1);
    let mut list = messages();
    list.push(ChatMessage {
        id,
        username,
        url: normalize_url(&url),
        avatar,
        message: text,
        timestamp: now,
        attachment: image,
    });
    messages.set(list);
    draft.set(String::new());
    attachment.set(None);
    eval_js(AFTER_SEND_JS.to_string());
}

#[component]
pub fn LiveChatWindowContent() -> Element {
    let messages = use_signal(load_seed_messages);
    let mut username = use_signal(String::new);
    let mut url = use_signal(String::new);
    let mut draft = use_signal(String::new);
    let mut next_id = use_signal(|| 1000u32);
    let mut avatar = use_signal(|| None::<String>);
    let mut attachment = use_signal(|| None::<String>);
    let mut upload_error = use_signal(|| None::<String>);

    rsx! {
        div { class: "flex flex-col items-center gap-5 h-full w-full bg-[var(--bg-secondary)] text-sm",

            // CHAT ENVIRONMENTS
            section {
                id: "livechat-scroll",
                class: "flex flex-col items-center relative flex-1 min-h-0 w-full overflow-y-scroll [scrollbar-width:none] [&::-webkit-scrollbar]:hidden border-t border-solid border-[var(--fg-secondary)]",

                for (idx, msg) in messages().into_iter().enumerate() {
                    {
                        let bg = if idx % 2 == 0 { BG_BLACK_TO_GRAY } else { BG_GRAY_TO_BLACK };
                        let row_class = format!("{ARTICLE_BASE_CLASS} {bg}");
                        // Avatar pengguna kalau ada, kalau tidak -> avatar default.
                        let avatar_src = match &msg.avatar {
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
                                                class: "{AUTHOR_CLASS} hover:underline",
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
                                            datetime: "{msg.timestamp}",
                                            class: "relative flex items-end flex-1 [font:'JetBrains_Mono-Regular',Helvetica] font-normal text-[var(--fg-secondary)] text-[10px] tracking-[0] leading-[normal]",
                                            "{relative_time(&msg.timestamp)}"
                                        }
                                    }
                                    if !msg.message.is_empty() {
                                        p { class: "relative flex items-end self-stretch opacity-50 [font:'JetBrains_Mono-Regular',Helvetica] font-normal text-[var(--fg-main)] text-xs tracking-[0] leading-[normal]",
                                            "{msg.message}"
                                        }
                                    }
                                    // Gambar lampiran (kalau ada)
                                    if let Some(src) = &msg.attachment {
                                        img {
                                            class: "self-start max-w-full max-h-60 object-contain border-0",
                                            src: "{src}",
                                            alt: "Lampiran dari {msg.username}",
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

            // PREVIEW LAMPIRAN + PESAN ERROR UPLOAD
            if let Some(src) = attachment() {
                div { class: "flex w-full items-start gap-2.5 px-2.5",
                    div { class: "relative border border-solid border-[var(--fg-main)]",
                        img {
                            class: "block max-w-[120px] max-h-[120px] object-contain",
                            src: "{src}",
                            alt: "Pratinjau gambar yang akan dikirim",
                        }
                        button {
                            r#type: "button",
                            aria_label: "Remove attached image",
                            title: "Remove image",
                            class: "absolute top-0 right-0 w-5 h-5 flex items-center justify-center text-[10px] bg-[var(--bg-secondary)] text-[var(--fg-main)] border border-solid border-[var(--fg-main)] cursor-pointer hover:bg-[var(--fg-main)] hover:text-[var(--bg-secondary)]",
                            onclick: move |_| attachment.set(None),
                            "x"
                        }
                    }
                }
            }
            if let Some(err) = upload_error() {
                p {
                    role: "alert",
                    class: "w-full px-2.5 text-[10px] text-[var(--red)] [font:'JetBrains_Mono-Regular',Helvetica]",
                    "{err}"
                }
            }

            // MAIN CONTENTS : LIVE CHAT
            form {
                aria_label: "Message submission form",
                class: "flex w-full items-center justify-center relative",
                onsubmit: move |evt: FormEvent| {
                    evt.prevent_default();
                    send_message(messages, draft, next_id, attachment, username(), url(), avatar());
                },

                // Tombol upload avatar profil (input file disembunyikan, dipicu oleh label).
                // Sebelum upload: ikon "+". Sesudah upload: avatar pengguna sendiri.
                label {
                    r#for: "avatar-upload",
                    aria_label: "Upload avatar",
                    class: "group flex flex-col w-[50px] h-[50px] ml-2.5 mr-5 items-center justify-center gap-2.5 relative border border-solid text-[var(--fg-main)] bg-[var(--bg-secondary)] border-[var(--fg-main)] aspect-[1] cursor-pointer overflow-hidden hover:text-[var(--bg-secondary)] hover:bg-[var(--fg-main)]",
                    if let Some(src) = avatar() {
                        img {
                            class: "w-full h-full object-cover",
                            title: "Change avatar",
                            src: "{src}",
                            alt: "Your avatar",
                        }
                    } else {
                        img {
                            class: "group-hover:invert ",
                            title: "Add avatar",
                            "aria-hidden": "true",
                            src: ICON_ADD,
                        }
                    }
                }
                input {
                    r#type: "file",
                    id: "avatar-upload",
                    accept: "image/*",
                    class: "hidden",
                    onchange: move |evt: FormEvent| async move {
                        if let Some(file) = evt.files().into_iter().next() {
                            match read_image_data_url(file, MAX_AVATAR_BYTES).await {
                                Ok(data_url) => {
                                    avatar.set(Some(data_url));
                                    upload_error.set(None);
                                }
                                Err(msg) => upload_error.set(Some(msg)),
                            }
                        }
                        eval_js(clear_file_input_js("avatar-upload"));
                    },
                }

                div { class: "flex flex-col items-start gap-2.5 pb-2.5 flex-1 grow border-b border-solid border-[var(--fg-secondary)] justify-center relative",
                    div { class: "flex w-full flex-row gap-2.5",
                        label { class: "sr-only", r#for: "username", "Username" }
                        input {
                            class: "relative w-full flex items-center self-stretch mt-[-1.00px] opacity-50 [font:'JetBrains_Mono-ExtraLight',Helvetica] font-extralight text-[var(--fg-main)] text-xs tracking-[0] leading-[normal] appearance-none bg-[var(--bg-secondary)] border-0 outline-none p-0",
                            id: "username",
                            name: "username",
                            r#type: "text",
                            placeholder: "Set username",
                            autocomplete: "username",
                            value: "{username}",
                            oninput: move |evt: FormEvent| username.set(evt.value()),
                        }
                        label { class: "sr-only", r#for: "url", "Website URL" }
                        input {
                            class: "relative w-full flex items-center self-stretch mt-[-1.00px] opacity-50 [font:'JetBrains_Mono-ExtraLight',Helvetica] font-extralight text-[var(--fg-main)] text-xs tracking-[0] leading-[normal] appearance-none bg-[var(--bg-secondary)] border-0 outline-none p-0",
                            id: "url",
                            name: "url",
                            r#type: "text",
                            inputmode: "url",
                            placeholder: "url (optional)",
                            autocomplete: "url",
                            value: "{url}",
                            oninput: move |evt: FormEvent| url.set(evt.value()),
                        }
                    }

                    div { class: "flex w-full flex-row",
                        label { class: "sr-only", r#for: "message", "Message" }
                        textarea {
                            class: "relative w-full flex items-center self-stretch opacity-50 [font:'JetBrains_Mono-ExtraLight',Helvetica] font-extralight text-[var(--fg-main)] text-xs tracking-[0] leading-[normal] appearance-none bg-[var(--bg-secondary)] border-0 outline-none p-0 resize-none max-h-[calc(1.4em*5)] overflow-y-hidden",
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
                                    send_message(messages, draft, next_id, attachment, username(), url(), avatar());
                                }
                                _ => {}
                            },
                        }

                        // Tombol tambah gambar media ke pesan (input file disembunyikan, dipicu oleh label)
                        label {
                            r#for: "image-upload",
                            aria_label: "Add image file",
                            class: "flex flex-col h-5 items-center justify-center gap-2.5 relative bg-[var(--bg-secondary)] aspect-[1] cursor-pointer opacity-50 hover:opacity-100",
                            img {
                                title: "Add Media",
                                "aria-hidden": "true",
                                src: ICON_ADD_MEDIA,
                            }
                        }
                        input {
                            r#type: "file",
                            id: "image-upload",
                            accept: "image/*",
                            class: "hidden",
                            onchange: move |evt: FormEvent| async move {
                                if let Some(file) = evt.files().into_iter().next() {
                                    match read_image_data_url(file, MAX_ATTACHMENT_BYTES).await {
                                        Ok(data_url) => {
                                            attachment.set(Some(data_url));
                                            upload_error.set(None);
                                        }
                                        Err(msg) => upload_error.set(Some(msg)),
                                    }
                                }
                                eval_js(clear_file_input_js("image-upload"));
                            },
                        }
                    }
                }

                button {
                    r#type: "submit",
                    class: "group text-[10px] sm:text-xs inline-flex bg-[var(--bg-secondary)] border-0 hover:bg-[var(--fg-main)] items-center text-[var(--fg-secondary)] hover:text-[var(--bg-secondary)] gap-2.5 sm:ml-5 p-2.5 flex-[0_0_auto] justify-center relative cursor-pointer",
                    "Send"
                }
            }
        }
    }
}
