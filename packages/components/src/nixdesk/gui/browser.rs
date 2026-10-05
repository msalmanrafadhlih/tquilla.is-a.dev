use dioxus::prelude::*;
use serde::Deserialize;

const BOOKMARKS_JSON: &str = include_str!("../../../data/browser.json");
const ICON_LINK: Asset = asset!("/assets/icon-url.svg");

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct Bookmark {
    placeholder: String,
    url: String,
}

fn load_bookmarks() -> Vec<Bookmark> {
    serde_json::from_str(BOOKMARKS_JSON).unwrap_or_default()
}

/// Session-only — new bookmarks don't persist across a reload, same as
/// this app's other "no real backend" pieces (Live Chat's messages).
fn add_bookmark(
    mut bookmarks: Signal<Vec<Bookmark>>,
    mut selected: Signal<usize>,
    mut new_label: Signal<String>,
    mut new_url: Signal<String>,
) {
    let label = new_label();
    let url = new_url();
    if label.trim().is_empty() || url.trim().is_empty() {
        return;
    }
    let mut list = bookmarks();
    list.push(Bookmark { placeholder: label, url });
    let idx = list.len() - 1;
    bookmarks.set(list);
    selected.set(idx);
    new_label.set(String::new());
    new_url.set(String::new());
}

const SIDEBAR_CLASS: &str = "w-full h-max @md:h-full @md:max-w-[200px] max-w-full shrink-0 border-r border-solid gap-2 p-2 items-center justify-between border-[var(--variable-collection-fg-main)] flex flex-row @md:flex-col";
const TAB_LIST_CLASS: &str = "flex gap-1 min-h-0 h-max w-full @md:max-w-full overflow-y-auto flex-row @md:flex-col";
const TAB_ACTIVE_CLASS: &str = "w-max @md:w-full h-max text-left px-3 py-1.5 truncate bg-[var(--variable-collection-fg-main)] text-xs text-[var(--variable-collection-bg-main)] border-0";
const TAB_INACTIVE_CLASS: &str = "w-max @md:w-full h-max text-left px-3 py-1.5 truncate text-[var(--variable-collection-fg-secondary)] text-xs hover:text-white bg-transparent border-0";
const INPUT_SECTION_CLASS: &str = "w-max @md:w-full h-max pr-3 border-[var(--variable-collection-fg-secondary)] flex flex-col gap-2";
const INPUT_LABEL_CLASS: &str = "hidden @md:block bg-transparent outline-none border-0 border-b border-[var(--variable-collection-fg-secondary)] text-[var(--variable-collection-fg-main)] placeholder-[var(--variable-collection-fg-secondary)] text-xs";
const INPUT_URL_CLASS: &str = "hidden @md:block w-max flex-1 min-w-0 bg-transparent outline-none border-0 border-b border-[var(--variable-collection-fg-secondary)] text-[var(--variable-collection-fg-main)] placeholder-[var(--variable-collection-fg-secondary)] text-xs";
const ADD_BUTTON_CLASS: &str = "text-[var(--variable-collection-fg-secondary)] hover:text-[var(--variable-collection-fg-main)] shrink-0 bg-transparent border-0";
const URL_BAR_CLASS: &str = "flex shrink-0 items-center border-b border-solid border-[var(--variable-collection-fg-secondary)] px-3 pb-2 truncate text-[var(--variable-collection-fg-secondary)] gap-2 text-xs";
const IFRAME_WRAPPER_CLASS: &str = "flex-1 min-h-0 bg-white/[0.02]";
const FOOTER_CLASS: &str = "shrink-0 border-t border-solid border-[var(--variable-collection-fg-secondary)] px-3 pt-2 flex justify-end";
const LINK_CLASS: &str = "text-[var(--variable-collection-fg-secondary)] hover:text-[var(--variable-collection-fg-main)] transition-colors duration-150 text-xs";

fn tab_class(active: bool) -> &'static str {
    if active {
        TAB_ACTIVE_CLASS
    } else {
        TAB_INACTIVE_CLASS
    }
}

#[component]
pub fn BrowserWindowContent() -> Element {
    let bookmarks = use_signal(load_bookmarks);
    let mut selected = use_signal(|| 0usize);
    let mut new_label = use_signal(String::new);
    let mut new_url = use_signal(String::new);

    let list = bookmarks();
    let current = list
        .get(selected())
        .cloned()
        .unwrap_or(Bookmark { placeholder: String::new(), url: String::new() });

    rsx! {
        div {
            class: "flex flex-col @md:flex-row min-h-0 h-full w-full overflow-auto",

            // sidebar: tabs + input
            section {
                class: SIDEBAR_CLASS,

                // list tabs
                div {
                    class: TAB_LIST_CLASS,
                    for (idx, bm) in list.iter().enumerate() {
                        button {
                            key: "{bm.placeholder}-{idx}",
                            r#type: "button",
                            class: tab_class(selected() == idx),
                            onclick: move |_| selected.set(idx),
                            "{bm.placeholder}"
                        }
                    }
                }

                div {
                    class: INPUT_SECTION_CLASS,
                    input {
                        r#type: "text",
                        placeholder: "placeholder...",
                        class: INPUT_LABEL_CLASS,
                        value: "{new_label}",
                        oninput: move |evt: FormEvent| new_label.set(evt.value()),
                    }
                    div {
                        class: "flex items-center gap-2",
                        input {
                            r#type: "text",
                            placeholder: "https://...",
                            class: INPUT_URL_CLASS,
                            value: "{new_url}",
                            onkeydown: move |evt: KeyboardEvent| match evt.key() {
                                Key::Enter => {
                                    evt.prevent_default();
                                    add_bookmark(bookmarks, selected, new_label, new_url);
                                }
                                _ => {}
                            },
                            oninput: move |evt: FormEvent| new_url.set(evt.value()),
                        }
                        button {
                            r#type: "button",
                            class: ADD_BUTTON_CLASS,
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
                    class: URL_BAR_CLASS,
                    img {
                        src: ICON_LINK,
                        alt: "link",
                        class: "h-4",
                    }
                    "{current.url}"
                }

                div {
                    class: IFRAME_WRAPPER_CLASS,
                    iframe {
                        src: "{current.url}",
                        class: "w-full h-full border-0",
                        title: "{current.placeholder}",
                    }
                }

                div {
                    class: FOOTER_CLASS,
                    a {
                        target: "_blank",
                        rel: "noopener noreferrer",
                        href: "{current.url}",
                        class: LINK_CLASS,
                        "Go to Website ↗"
                    }
                }
            }
        }
    }
}
