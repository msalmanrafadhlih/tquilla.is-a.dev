use dioxus::prelude::*;

const NIXOS_ICON: Asset = asset!("/assets/Icon-Nixos.svg");

const SYSTEM_INFO: [(&str, &str); 10] = [
    ("Version", "NixOS 26.11 (Zakar)"),
    ("Kernel", "Linux 6.18.38"),
    ("Architecture", "x86_64"),
    ("Desktop", "DeistifyNix (Wayland)"),
    ("Memory", "1.70 GB / 8.00 GB"),
    ("Philosophy", "Declaratif Desktop OS (Joke)"),
    ("Design", "Black & White"),
    ("Built in", "Dioxus (Rust) + Tailwind"),
    ("Network", "100 People Connected"),
    ("Files shared", "10 Open files"),
];

#[component]
fn InfoBlock() -> Element {
    rsx! {
        div { class: "flex flex-col items-center gap-4 px-6 py-6 text-center",
            img { src: NIXOS_ICON, alt: "NixOS", class: "w-14 h-14" }
            div {
                p { class: "font-semibold", "NixOS 26.11" }
                a {
                    href: "https://nixos.org",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "text-[11px] text-white/40 hover:text-white transition-colors duration-150",
                    "https://nixos.org",
                }
            }
            div { class: "w-full text-left text-[11px] leading-relaxed",
                for (label , value) in SYSTEM_INFO {
                    div {
                        key: "{label}",
                        class: "flex justify-between gap-4 py-0.5 border-b border-white/5",
                        span { class: "text-white/40", "{label}:" }
                        span { class: "text-white/80 text-right", "{value}" }
                    }
                }
            }
        }
    }
}

/// "About" — a standalone info card.
#[component]
pub fn AboutWindowContent() -> Element {
    rsx! {
        div { class: "h-full overflow-y-auto", InfoBlock {} }
    }
}

/// "Settings" — the same info, behind a sidebar (just an "About" section
/// for now; more sections can slot into `SECTIONS` later).
const SECTIONS: [&str; 1] = ["About"];

#[component]
pub fn SettingsWindowContent() -> Element {
    let mut section = use_signal(|| "About");

    rsx! {
        div { class: "flex h-full text-[11px]",
            div { class: "w-28 shrink-0 border-r border-white/15 py-2",
                for s in SECTIONS {
                    button {
                        key: "{s}",
                        r#type: "button",
                        class: "w-full text-left px-3 py-1.5",
                        class: if section() == s { "bg-white text-black" } else { "text-white/60 hover:text-white" },
                        onclick: move |_| section.set(s),
                        "{s}"
                    }
                }
            }
            div { class: "flex-1 min-w-0 overflow-y-auto", InfoBlock {} }
        }
    }
}
