use dioxus::prelude::*;

use super::window::{open_or_focus, AppId, OpenWindow};

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

const GITHUB_URL: &str = "https://github.com/msalmanrafadhlih";
const LINKEDIN_URL: &str = "https://www.linkedin.com/feed";
const DISCORD_URL: &str = "https://discord.com/invite/motionime";

const LABEL_CLASS: &str = "relative flex items-center justify-center group-hover:block md:hidden w-auto max-w-max whitespace-nowrap mt-[-0.50px] font-['JetBrains_Mono'] font-normal text-white text-sm text-center tracking-[0] leading-[normal] group-hover:text-black";

#[component]
pub fn Dock(open_windows: Signal<Vec<OpenWindow>>, next_z: Signal<i32>) -> Element {
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

    let is_open = move |app_id: AppId| open_windows().iter().any(|w| w.id == app_id && !w.minimized);

    rsx! {
        footer { class: "relative flex w-full flex-none items-center justify-center gap-2.5 px-2.5",

            // LEFT DOCK — social links. Fixed-width icon column on
            nav {
                class: "flex flex-col md:flex-row w-10 md:w-full md:justify-start md:items-center md:gap-0 h-10 items-start justify-end gap-5 relative",
                "aria-label": "Social links",
                for (icon , label , href) in socials {
                    a {
                        key: "{label}",
                        href,
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "aria-label": "{label}",
                        class: "group inline-flex w-max items-center gap-2.5 relative flex-[0_0_auto] z-[3] px-2 py-1 hover:bg-white",
                        img { src: icon, alt: "", class: "relative w-5 h-5 aspect-[1] group-hover:invert" }
                        span { class: LABEL_CLASS, "{label}" }
                    }
                }
                button {
                    r#type: "button",
                    "aria-label": "Open social menu",
                    class: "inline-flex items-center md:hidden gap-2.5 relative flex-[0_0_auto] z-0 bg-transparent border-0 p-0",
                    img { src: MENU_ICON, alt: "", class: "relative w-10 h-10 aspect-[1]" }
                }
            }

            // MIDDLE DOCK — app launcher. Always a single row; the icon
            // strip scrolls horizontally instead of wrapping.
            div { class: "relative flex w-full min-w-0 flex-1 items-end justify-center md:w-fit md:flex-none",
                nav {
                    class: "flex w-full min-w-0 flex-1 items-center justify-center gap-5 self-stretch border-b-[0.5px] border-white p-2.5 [background:linear-gradient(180deg,rgba(0,0,0,1)_50%,rgba(37,37,37,1)_100%)] md:w-fit md:flex-none",
                    "aria-label": "Application dock",

                    button {
                        r#type: "button",
                        "aria-label": "Open settings",
                        class: "relative h-[30px] w-[30px] flex-none border-0 bg-transparent p-0",
                        onclick: move |_| open_or_focus(open_windows, next_z, AppId::Settings),
                        img { src: SETTINGS_ICON, alt: "", class: "h-full w-full" }
                    }

                    div { class: "h-[31.5px] w-px flex-none bg-[#555]" }

                    div { class: "scrollbar-hide flex min-w-0 flex-1 items-center gap-5 overflow-x-scroll overflow-y-hidden border-x border-[#555] px-5 py-0 md:w-fit md:flex-none md:border-0",
                        for (icon , app_id) in apps {
                            button {
                                key: "{app_id.key()}",
                                r#type: "button",
                                title: "{app_id.title()}",
                                class: if is_open(app_id) { "flex w-10 flex-none flex-col items-start gap-2.5 border-0 border-b-[0.5px] border-white bg-transparent px-0 pb-[5px] pt-0" } else { "flex w-10 flex-none flex-col items-start gap-2.5 border-0 bg-transparent px-0 pb-[5px] pt-0" },
                                onclick: move |_| open_or_focus(open_windows, next_z, app_id),
                                img { src: icon, alt: "{app_id.title()}", class: "h-10 w-10" }
                            }
                        }
                    }

                    div { class: "h-[31.5px] w-px flex-none bg-[#555]" }

                    button {
                        r#type: "button",
                        "aria-label": "About NixDesktop",
                        class: "relative h-[30px] w-[30px] flex-none border-0 bg-transparent p-0",
                        onclick: move |_| open_or_focus(open_windows, next_z, AppId::About),
                        img { src: NIXOS_ICON, alt: "", class: "h-full w-full" }
                    }
                }
            }

            // RIGHT DOCK — live chat.
            nav {
                class: "flex flex-col md:flex-row md:gap-1 w-10 md:w-full h-10 items-end justify-end md:items-center gap-5 relative aspect-[1]",
                "aria-label": "Chat",
                button {
                    r#type: "button",
                    class: "group inline-flex w-max items-center justify-end gap-2.5 relative flex-[0_0_auto] z-[1] bg-transparent border-0 px-2 py-1 hover:bg-white",
                    onclick: move |_| open_or_focus(open_windows, next_z, AppId::LiveChat),
                    span { class: LABEL_CLASS, "Live Chat" }
                    img { src: LIVE_CHAT_ICON, alt: "", class: "relative w-5 h-5 aspect-[1] group-hover:invert" }
                }
                button {
                    r#type: "button",
                    "aria-label": "Open chat menu",
                    class: "inline-flex md:hidden items-center justify-end gap-2.5 relative flex-[0_0_auto] z-0 bg-transparent border-0 p-0",
                    img { src: MENU_ICON, alt: "", class: "relative w-10 h-10 aspect-[1]" }
                }
            }
        }
    }
}
