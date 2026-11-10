use dioxus::prelude::*;

pub use crate::nixdesk::clock::{format_date, format_time, use_live_clock};

/// "shell 3" — the digital clock panel.
#[component]
pub fn ClockPanel(time: Signal<String>, date: Signal<String>) -> Element {
    rsx! {
        div { class: "terminal__panel",
            span { class: "terminal__clock-time", "{time}" }
            span { class: "terminal__clock-date", "{date}" }
        }
    }
}
