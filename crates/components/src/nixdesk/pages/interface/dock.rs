use dioxus::prelude::*;

use super::window::{toggle_window, AppId, OpenWindow};
use crate::languages::use_strings;
use crate::shared::{DISCORD_URL, GITHUB_URL, LINKEDIN_URL};

const GITHUB_ICON: Asset = asset!("/assets/icon-github.svg");
const DISCORD_ICON: Asset = asset!("/assets/icon-discord.svg");
const LINKEDIN_ICON: Asset = asset!("/assets/icon-linkedin.svg");
const MENU_ICON: Asset = asset!("/assets/logo-menu.svg");
const SETTINGS_ICON: Asset = asset!("/assets/logo-Settings.svg");
const FILE_MANAGER_ICON: Asset = asset!("/assets/logo-FileManager.svg");
const DVD_ICON: Asset = asset!("/assets/logo-radio.svg");
const EMBIENCE_ICON: Asset = asset!("/assets/logo-Embience.svg");
const BROWSER_ICON: Asset = asset!("/assets/logo-Browser.svg");
const AI_ICON: Asset = asset!("/assets/logo-Ai-Assistent.svg");
const CALCULATOR_ICON: Asset = asset!("/assets/logo-calculator.svg");
const NIXOS_ICON: Asset = asset!("/assets/Icon-nixos.svg");
const GUESTBOOK_ICON: Asset = asset!("/assets/logo-guestbook.svg");

#[allow(dead_code)]
const LABEL_CLASS: &str = "dock__label";
const IMG_CLASS: &str = "dock__icon";

#[component]
pub fn Dock(open_windows: Signal<Vec<OpenWindow>>, next_z: Signal<i32>) -> Element {
    let s = use_strings();
    let mut social_menu_open = use_signal(|| false);
    let mut chat_menu_open = use_signal(|| false);

    let apps: [(Asset, AppId); 6] = [
        (FILE_MANAGER_ICON, AppId::FileManager),
        (DVD_ICON, AppId::Radio),
        (EMBIENCE_ICON, AppId::Embience),
        (BROWSER_ICON, AppId::Browser),
        (AI_ICON, AppId::AiAssistant),
        (CALCULATOR_ICON, AppId::Calculator),
    ];
    let socials: [(Asset, &str, &str); 3] = [
        (GITHUB_ICON, s.common.github, GITHUB_URL),
        (DISCORD_ICON, s.common.discord, DISCORD_URL),
        (LINKEDIN_ICON, s.common.linkedin, LINKEDIN_URL),
    ];

    // Indikator tetap tampil selama window masih ada (terbuka ATAU minimized).
    // Hilang hanya saat benar-benar ditutup.
    let has_window =
        move |app_id: AppId| open_windows().iter().any(|w| w.id == app_id && !w.closing);

    rsx! {
        footer { class: "dock",

            // LEFT DOCK — social links. jika button `Open social menu` di klick, maka akan memunculkan tag list anchor diatasnya
            section {
                class: "dock__group dock__group--social",
                "aria-label": s.dock.social_links,
                div {
                    class: if social_menu_open() {
                        "dock__fold dock__fold--open"
                    } else {
                        "dock__fold dock__fold--closed"
                    },
                    div { class: "dock__links",
                        for (icon , label , href) in socials {
                            a {
                                key: "{label}",
                                href,
                                target: "_blank",
                                rel: "noopener noreferrer",
                                "aria-label": "{label}",
                                class: "group dock__chip",
                                img { src: icon, alt: "", class: IMG_CLASS }
                                span { class: "dock__reveal",
                                    span { class: "dock__reveal-text",
                                        "{label}"
                                    }
                                }
                            }
                        }
                    }
                }
                button {
                    r#type: "button",
                    "aria-label": s.dock.open_social_menu,
                    "aria-expanded": if social_menu_open() { "true" } else { "false" },
                    class: "dock__toggle",
                    onclick: move |_| social_menu_open.set(!social_menu_open()),
                    img {
                        src: MENU_ICON,
                        alt: "",
                        class: if social_menu_open() {
                            "dock__chevron dock__chevron--up"
                        } else {
                            "dock__chevron"
                        }
                    }
                }
            }

            // MIDDLE DOCK — app launcher. Always a single row; the icon
            // strip scrolls horizontally instead of wrapping.
            nav {
                class: "dock__apps",
                "aria-label": s.dock.app_dock,

                button {
                    r#type: "button",
                    "aria-label": s.dock.open_settings,
                    class: "dock__avatar-btn",
                    onclick: move |_| toggle_window(open_windows, next_z, AppId::Settings),
                    img { src: SETTINGS_ICON, alt: "", class: "dock__avatar" }
                }

                div { class: "dock__divider" }

                div { class: "dock__scroller",
                    for (icon , app_id) in apps {
                        button {
                            key: "{app_id.key()}",
                            r#type: "button",
                            title: "{app_id.title(s)}",
                            class: if has_window(app_id) { "dock__app dock__app--open" } else { "dock__app dock__app--idle" },
                            onclick: move |_| toggle_window(open_windows, next_z, app_id),
                            img { src: icon, alt: "{app_id.title(s)}", class: "dock__app-icon" }
                        }
                    }
                }

                div { class: "dock__divider" }

                button {
                    r#type: "button",
                    "aria-label": s.dock.about_nixdesktop,
                    class: "dock__avatar-btn",
                    onclick: move |_| toggle_window(open_windows, next_z, AppId::About),
                    img { src: NIXOS_ICON, alt: "", class: "dock__avatar" }
                }
            }

            // RIGHT DOCK — guestbook. jika button `Open guestbook menu` di klick, maka akan memunculkan tag list button atasnya
            section {
                class: "dock__group dock__group--chat",
                "aria-label": s.dock.guestbook_group,
                div {
                    class: if chat_menu_open() {
                        "dock__fold dock__fold--open"
                    } else {
                        "dock__fold dock__fold--closed"
                    },
                    div { class: "dock__chat-list",
                        button {
                            r#type: "button",
                            class: "group dock__chip dock__chip--action",
                            onclick: move |_| toggle_window(open_windows, next_z, AppId::Guestbook),
                            span { class: "dock__reveal",
                                span { class: "dock__reveal-text",
                                    "{s.dock.guestbook}"
                                }
                            }
                            img { src: GUESTBOOK_ICON, alt: "", class: IMG_CLASS }
                        }
                    }
                }
                button {
                    r#type: "button",
                    "aria-label": s.dock.open_guestbook_menu,
                    "aria-expanded": if chat_menu_open() { "true" } else { "false" },
                    class: "dock__toggle dock__toggle--end",
                    onclick: move |_| chat_menu_open.set(!chat_menu_open()),
                    img {
                        src: MENU_ICON,
                        alt: "",
                        class: if chat_menu_open() {
                            "dock__chevron dock__chevron--up"
                        } else {
                            "dock__chevron"
                        }
                    }
                }
            }
        }
    }
}
