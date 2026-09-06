use dioxus::prelude::*;

#[component]
pub fn DesktopMode() -> Element {
    let mut loaded = use_signal(|| false);

    rsx! {
        main { id: "eclipse",
            class: "min-h-screen w-full bg-black text-white font-mono flex flex-col items-center justify-center gap-3 px-6 transition-opacity duration-500 ease-in",
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

            h1 { class: "text-lg tracking-wide", "This is a Desktop Mode." }
        }
    }
}
