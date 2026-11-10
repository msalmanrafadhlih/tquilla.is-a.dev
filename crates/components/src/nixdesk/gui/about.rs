use crate::languages::use_strings;
use dioxus::prelude::*;

const NIXOS_ICON: Asset = asset!("/assets/logo-nixos.svg");

#[component]
pub fn InfoBlock() -> Element {
    let s = use_strings();
    // Blok kedua (`s.about.extra_info()`: Philosophy, Design, ...) belum dipakai
    // di UI; blok pertama dirender dua kali seperti sebelumnya.
    let system_info = s.about.system_info();
    rsx! {
        header {
            class: "about__header",
            div { class: "about__logo-box",
                img { src: NIXOS_ICON, class: "about__logo", width: "75", height: "64", alt: s.about.logo_alt }
            }
            div { class: "about__title-box",
                h2 { class: "about__title",
                    "{s.about.title}"
                }
                a {
                    href: "https://nixos.org",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    class: "about__link",
                    "https://nixos.org",
                }
            }
        }
        div { class: "about__info",
            for (label , value) in system_info {
                div {
                    key: "{label}",
                    class: "about__row",
                    span { class: "about__label", "{label}:" }
                    span { class: "about__value", "{value}" }
                }
            }
        }
        div { class: "about__info",
            for (label , value) in system_info {
                div {
                    key: "{label}",
                    class: "about__row",
                    span { class: "about__label", "{label}:" }
                    span { class: "about__value", "{value}" }
                }
            }
        }
    }
}

/// "About" — a standalone info card.
#[component]
pub fn AboutWindowContent() -> Element {
    rsx! {
        div { class: "about__window",
            section {
                class: "app-window__panel",
                class: "app-window__scroll",

                InfoBlock {}
            }
        }
    }
}
