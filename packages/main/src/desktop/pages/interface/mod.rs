use dioxus::prelude::*;

mod dock;
mod footer;
mod main_area;
mod navbar;

use crate::desktop::gui::AppId;
use footer::Footer;
use main_area::MainArea;
use navbar::Navbar;

/// The full "OS desktop" simulation: a responsive shell (navbar, dock,
/// footer) around a window manager for the six dock apps. Every element
/// that was labelled `button - *` in the original design is now a real,
/// hoverable, clickable button — see `navbar.rs`, `dock.rs`, `footer.rs`
/// and `../../gui/mod.rs` for the pieces.
#[component]
pub fn DesktopMode(on_toggle: EventHandler<()>) -> Element {
    let mut loaded = use_signal(|| false);
    let active_app = use_signal(|| Option::<AppId>::None);
    let dock_open = use_signal(|| true);
    let brightness = use_signal(|| 0u8);

    let dim_class = if brightness() == 0 {
        "pointer-events-none absolute inset-0 z-30 bg-black opacity-0 transition-opacity duration-300"
    } else if brightness() == 1 {
        "pointer-events-none absolute inset-0 z-30 bg-black opacity-30 transition-opacity duration-300"
    } else {
        "pointer-events-none absolute inset-0 z-30 bg-black opacity-60 transition-opacity duration-300"
    };

    rsx! {
        main { id: "eclipse",
            class: "relative min-h-screen w-full bg-black text-white font-mono inline-flex flex-col items-stretch justify-start overflow-hidden transition-opacity duration-500 ease-in",
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

            Navbar { on_toggle, brightness }
            MainArea { active_app, dock_open }
            Footer { active_app, dock_open }

            div { class: dim_class }
        }
    }
}
