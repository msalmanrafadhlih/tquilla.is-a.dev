use std::sync::OnceLock;

use dioxus::prelude::*;
use serde::Deserialize;

use crate::app::Route;
use crate::platform::{use_platform, PlatformServices, TimeoutFuture};
use crate::shared::FAVICON;

const GENERATIONS_JSON: &str = include_str!("../data/generations.json");

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct Generation {
    number: u32,
    label: String,
    link: String,
    kernel: String,
    date: String,
}

/// Parsed once, then shared: the `Generation` list and, in the same order,
/// just their links.
fn generation_data() -> (&'static [Generation], &'static [String]) {
    static DATA: OnceLock<(Vec<Generation>, Vec<String>)> = OnceLock::new();
    let (generations, links) = DATA.get_or_init(|| {
        let generations: Vec<Generation> =
            serde_json::from_str(GENERATIONS_JSON).unwrap_or_default();
        let links = generations.iter().map(|g| g.link.clone()).collect();
        (generations, links)
    });
    (generations.as_slice(), links.as_slice())
}

/// Internal routes (`/profile`, `/deisktify`) go through the router on every
/// platform; anything else is handed to the platform package, which knows how
/// to open it (same tab on web, system browser on desktop, ...).
fn open_link(platform: PlatformServices, url: &str) {
    if url.starts_with('/') {
        if let Ok(route) = url.parse::<Route>() {
            navigator().push(route);
            return;
        }
    }
    (platform.open_external)(url);
}

#[component]
pub fn Home() -> Element {
    let platform = use_platform();
    let open = use_callback(move |url: String| open_link(platform, &url));
    // Di-parse sekali per sesi (bukan tiap render: komponen ini re-render
    // setiap detik karena countdown dan setiap hover).
    let (generations, links) = generation_data();
    let total = generations.len();
    let mut number: f32 = 1.01;
    // `links` dipisah supaya handler keyboard bisa mengambil URL yang sedang
    // dipilih tanpa meminjam `generations` melewati batas closure 'static
    // (keduanya `&'static`, jadi `Copy`).
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
                        open.call(url.clone());
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
                open.call(url.clone());
            }
        }
        Key::Character(c) if c == " " => {
            if let Some(url) = links.get(selected()) {
                open.call(url.clone());
            }
        }
        _ => {}
    };

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Title { "Generation Menu" }

        section {
            tabindex: "{total}",
            class: "min-h-screen w-full bg-[var(--bg-main)] text-neutral-200 font-mono flex flex-col items-center justify-center px-3 py-10 outline-none select-none",
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
                                move |_| open.call(url.clone())
                            },
                            class: if selected() == idx {
                                "bg-neutral-200 text-black px-3 py-1.5 cursor-pointer text-[11px] xs:text-xs sm:text-sm break-words transition-colors duration-75 text-center"
                            } else {
                                "bg-[var(--bg-main)] text-neutral-200 px-3 py-1.5 cursor-pointer text-[11px] xs:text-xs sm:text-sm break-words transition-colors duration-75 hover:bg-neutral-800 text-center"
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
