use dioxus::prelude::*;

const FOLDER_NAMES: [&str; 5] = ["Documents", "Downloads", "Musics", "Pictures", "Videos"];

#[component]
pub fn FileManagerWindowContent() -> Element {
    let mut current = use_signal(|| Option::<&'static str>::None);
    let breadcrumb_suffix = current().unwrap_or("");

    rsx! {
        div { class: "flex h-full text-[11px]",
            div { class: "w-32 shrink-0 border-r border-white/15 py-2",
                button {
                    r#type: "button",
                    class: "w-full text-left px-3 py-1.5",
                    class: if current().is_none() { "bg-white text-black" } else { "text-white/60 hover:text-white" },
                    onclick: move |_| current.set(None),
                    "Desktops"
                }
                for name in FOLDER_NAMES {
                    button {
                        key: "{name}",
                        r#type: "button",
                        class: "w-full text-left px-3 py-1.5",
                        class: if current() == Some(name) { "bg-white text-black" } else { "text-white/60 hover:text-white" },
                        onclick: move |_| current.set(Some(name)),
                        "{name}"
                    }
                }
            }
            div { class: "flex-1 min-w-0 flex flex-col",
                div { class: "px-3 py-1.5 border-b border-white/15 text-white/40 truncate",
                    "home/tquilla/{breadcrumb_suffix}"
                }
                div { class: "flex-1 min-h-0 overflow-y-auto p-4",
                    if current().is_none() {
                        div { class: "grid grid-cols-3 gap-4",
                            for name in FOLDER_NAMES {
                                div {
                                    key: "{name}",
                                    class: "flex flex-col items-center gap-1.5 cursor-pointer hover:opacity-70",
                                    onclick: move |_| current.set(Some(name)),
                                    div { class: "w-10 h-8 border border-white/30" }
                                    span { class: "text-white/70", "{name}" }
                                }
                            }
                        }
                    } else {
                        p { class: "text-white/30", "(empty)" }
                    }
                }
            }
        }
    }
}
