use dioxus::prelude::*;

/// "shell 5" / "shell 6" — placeholder panels for features not built yet.
#[component]
pub fn ComingSoonPanel(label: &'static str) -> Element {
    rsx! {
        div { class: "flex flex-col items-center justify-center h-full gap-2 border border-white/15 px-6 py-8 text-center text-white/30",
            span { class: "text-[10px] tracking-[0.2em] uppercase", "{label}" }
            span { class: "text-sm", "Coming Soon" }
        }
    }
}
