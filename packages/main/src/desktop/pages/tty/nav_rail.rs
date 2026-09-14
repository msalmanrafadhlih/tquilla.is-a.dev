use dioxus::prelude::*;

const GITHUB_ICON: Asset = asset!("/assets/Github.svg");
const GITHUB_ICON_HOVER: Asset = asset!("/assets/Github_2.svg");
const LINKEDIN_ICON: Asset = asset!("/assets/LinkedIn.svg");
const LINKEDIN_ICON_HOVER: Asset = asset!("/assets/Linkedin_2.svg");
const DISCORD_ICON: Asset = asset!("/assets/Discord.svg");
const DISCORD_ICON_HOVER: Asset = asset!("/assets/Discord_2.svg");

// TODO(moch): swap in the real LinkedIn / Discord profile URLs — the
// Github one is confirmed from data/browser.json, the other two are
// placeholders until you hand them over.
const GITHUB_URL: &str = "https://github.com/msalmanrafadhlih";
const LINKEDIN_URL: &str = "https://www.linkedin.com/feed";
const DISCORD_URL: &str = "https://discord.com/invite/motionime";

/// Mode-switch button + Discord/LinkedIn/Github links. Renders as a
/// vertical rail on desktop (right edge, full height) or a horizontal bar
/// on mobile (bottom row) depending on `vertical`.
#[component]
pub fn NavRail(on_toggle: EventHandler<()>, vertical: bool) -> Element {
    let links: [(&str, &str, Asset, Asset); 3] = [
        ("Discord", DISCORD_URL, DISCORD_ICON, DISCORD_ICON_HOVER),
        ("LinkedIn", LINKEDIN_URL, LINKEDIN_ICON, LINKEDIN_ICON_HOVER),
        ("Github", GITHUB_URL, GITHUB_ICON, GITHUB_ICON_HOVER),
    ];

    let wrapper_class = if vertical {
        "flex flex-col pl-2 items-center justify-between h-full py-0 w-max border-l border-white/10 shrink-0 gap-2"
    } else {
        "flex items-center justify-between w-full px-4 py-2.5 shrink-0 gap-2"
    };

    rsx! {
        div { class: wrapper_class,
            button {
                r#type: "button",
                class: "text-[9px] leading-tight tracking-wide text-white/60 hover:text-black hover:bg-white transition-colors duration-150 sm:h-[50px] px-2 py-2 text-center cursor-pointer border border-white/15",
                onclick: move |_| on_toggle.call(()),
                "Switch"
                br {}
                "Desktop"
            }
            div { class: "h-full w-full w-full border border-white/15" }

            for (label , href , icon , icon_hover) in links {
                a {
                    key: "{label}",
                    href,
                    target: "_blank",
                    rel: "noopener noreferrer",
                    title: "{label}",
                    class: "group relative w-[50px] sm:h-[50px] aspect-square flex items-center justify-center opacity-70 hover:opacity-100 transition-opacity duration-150 border border-white/20",
                    img {
                        src: icon,
                        alt: "{label}",
                        class: "w-5 h-5 absolute transition-opacity duration-150 group-hover:opacity-0",
                    }
                    img {
                        src: icon_hover,
                        alt: "{label}",
                        class: "w-5 h-5 absolute opacity-0 transition-opacity duration-150 group-hover:opacity-100",
                    }
                }
            }
        }
    }
}
