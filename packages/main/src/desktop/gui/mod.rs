use dioxus::prelude::*;

#[component]
pub fn GuiApps() -> Element {

    rsx! {
        h1 { class: "text-lg tracking-wide", "Window App Stuff" }
    }
}
