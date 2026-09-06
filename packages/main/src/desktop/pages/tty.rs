use dioxus::prelude::*;

#[component]
pub fn TerminalMode() -> Element {
    rsx! {
        div { class: "size- p-2.5 right-0 top-0 absolute opacity-70 inline-flex flex-col justify-center items-center",
            div { class: "h-11 px-3.5 outline outline-[0.50px] outline-offset-[-0.50px] outline-white inline-flex justify-center items-center gap-3.5 overflow-hidden",
                div { id: "toggle", class: "justify-center text-white text-[8px] font-normal font-['JetBrains_Mono']", "Terminal", p { "Mode" } }
            }
        }
    }
}

