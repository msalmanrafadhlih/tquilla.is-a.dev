use api_types::validate::is_http_url;
use dioxus::prelude::*;

use crate::api::{self, Bookmark};
use crate::languages::use_strings;
use crate::nixdesk::js_util::set_field;
use crate::remote::LoadFailed;
use crate::skeleton::{SkeletonGroup, SkeletonRows};

const ICON_LINK: Asset = asset!("/assets/icon-url.svg");
const NEW_LABEL_ID: &str = "browser-new-label";
const NEW_URL_ID: &str = "browser-new-url";

/// What the user typed becomes a tab only if it is an `http(s)` address
/// (`https://` is added when the scheme is missing). This also keeps
/// `javascript:` and `data:` out of the iframe.
fn normalize_user_url(input: &str) -> Option<String> {
    let input = input.trim();
    let url = if input.starts_with("http://") || input.starts_with("https://") {
        input.to_owned()
    } else {
        format!("https://{input}")
    };
    is_http_url(&url).then_some(url)
}

/// Session-only: tabs the user adds are not saved or sent anywhere.
fn add_bookmark(
    mut bookmarks: Signal<Vec<Bookmark>>,
    mut selected: Signal<usize>,
    mut new_label: Signal<String>,
    mut new_url: Signal<String>,
) {
    let label = new_label().trim().to_string();
    let Some(url) = normalize_user_url(&new_url()) else {
        return;
    };
    if label.is_empty() {
        return;
    }

    let mut list = bookmarks();
    // `id: 0` marks a tab that does not exist on the server.
    list.push(Bookmark { id: 0, label, url });
    let idx = list.len() - 1;
    bookmarks.set(list);
    selected.set(idx);
    new_label.set(String::new());
    new_url.set(String::new());
    // DOM pemilik teks (tanpa `value`), lihat `js_util::set_field`.
    set_field(NEW_LABEL_ID, "", None);
    set_field(NEW_URL_ID, "", None);
}

fn tab_class(active: bool) -> &'static str {
    const TAB_ACTIVE_CLASS: &str = "w-max @md:w-full h-max text-left px-3 py-1.5 truncate bg-[var(--fg-main)] text-xs text-[var(--bg-secondary)] border-0";
    const TAB_INACTIVE_CLASS: &str = "w-max @md:w-full h-max text-left px-3 py-1.5 truncate text-[var(--fg-secondary)] text-xs hover:text-white bg-[var(--bg-secondary)] border-0";

    if active {
        TAB_ACTIVE_CLASS
    } else {
        TAB_INACTIVE_CLASS
    }
}

/// Loads the bookmarks, then hands them to [`BrowserBoard`].
#[component]
pub fn BrowserWindowContent() -> Element {
    let mut resource = use_resource(api::bookmarks);
    let state = resource.read_unchecked();
    match &*state {
        None => rsx! {
            SkeletonGroup { class: "p-3",
                SkeletonRows { rows: 5 }
            }
        },
        Some(Err(e)) => rsx! {
            LoadFailed { error: e.clone(), on_retry: move |_| resource.restart() }
        },
        Some(Ok(list)) => rsx! {
            BrowserBoard { initial: list.clone() }
        },
    }
}

