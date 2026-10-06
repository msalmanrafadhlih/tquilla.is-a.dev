use dioxus::prelude::*;

use super::about::InfoBlock;

/// "Settings" — the same info, behind a sidebar. More sections can slot
/// into `SECTIONS` later.
const SECTIONS: [&str; 2] = ["About", "Preferences"];

#[component]
pub fn SettingsWindowContent() -> Element {
    let mut section = use_signal(|| SECTIONS[0]);

    rsx! {
        div { class: "flex items-start w-full h-full justify-center gap-5 relative",
            nav { class: "inline-flex flex-col h-full min-w-[100px] items-center relative self-stretch flex-[0_0_auto] border-r border-solid border-[var(--fg-secondary)]",
                for s in SECTIONS {
                    button {
                        key: "{s}",
                        r#type: "button",
                        class: "flex items-center p-2 relative self-stretch w-full flex-[0_0_auto] border-0 text-xs",
                        class: if section() == s { "bg-[var(--fg-main)]  text-[var(--bg-main)] " } else { "text-[var(--fg-secondary)] hover:text-[var(--fg-main)]" },
                        onclick: move |_| section.set(s),
                        "{s}"
                    }
                }
            }
            section { class: "max-w-[500px] flex w-full h-full flex-col items-center justify-start gap-[25px] pb-5 relative flex-1 grow self-stretch",
                class: "overflow-auto [-webkit-touch-callout:none] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden",
                match section() {
                    "About" => rsx! { InfoBlock {} },
                    _ => rsx! { p { class: "text-white/40 text-[11px]", "Coming soon." } },
                }
            }
        }
    }
}
