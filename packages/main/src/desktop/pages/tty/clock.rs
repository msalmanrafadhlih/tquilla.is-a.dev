use dioxus::prelude::*;

pub use crate::desktop::clock::{format_date, format_time, use_live_clock};

/// "shell 3" — the digital clock panel.
#[component]
pub fn ClockPanel(time: Signal<String>, date: Signal<String>) -> Element {
    rsx! {
        div { class: "flex flex-col items-center justify-center h-full gap-2 border border-white/15 px-6 py-8 text-center",
            span { class: "text-3xl sm:text-4xl tracking-widest font-semibold", "{time}" }
            span { class: "text-white/50 text-xs sm:text-sm tracking-wide", "{date}" }
        }
    }
}
