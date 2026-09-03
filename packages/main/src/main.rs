mod journal;
mod desktop;

use journal::JournalPage;
use desktop::DesktopPage;

use dioxus::prelude::*;
use serde::Deserialize;
use gloo_timers::future::TimeoutFuture;

const GENERATIONS_JSON: &str = include_str!("../data/generations.json");

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct Generation {
    number: u32,
    label: String,
    link: String,
    kernel: String,
    date: String,
}

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/profile")]
    JournalPage {},
    #[route("/deisktify")]
    DesktopPage {},
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
    rsx! {
        Router::<Route> {}
    }
}

#[component]
pub fn Home() -> Element {
    let generations: Vec<Generation> = serde_json::from_str(GENERATIONS_JSON).unwrap_or_default();
    let total = generations.len();
    let mut number: f32 = 1.01;
    // Links pulled out separately so the keyboard handler can grab the
    // currently selected URL without borrowing `generations` across the
    // 'static closure boundary.
    let links: Vec<String> = generations.iter().map(|g| g.link.clone()).collect();
    let mut selected = use_signal(|| 0usize);
    // Tracks which row the mouse is currently over, independent of
    // keyboard `selected`, so the label swap only reacts to hover.
    let mut hovered = use_signal(|| Option::<usize>::None);

    let mut counting = use_signal(|| 10i32);
    let last_link = links.first().cloned();
    // Jalan sekali saat mount. Setiap 1 detik kurangi counting; begas 0,
    // buka link generasi terakhir dan hentikan loop-nya.
    use_effect(move || {
        let last_link = last_link.clone();
        spawn(async move {
            loop {
                TimeoutFuture::new(1000).await;
                let remaining = counting() - 1;
                counting.set(remaining);
                if remaining <= 0 {
                    if let Some(url) = &last_link {
                        open_link(url);
                    }
                    break;
                }
            }
        });
    });
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
        document::Title { "Generation Menu" }

        section {
            tabindex: "{total}",
            class: "min-h-screen w-full bg-black text-neutral-200 font-mono flex flex-col items-center justify-center px-3 py-10 outline-none select-none",
            onkeydown,
            onmounted: move |evt| {
                let data = evt.data();
                spawn(async move {
                    let _ = data.set_focus(true).await;
                });
            },

            div {
                class: "w-full max-w-3xl",
                ul {
                    class: "flex flex-col",
                    for (idx, gen) in generations.iter().enumerate() {
                        li {
                            key: "{gen.number}",
                            onmouseenter: move |_| {
                                selected.set(idx);
                                hovered.set(Some(idx));
                            },
                            onmouseleave: move |_| {
                                if hovered() == Some(idx) {
                                    hovered.set(None);
                                }
                            },
                            onclick: {
                                let url = gen.link.clone();
                                move |_| open_link(&url)
                            },
                            class: if selected() == idx {
                                "bg-neutral-200 text-black px-3 py-1.5 cursor-pointer text-[11px] xs:text-xs sm:text-sm break-words transition-colors duration-75 text-center"
                            } else {
                                "bg-black text-neutral-200 px-3 py-1.5 cursor-pointer text-[11px] xs:text-xs sm:text-sm break-words transition-colors duration-75 hover:bg-neutral-800 text-center"
                            },
                            if hovered() == Some(idx) {
                                "{gen.label}"
                            } else {
                                "NixOS (Generation {gen.number} 22.11.2979.47c003416{number}, Linux Kernel {gen.kernel}, Built on {gen.date})"
                                { number += 0.01 }
                            }
                        }
                    }
                }
                p {
                    class: "text-center text-[11px] sm:text-sm py-2 border-t border-neutral-700 text-neutral-500",
                    "Reboot Into Firmware Interface"
                }
            }
            p {
                class: "mt-6 text-center text-[10px] sm:text-xs text-neutral-600 max-w-md",
                "Boot in {counting}s"
            }
            p {
                class: "mt-6 text-center text-[10px] sm:text-xs text-neutral-600 max-w-md",
                "Navigasi pakai ↑ / ↓ atau j / k · Enter / klik untuk buka link"
            }
        }
    }
}
