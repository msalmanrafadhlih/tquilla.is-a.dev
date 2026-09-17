use dioxus::prelude::*;
use serde::Deserialize;

const BOOKMARKS_JSON: &str = include_str!("../../../../data/browser.json");

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
        div { class: "flex h-full text-[11px]",
            div { class: "w-40 shrink-0 border-r border-white/15 flex flex-col",
                div { class: "flex-1 min-h-0 overflow-y-auto py-2",
                    for (idx , bm) in list.iter().enumerate() {
                        {
                            let is_selected = selected() == idx;
                            let label = bm.placeholder.clone();
                            rsx! {
                                button {
                                    key: "{label}-{idx}",
                                    r#type: "button",
                                    class: "w-full text-left px-3 py-1.5 truncate",
                                    class: if is_selected { "bg-white text-black" } else { "text-white/70 hover:text-white" },
                                    onclick: move |_| selected.set(idx),
                                    "{label}"
                                }
                            }
                        }
                    }
                }
                div { class: "shrink-0 border-t border-white/15 p-2 flex flex-col gap-1",
                    input {
                        r#type: "text",
                        placeholder: "placeholder...",
                        class: "bg-transparent outline-none border-b border-white/10 text-white placeholder-white/30 text-[10px] pb-1",
                        value: "{new_label}",
                        oninput: move |evt: FormEvent| new_label.set(evt.value()),
                    }
                    div { class: "flex items-center gap-1",
                        input {
                            r#type: "text",
                            placeholder: "https://...",
                            class: "flex-1 min-w-0 bg-transparent outline-none border-b border-white/10 text-white placeholder-white/30 text-[10px] pb-1",
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
                            class: "text-white/50 hover:text-white shrink-0",
                            onclick: move |_| add_bookmark(bookmarks, selected, new_label, new_url),
                            "+"
                        }
                    }
                }
            }
            div { class: "flex-1 min-w-0 flex flex-col",
                div { class: "shrink-0 border-b border-white/15 px-3 py-1.5 truncate text-white/50", "{current.url}" }
                div { class: "flex-1 min-h-0 bg-white/[0.02]",
                    iframe { src: "{current.url}", class: "w-full h-full border-0", title: "{current.placeholder}" }
                }
                div { class: "shrink-0 border-t border-white/15 px-3 py-2 flex justify-end",
                    a {
                        href: "{current.url}",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "text-white/50 hover:text-white transition-colors duration-150",
                        "Go to Website ↗"
                    }
                }
            }
        }
    }
}
