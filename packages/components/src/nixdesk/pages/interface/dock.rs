use dioxus::prelude::*;

use super::window::{toggle_window, AppId, OpenWindow};
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
const LIVE_CHAT_ICON: Asset = asset!("/assets/logo-livechat.svg");

#[allow(dead_code)]
const LABEL_CLASS: &str = "relative flex items-center justify-center group-hover:block md:hidden w-auto max-w-max whitespace-nowrap mt-[-0.50px] font-['JetBrains_Mono'] font-normal text-white text-sm text-center tracking-[0] leading-[normal] group-hover:text-black";
const IMG_CLASS: &str = "relative w-5 h-5 aspect-[1] group-hover:invert transition-transform duration-300 ease-out group-hover:scale-110";

#[component]
pub fn Dock(open_windows: Signal<Vec<OpenWindow>>, next_z: Signal<i32>) -> Element {
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
        (GITHUB_ICON, "Github", GITHUB_URL),
        (DISCORD_ICON, "Discord", DISCORD_URL),
        (LINKEDIN_ICON, "LinkedIn", LINKEDIN_URL),
    ];

    // Indikator tetap tampil selama window masih ada (terbuka ATAU minimized).
    // Hilang hanya saat benar-benar ditutup.
    let has_window =
        move |app_id: AppId| open_windows().iter().any(|w| w.id == app_id && !w.closing);

    rsx! {
        footer { class: "relative flex w-full flex-none items-center justify-center gap-2.5 px-2.5 pb-2.5 md:pb-0 z-[999] rounded-3xl border-b-[0.5px] border-white backdrop-blur-sm bg-[image:var(--linear-b)]",

            // LEFT DOCK — social links. jika button `Open social menu` di klick, maka akan memunculkan tag list anchor diatasnya
            section {
                class: "flex flex-col md:flex-row w-10 md:w-full md:justify-start md:items-center md:gap-0 h-10 items-start justify-end gap-5 relative",
                "aria-label": "Social links",
                div {
                    class: if social_menu_open() {
                        "grid grid-rows-[1fr] transition-[grid-template-rows] duration-300 ease-out overflow-hidden shrink-0 md:flex md:overflow-visible"
                    } else {
                        "grid grid-rows-[0fr] transition-[grid-template-rows] duration-300 ease-out overflow-hidden shrink-0 md:flex md:overflow-visible"
                    },
                    div { class: "flex flex-col gap-5 w-max md:flex-row md:gap-0 overflow-hidden md:overflow-visible",
                        for (icon , label , href) in socials {
                            a {
                                key: "{label}",
                                href,
                                target: "_blank",
                                rel: "noopener noreferrer",
                                "aria-label": "{label}",
                                class: "group inline-flex w-max items-center gap-2.5 relative flex-[0_0_auto] z-[3] px-2 py-1 rounded-xl hover:bg-white transition-color duration-300 ease-out",
                                img { src: icon, alt: "", class: IMG_CLASS }
                                span { class: "grid grid-cols-[0fr] group-hover:grid-cols-[1fr] transition-[grid-template-columns] duration-300 ease-out overflow-hidden",
                                    span { class: "overflow-hidden whitespace-nowrap mt-[-0.50px] font-['JetBrains_Mono'] font-normal text-white text-sm text-center tracking-[0] leading-[normal] group-hover:text-black",
                                        "{label}"
                                    }
                                }
                            }
                        }
                    }
                }
                button {
                    r#type: "button",
                    "aria-label": "Open social menu",
                    "aria-expanded": if social_menu_open() { "true" } else { "false" },
                    class: "inline-flex items-center md:hidden gap-2.5 relative flex-[0_0_auto] z-0 border-0 p-0 shrink-0",
                    onclick: move |_| social_menu_open.set(!social_menu_open()),
                    img {
                        src: MENU_ICON,
                        alt: "",
                        class: if social_menu_open() {
                            "relative w-10 h-10 aspect-[1] rotate-180 transition-transform duration-300 ease-out"
                        } else {
                            "relative w-10 h-10 aspect-[1] transition-transform duration-300 ease-out"
                        }
                    }
                }
            }

            // MIDDLE DOCK — app launcher. Always a single row; the icon
            // strip scrolls horizontally instead of wrapping.
            nav {
                class: "flex w-full min-w-0 flex-1 items-center justify-center gap-2 sm:gap-5 self-stretch px-2.5 md:pb-2.5 md:w-fit md:flex-none",
                "aria-label": "Application dock",

                button {
                    r#type: "button",
                    "aria-label": "Open settings",
                    class: "relative h-[30px] w-[30px] flex-none border-0 p-0",
                    onclick: move |_| toggle_window(open_windows, next_z, AppId::Settings),
                    img { src: SETTINGS_ICON, alt: "", class: "h-full w-full" }
                }

                div { class: "h-[31.5px] w-px flex-none bg-[#555]" }

                div { class: "scrollbar-hide flex min-w-0 flex-1 items-center gap-5 overflow-x-scroll overflow-y-hidden border-x border-[#555] px-2 md:px-0 py-1 md:w-fit md:flex-none md:border-0",
                    for (icon , app_id) in apps {
                        button {
                            key: "{app_id.key()}",
                            r#type: "button",
                            title: "{app_id.title()}",
                            class: if has_window(app_id) { "flex w-10 flex-none flex-col items-start gap-2.5 border-0 border-b-[0.5px] border-white bg-[var(--linear-b)] px-0 pb-[5px] pt-0" } else { "flex w-10 flex-none flex-col items-start gap-2.5 border-0 px-0 pb-[5px] pt-0 transition-transform duration-150 ease-out hover:scale-125 active:scale-90" },
                            onclick: move |_| toggle_window(open_windows, next_z, app_id),
                            img { src: icon, alt: "{app_id.title()}", class: "h-10 w-10" }
                        }
                    }
                }

                div { class: "h-[31.5px] w-px flex-none bg-[#555]" }

                button {
                    r#type: "button",
                    "aria-label": "About NixDesktop",
                    class: "relative h-[30px] w-[30px] flex-none border-0 p-0",
                    onclick: move |_| toggle_window(open_windows, next_z, AppId::About),
                    img { src: NIXOS_ICON, alt: "", class: "h-full w-full" }
                }
            }

            // RIGHT DOCK — live chat. jika button `Open chat menu` di klick, maka akan memunculkan tag list button atasnya
            section {
                class: "flex flex-col md:flex-row md:gap-1 w-10 md:w-full h-10 items-end justify-end md:items-center gap-5 relative md:aspect-[1]",
                "aria-label": "Chat",
                div {
                    class: if chat_menu_open() {
                        "grid grid-rows-[1fr] transition-[grid-template-rows] duration-300 ease-out shrink-0 md:flex overflow-hidden md:overflow-visible"
                    } else {
                        "grid grid-rows-[0fr] transition-[grid-template-rows] duration-300 ease-out shrink-0 md:flex overflow-hidden md:overflow-visible"
                    },
                    div { class: "w-max overflow-hidden md:overflow-visible",
                        button {
                            r#type: "button",
                            class: "group inline-flex w-max items-center gap-2.5 relative flex-[0_0_auto] z-[3] px-2 py-1 rounded-xl hover:bg-white transition-colors duration-300 ease-out",
                            onclick: move |_| toggle_window(open_windows, next_z, AppId::LiveChat),
                            span { class: "grid grid-cols-[0fr] group-hover:grid-cols-[1fr] transition-[grid-template-columns] duration-300 ease-out overflow-hidden",
                                span { class: "overflow-hidden whitespace-nowrap mt-[-0.50px] font-['JetBrains_Mono'] font-normal text-white text-sm text-center tracking-[0] leading-[normal] group-hover:text-black",
                                    "Live Chat"
                                }
                            }
                            img { src: LIVE_CHAT_ICON, alt: "", class: IMG_CLASS }
                        }
                    }
                }
                button {
                    r#type: "button",
                    "aria-label": "Open chat menu",
                    "aria-expanded": if chat_menu_open() { "true" } else { "false" },
                    class: "inline-flex md:hidden items-center justify-end gap-2.5 relative flex-[0_0_auto] z-0 border-0 p-0 shrink-0",
                    onclick: move |_| chat_menu_open.set(!chat_menu_open()),
                    img {
                        src: MENU_ICON,
                        alt: "",
                        class: if chat_menu_open() {
                            "relative w-10 h-10 aspect-[1] rotate-180 transition-transform duration-300 ease-out"
                        } else {
                            "relative w-10 h-10 aspect-[1] transition-transform duration-300 ease-out"
                        }
                    }
                }
            }
        }
    }
}
