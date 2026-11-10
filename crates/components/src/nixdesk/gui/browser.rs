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
    const TAB_ACTIVE_CLASS: &str = "browser__tab browser__tab--active";
    const TAB_INACTIVE_CLASS: &str = "browser__tab browser__tab--idle";

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
            SkeletonGroup { class: "app-window__loading",
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
            class: "browser",

            // sidebar: tabs + input
            section {
                class: "browser__sidebar",

                // list tabs
                div {
                    class: "browser__tabs",
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
                        class: "browser__form-toggle",
                        onclick: move |_| form_open.set(!form_open()),
                        // or: onclick: move |_| form_open.toggle(),
                        "+"
                    }
                }

                div {
                    class: if form_open() { "browser__form browser__form--open" } else { "browser__form browser__form--closed" },
                    div {
                        class: "browser__row",
                        input {
                            id: NEW_LABEL_ID,
                            r#type: "text",
                            placeholder: s.browser.name_placeholder,
                            class: "browser__url",
                            oninput: move |evt: FormEvent| new_label.set(evt.value()),
                        }
                        button {
                            r#type: "button",
                            disabled: true,
                            class: "browser__url-submit",
                            "+"
                        }
                    }
                    div {
                        class: "browser__row",
                        input {
                            id: NEW_URL_ID,
                            r#type: "text",
                            placeholder: "https://...",
                            class: "browser__search",
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
                            class: "browser__search-submit",
                            onclick: move |_| add_bookmark(bookmarks, selected, new_label, new_url),
                            "+"
                        }
                    }
                }
            }

            // main content: url bar + iframe + footer link
            section {
                class: "browser__content",

                div {
                    class: "browser__title",
                    img {
                        src: ICON_LINK,
                        alt: s.browser.link_alt,
                        class: "browser__favicon",
                    }
                    "{current.url}"
                }

                div {
                    class: "browser__frame-box",
                    iframe {
                        src: "{current.url}",
                        class: "browser__frame",
                        title: "{current.label}",
                    }
                }

                div {
                    class: "browser__footer",
                    a {
                        target: "_blank",
                        rel: "noopener noreferrer",
                        href: "{current.url}",
                        class: "browser__open-link",
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
