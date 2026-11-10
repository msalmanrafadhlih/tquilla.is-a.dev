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
        div { class: "settings__layout",
            nav { class: "settings__nav",
                for item in SettingsSection::ALL {
                    button {
                        key: "{item.key()}",
                        r#type: "button",
                        class: "settings__nav-item",
                        class: if section() == item { "settings__nav-item--on " } else { "settings__nav-item--off" },
                        onclick: move |_| section.set(item),
                        "{item.label(s)}"
                    }
                }
            }
            section { class: "app-window__panel",
                class: "app-window__scroll",
                match section() {
                    SettingsSection::About => rsx! { InfoBlock {} },
                    SettingsSection::Preferences => rsx! { p { class: "settings__note", "{s.settings.coming_soon}" } },
                }
            }
        }
    }
}
