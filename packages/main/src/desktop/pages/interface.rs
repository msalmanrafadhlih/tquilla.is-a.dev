use dioxus::prelude::*;

#[component]
pub fn DesktopMode(on_toggle: EventHandler<()>) -> Element {
    let mut loaded = use_signal(|| false);

    rsx! {
        main { id: "eclipse",
            class: "relative min-h-screen w-full bg-black text-white font-mono flex flex-col items-center justify-center gap-3 px-6 transition-opacity duration-500 ease-in",
            class: if loaded() { "opacity-100" } else { "opacity-0" },
            onmounted: move |_| {
                spawn(async move {
                    // beri browser 1 frame untuk render state awal (opacity-0)
                    // sebelum transisi di-trigger, mirip double-rAF di JS
                    document::eval(
                        "await new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));"
                    ).await.ok();
                    loaded.set(true);
                });
            },

            div { class: "p-2.5 right-0 top-0 absolute inline-flex flex-col justify-center items-center",
                button {
                    id: "toggle",
                    r#type: "button",
                    class: "h-11 px-3.5 text-white opacity-70 outline outline-[0.50px] outline-offset-[-0.50px] outline-white inline-flex justify-center items-center gap-3.5 overflow-hidden cursor-pointer transition-all duration-200 ease-out hover:opacity-100 hover:bg-white hover:text-black",
                    onclick: move |_| on_toggle.call(()),

                    div { class: "justify-center text-[8px] font-normal font-['JetBrains_Mono']",
                        "Desktop"
                        p { "Mode" }
                    }
                }
            }

            h1 { class: "text-lg tracking-wide", "This is a Desktop Mode." }
        }
    }
}
