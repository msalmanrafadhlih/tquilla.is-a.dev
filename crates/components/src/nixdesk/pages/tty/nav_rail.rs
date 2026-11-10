use dioxus::prelude::*;

use crate::languages::use_strings;
use crate::shared::{DISCORD_URL, GITHUB_URL, LINKEDIN_URL};

const GITHUB_ICON: Asset = asset!("/assets/icon-github-2.svg");
const GITHUB_ICON_HOVER: Asset = asset!("/assets/icon-github.svg");
const LINKEDIN_ICON: Asset = asset!("/assets/icon-linkedin-2.svg");
const LINKEDIN_ICON_HOVER: Asset = asset!("/assets/icon-linkedin.svg");
const DISCORD_ICON: Asset = asset!("/assets/icon-discord-2.svg");
const DISCORD_ICON_HOVER: Asset = asset!("/assets/icon-discord.svg");

/// Mode-switch button + Discord/LinkedIn/Github links. Renders as a
/// vertical rail on desktop (right edge, full height) or a horizontal bar
/// on mobile (bottom row) depending on `vertical`.
#[component]
pub fn NavRail(on_toggle: EventHandler<()>, vertical: bool) -> Element {
    let s = use_strings();
    let links: [(&str, &str, Asset, Asset); 3] = [
        (
            s.common.discord,
            DISCORD_URL,
            DISCORD_ICON,
            DISCORD_ICON_HOVER,
        ),
        (
            s.common.linkedin,
            LINKEDIN_URL,
            LINKEDIN_ICON,
            LINKEDIN_ICON_HOVER,
        ),
        (s.common.github, GITHUB_URL, GITHUB_ICON, GITHUB_ICON_HOVER),
    ];

    let wrapper_class = if vertical {
        "terminal__rail terminal__rail--vertical"
    } else {
        "terminal__rail terminal__rail--horizontal"
    };

    rsx! {
        div { class: wrapper_class,
            button {
                r#type: "button",
                class: "terminal__rail-btn",
                onclick: move |_| on_toggle.call(()),
                "{s.terminal.switch}"
                br {}
                "{s.terminal.desktop}"
            }
            div { class: "terminal__rail-frame" }

            for (label , href , icon , icon_hover) in links {
                a {
                    key: "{label}",
                    href,
                    target: "_blank",
                    rel: "noopener noreferrer",
                    title: "{label}",
                    class: "group terminal__rail-link",
                    img {
                        src: icon,
                        alt: "{label}",
                        class: "terminal__rail-icon terminal__rail-icon--base",
                    }
                    img {
                        src: icon_hover,
                        alt: "{label}",
                        class: "terminal__rail-icon terminal__rail-icon--hover",
                    }
                }
            }
        }
    }
}
