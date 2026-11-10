use crate::languages::use_strings;
use dioxus::prelude::*;

/// "shell 5" / "shell 6" — placeholder panels for features not built yet.
#[component]
pub fn ComingSoonPanel(label: &'static str) -> Element {
    let s = use_strings();
    rsx! {
        div { class: "terminal__panel terminal__panel--muted",
            span { class: "terminal__soon-tag", "{label}" }
            span { class: "terminal__soon-text", "{s.terminal.coming_soon}" }
        }
    }
}
