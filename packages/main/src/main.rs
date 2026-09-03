use dioxus::prelude::*;
use serde::Deserialize;

// The generation list lives in its own JSON file so it can be edited without
// touching any Rust code. It's embedded at compile time (no network fetch,
// no loading state, works offline once built).
const GENERATIONS_JSON: &str = include_str!("../data/generations.json");

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct Generation {
    number: u32,
    label: String,
    link: String,
    kernel: String,
    date: String,
}

/// Open a link the same way a browser "open in new tab" would, so visitors
/// never navigate away from the boot-menu page itself.
fn open_link(url: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.open_with_url_and_target(url, "_self");
    }
}

fn main() {
    dioxus::launch(App);
}

#[component]
pub fn App() -> Element {
    let generations: Vec<Generation> = serde_json::from_str(GENERATIONS_JSON).unwrap_or_default();
    let total = generations.len();

    // Links pulled out separately so the keyboard handler can grab the
    // currently selected URL without borrowing `generations` across the
    // 'static closure boundary.
    let links: Vec<String> = generations.iter().map(|g| g.link.clone()).collect();

    let mut selected = use_signal(|| 0usize);

    let onkeydown = move |evt: KeyboardEvent| match evt.key() {
        Key::ArrowDown => {
            evt.prevent_default();
            if total > 0 {
                selected.set((selected() + 1) % total);
            }
        }
        Key::ArrowUp => {
            evt.prevent_default();
            if total > 0 {
                selected.set((selected() + total - 1) % total);
            }
        }
        Key::Character(c) if c.eq_ignore_ascii_case("j") => {
            if total > 0 {
                selected.set((selected() + 1) % total);
            }
        }
        Key::Character(c) if c.eq_ignore_ascii_case("k") => {
            if total > 0 {
                selected.set((selected() + total - 1) % total);
            }
        }
        Key::Enter => {
            if let Some(url) = links.get(selected()) {
                open_link(url);
            }
        }
        Key::Character(c) if c == " " => {
            if let Some(url) = links.get(selected()) {
                open_link(url);
            }
        }
        _ => {}
    };

    rsx! {
        div {
            tabindex: "0",
            class: "min-h-screen w-full bg-black text-neutral-200 font-mono flex flex-col items-center justify-center px-3 py-10 outline-none select-none",
            onkeydown,
            onmounted: move |evt| {
                let data = evt.data();
                spawn(async move {
                    let _ = data.set_focus(true).await;
                });
            },

            div {
                class: "w-full max-w-3xl border border-neutral-700",
                ul {
                    class: "flex flex-col",
                    for (idx , gen) in generations.iter().enumerate() {
                        li {
                            key: "{gen.number}",
                            onmouseenter: move |_| selected.set(idx),
                            onclick: {
                                let url = gen.link.clone();
                                move |_| open_link(&url)
                            },
                            class: if selected() == idx {
                                "bg-neutral-200 text-black px-3 py-1.5 cursor-pointer text-[11px] xs:text-xs sm:text-sm break-words transition-colors duration-75"
                            } else {
                                "bg-black text-neutral-200 px-3 py-1.5 cursor-pointer text-[11px] xs:text-xs sm:text-sm break-words transition-colors duration-75 hover:bg-neutral-800"
                            },
                            "NixOS (Generation {gen.number} {gen.label}, Linux Kernel {gen.kernel}, Built on {gen.date})"
                        }
                    }
                }
                div {
                    class: "text-center text-[11px] sm:text-sm py-2 border-t border-neutral-700 text-neutral-500",
                    "Reboot Into Firmware Interface"
                }
            }

            p {
                class: "mt-6 text-center text-[10px] sm:text-xs text-neutral-600 max-w-md",
                "Navigasi pakai ↑ / ↓ atau j / k · Enter / klik untuk buka link"
            }
        }
    }
}