#[component]
fn BrowserBoard(initial: Vec<Bookmark>) -> Element {
    let s = use_strings();
    let bookmarks = use_signal(move || initial.clone());
    let mut selected = use_signal(|| 0usize);
    let mut new_label = use_signal(String::new);
    let mut new_url = use_signal(String::new);
    let mut form_open = use_signal(|| false);

    let list = bookmarks();
    let current = list.get(selected()).cloned().unwrap_or(Bookmark {
        id: 0,
        label: String::new(),
        url: String::new(),
    });

    rsx! {
        div {
            class: "flex flex-col @md:flex-row min-h-0 h-full w-full overflow-auto [-webkit-touch-callout:none] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden",

            // sidebar: tabs + input
            section {
                class: "w-full h-max @md:h-full @md:max-w-[200px] max-w-full shrink-0 border-r border-solid gap-2 p-2 items-center justify-between border-[var(--fg-main)] flex flex-col",

                // list tabs
                div {
                    class: "flex gap-1 min-h-0 h-max w-full @md:max-w-full overflow-y-auto flex-row @md:flex-col",
                    for (idx, bm) in list.iter().enumerate() {
                        button {
                            key: "{bm.label}-{idx}",
                            r#type: "button",
                            class: tab_class(selected() == idx),
                            onclick: move |_| selected.set(idx),
                            "{bm.label}"
                        }
                    }

                    button {
                        r#type: "button",
                        class: "@md:hidden px-2 block text-[var(--fg-secondary)] hover:text-[var(--fg-main)] shrink-0 bg-[var(--bg-secondary)] border-0",
                        onclick: move |_| form_open.set(!form_open()),
                        // or: onclick: move |_| form_open.toggle(),
                        "+"
                    }
                }

                div {
                    class: if form_open() { "w-full flex flex-row @md:flex-col gap-2" } else { "hidden w-full @md:flex flex-col gap-2" },
                    div {
                        class: "w-full flex items-center gap-2",
                        input {
                            id: NEW_LABEL_ID,
                            r#type: "text",
                            placeholder: s.browser.name_placeholder,
                            class: "w-full py-2 bg-[var(--bg-secondary)] outline-none border-0 border-b border-[var(--fg-secondary)] text-[var(--fg-main)] placeholder-[var(--fg-secondary)] text-xs",
                            oninput: move |evt: FormEvent| new_label.set(evt.value()),
                        }
                        button {
                            r#type: "button",
                            disabled: true,
                            class: "px-2 @md:pl-2 hidden @md:flex text-transparent shrink-0 bg-[var(--bg-secondary)] border-0",
                            "+"
                        }
                    }
                    div {
                        class: "w-full flex items-center gap-2",
                        input {
                            id: NEW_URL_ID,
                            r#type: "text",
                            placeholder: "https://...",
                            class: "w-full h-full py-2 flex-1 min-w-0 bg-[var(--bg-secondary)] outline-none border-0 border-b border-[var(--fg-secondary)] text-[var(--fg-main)] placeholder-[var(--fg-secondary)] text-xs",
                            onkeydown: move |evt: KeyboardEvent| {
                                if evt.key() == Key::Enter {
                                    evt.prevent_default();
                                    add_bookmark(bookmarks, selected, new_label, new_url);
                                }
                            },
                            oninput: move |evt: FormEvent| new_url.set(evt.value()),
                        }
                        button {
                            r#type: "button",
                            class: "px-2 @md:pl-2 text-[var(--fg-secondary)] hover:text-[var(--fg-main)] shrink-0 bg-[var(--bg-secondary)] border-0",
                            onclick: move |_| add_bookmark(bookmarks, selected, new_label, new_url),
                            "+"
                        }
                    }
                }
            }

            // main content: url bar + iframe + footer link
            section {
                class: "flex-1 min-w-0 flex gap-2 flex-col p-2",

                div {
                    class: "flex shrink-0 items-center border-b border-solid border-[var(--fg-secondary)] px-3 pb-2 truncate text-[var(--fg-secondary)] gap-2 text-xs",
                    img {
                        src: ICON_LINK,
                        alt: s.browser.link_alt,
                        class: "h-4",
                    }
                    "{current.url}"
                }

                div {
                    class: "flex-1 min-h-0 bg-white/[0.02]",
                    iframe {
                        src: "{current.url}",
                        class: "w-full h-full border-0",
                        title: "{current.label}",
                    }
                }

                div {
                    class: "shrink-0 border-t border-solid border-[var(--fg-secondary)] px-3 pt-2 flex justify-end",
                    a {
                        target: "_blank",
                        rel: "noopener noreferrer",
                        href: "{current.url}",
                        class: "text-[var(--fg-secondary)] hover:text-[var(--fg-main)] transition-colors duration-150 text-xs",
                        "{s.browser.go_to_website}"
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
    fn typed_addresses_get_https_and_must_be_web_urls() {
        assert_eq!(
            normalize_user_url("  example.com/a ").as_deref(),
            Some("https://example.com/a")
        );
        assert_eq!(
            normalize_user_url("http://example.com").as_deref(),
            Some("http://example.com")
        );
        // A scheme the user typed is never kept: "javascript:x" turns into
        // "https://javascript:x", a harmless (invalid) web address.
        for input in ["javascript:alert(1)", "data:text/html,x"] {
            let url = normalize_user_url(input).unwrap();
            assert!(url.starts_with("https://"), "{url}");
        }
        assert_eq!(normalize_user_url(""), None);
        assert_eq!(normalize_user_url("has space.com"), None);
        assert_eq!(normalize_user_url("/local"), None);
    }
}
