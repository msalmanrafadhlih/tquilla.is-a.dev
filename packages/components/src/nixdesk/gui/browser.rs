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
    let label = new_label().trim().to_string();
    let url = new_url().trim().to_string();
    if label.is_empty() || url.is_empty() { return; }
    let url = if url.starts_with("http://") || url.starts_with("https://") { url } else { format!("https://{url}") };

    let mut list = bookmarks();
    list.push(Bookmark { placeholder: label, url });
    let idx = list.len() - 1;
    bookmarks.set(list);
    selected.set(idx);
    new_label.set(String::new());
    new_url.set(String::new());
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

#[component]
pub fn BrowserWindowContent() -> Element {
    let bookmarks = use_signal(load_bookmarks);
    let mut selected = use_signal(|| 0usize);
    let mut new_label = use_signal(String::new);
    let mut new_url = use_signal(String::new);
    let mut form_open = use_signal(|| false);

    let list = bookmarks();
    let current = list
        .get(selected())
        .cloned()
        .unwrap_or(Bookmark { placeholder: String::new(), url: String::new() });

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
                            key: "{bm.placeholder}-{idx}",
                            r#type: "button",
                            class: tab_class(selected() == idx),
                            onclick: move |_| selected.set(idx),
                            "{bm.placeholder}"
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
                            r#type: "text",
                            placeholder: "placeholder...",
                            class: "w-full py-2 bg-[var(--bg-secondary)] outline-none border-0 border-b border-[var(--fg-secondary)] text-[var(--fg-main)] placeholder-[var(--fg-secondary)] text-xs",
                            value: "{new_label}",
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
                            r#type: "text",
                            placeholder: "https://...",
                            class: "w-full h-full py-2 flex-1 min-w-0 bg-[var(--bg-secondary)] outline-none border-0 border-b border-[var(--fg-secondary)] text-[var(--fg-main)] placeholder-[var(--fg-secondary)] text-xs",
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
                        alt: "link",
                        class: "h-4",
                    }
                    "{current.url}"
                }

                div {
                    class: "flex-1 min-h-0 bg-white/[0.02]",
                    iframe {
                        src: "{current.url}",
                        class: "w-full h-full border-0",
                        title: "{current.placeholder}",
                    }
                }

                div {
                    class: "shrink-0 border-t border-solid border-[var(--fg-secondary)] px-3 pt-2 flex justify-end",
                    a {
                        target: "_blank",
                        rel: "noopener noreferrer",
                        href: "{current.url}",
                        class: "text-[var(--fg-secondary)] hover:text-[var(--fg-main)] transition-colors duration-150 text-xs",
                        "Go to Website ↗"
                    }
                }
            }
        }
    }
}


