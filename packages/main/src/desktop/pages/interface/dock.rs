use dioxus::prelude::*;

use super::window::{open_or_focus, AppId, OpenWindow};

const GITHUB_ICON: Asset = asset!("/assets/Github.svg");
const DISCORD_ICON: Asset = asset!("/assets/Discord.svg");
const LINKEDIN_ICON: Asset = asset!("/assets/LinkedIn.svg");
const SETTINGS_ICON: Asset = asset!("/assets/button-Settings.svg");
const FILE_MANAGER_ICON: Asset = asset!("/assets/button-FileManager.svg");
const DVD_ICON: Asset = asset!("/assets/button-DVD.svg");
const EMBIENCE_ICON: Asset = asset!("/assets/button-Embience.svg");
const BROWSER_ICON: Asset = asset!("/assets/button-Browser.svg");
const AI_ICON: Asset = asset!("/assets/button-Ai-Assistent.svg");
const CALCULATOR_ICON: Asset = asset!("/assets/button-calculator.svg");
const NIXOS_ICON: Asset = asset!("/assets/Nixos_Logo.svg");
const LIVE_CHAT_ICON: Asset = asset!("/assets/button-live chat.svg");

// TODO(moch): same placeholder gap as the Terminal nav rail — swap in the
// real LinkedIn / Discord URLs whenever you're ready, Github is confirmed.
const GITHUB_URL: &str = "https://github.com/msalmanrafadhlih";
const LINKEDIN_URL: &str = "https://www.linkedin.com/feed";
const DISCORD_URL: &str = "https://discord.com/invite/motionime";

#[component]
pub fn Dock(open_windows: Signal<Vec<OpenWindow>>, next_z: Signal<i32>) -> Element {
    let apps: [(Asset, AppId); 7] = [
        (SETTINGS_ICON, AppId::Settings),
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
        div { class: "flex items-center justify-between h-12 px-3 border-t border-white/15 shrink-0",
            div { class: "flex items-center gap-3",
                for (icon , label , href) in socials {
                    a {
                        key: "{label}",
                        href,
                        target: "_blank",
                        rel: "noopener noreferrer",
                        title: "{label}",
                        class: "w-7 h-7 flex items-center justify-center opacity-70 hover:opacity-100 transition-opacity duration-150",
                        img { src: icon, alt: "{label}", class: "w-4 h-4" }
                    }
                }
            }
            div { class: "flex items-center gap-2",
                for (icon , app_id) in apps {
                    button {
                        key: "{app_id.key()}",
                        r#type: "button",
                        title: "{app_id.title()}",
                        class: "relative w-8 h-8 flex items-center justify-center border border-transparent hover:border-white/20 transition-colors duration-150",
                        onclick: move |_| open_or_focus(open_windows, next_z, app_id),
                        img { src: icon, alt: "{app_id.title()}", class: "w-4 h-4" }
                        if is_open(app_id) {
                            span { class: "absolute -bottom-1 left-1/2 -translate-x-1/2 w-1 h-1 rounded-full bg-white" }
                        }
                    }
                }
                img { src: NIXOS_ICON, alt: "NixOS", class: "w-5 h-5 ml-2 opacity-80" }
            }
            button {
                r#type: "button",
                class: "flex items-center gap-1.5 border border-white/15 px-2.5 py-1.5 hover:bg-white hover:text-black transition-colors duration-150",
                onclick: move |_| open_or_focus(open_windows, next_z, AppId::LiveChat),
                img { src: LIVE_CHAT_ICON, alt: "", class: "w-3.5 h-3.5" }
                span { class: "text-[11px]", "Live Chat!" }
            }
        }
    }
}
