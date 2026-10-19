use dioxus::prelude::*;

use super::about::InfoBlock;
use crate::languages::{use_strings, Strings};

/// "Settings" — the same info, behind a sidebar. More sections can slot
/// into `SettingsSection::ALL` later (the language picker goes under
/// Preferences: write to `use_language()`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SettingsSection {
    About,
    Preferences,
}

impl SettingsSection {
    const ALL: [SettingsSection; 2] = [SettingsSection::About, SettingsSection::Preferences];

    fn label(self, s: &Strings) -> &'static str {
        match self {
            SettingsSection::About => s.settings.section_about,
            SettingsSection::Preferences => s.settings.section_preferences,
        }
    }

    /// Stable key (not translated): used for rsx keys.
    fn key(self) -> &'static str {
        match self {
            SettingsSection::About => "about",
            SettingsSection::Preferences => "preferences",
        }
    }
}

#[component]
pub fn SettingsWindowContent() -> Element {
    let s = use_strings();
    let mut section = use_signal(|| SettingsSection::About);

    rsx! {
        div { class: "flex items-start w-full h-full justify-center gap-5 relative",
            nav { class: "inline-flex flex-col h-full min-w-[100px] items-center relative self-stretch flex-[0_0_auto] border-r border-solid border-[var(--fg-secondary)]",
                for item in SettingsSection::ALL {
                    button {
                        key: "{item.key()}",
                        r#type: "button",
                        class: "flex items-center p-2 relative self-stretch w-full flex-[0_0_auto] border-0 text-xs",
                        class: if section() == item { "bg-[var(--fg-main)]  text-[var(--bg-secondary)] " } else { "text-[var(--fg-secondary)] hover:text-[var(--fg-main)]" },
                        onclick: move |_| section.set(item),
                        "{item.label(s)}"
                    }
                }
            }
            section { class: "max-w-[500px] flex w-full h-full flex-col items-center justify-start gap-[25px] pb-5 relative flex-1 grow self-stretch",
                class: "overflow-auto [-webkit-touch-callout:none] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden",
                match section() {
                    SettingsSection::About => rsx! { InfoBlock {} },
                    SettingsSection::Preferences => rsx! { p { class: "text-white/40 text-[11px]", "{s.settings.coming_soon}" } },
                }
            }
        }
    }
}
