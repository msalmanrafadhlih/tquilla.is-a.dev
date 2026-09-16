use dioxus::prelude::*;

mod calculator;
mod dock;
mod file_manager;
mod navbar;
mod system_info;
mod window_frame;

use calculator::Calculator;
use dock::Dock;
use file_manager::FileManagerWindowContent;
use navbar::Navbar;
use system_info::{AboutWindowContent, SettingsWindowContent};
use window_frame::{AppId, OpenWindow, WindowFrame, WINDOW_MANAGER_JS};

use crate::desktop::clock::use_live_clock;

/// Picks which app's UI fills a `WindowFrame`. A few apps aren't built
/// yet — they still open (and drag/resize/close like any other window),
/// just with a placeholder inside, so the whole dock is clickable now
/// rather than only the apps that happen to be finished.
#[component]
fn AppContent(id: AppId) -> Element {
    match id {
        AppId::Calculator => rsx! {
            Calculator {}
        },
        AppId::About => rsx! {
            AboutWindowContent {}
        },
        AppId::Settings => rsx! {
            SettingsWindowContent {}
        },
        AppId::FileManager => rsx! {
            FileManagerWindowContent {}
        },
        _ => rsx! {
            div { class: "flex items-center justify-center h-full text-white/30 text-sm", "Coming soon." }
        },
    }
}

/// Desktop view. Mirrors the "Desktop + GUIs" Figma frame: a navbar, a
/// canvas of draggable/resizable app windows, and a launcher dock.
#[component]
pub fn DesktopMode(on_toggle: EventHandler<()>) -> Element {
    let mut loaded = use_signal(|| false);
    let open_windows: Signal<Vec<OpenWindow>> = use_signal(Vec::new);
    let next_z = use_signal(|| 1i32);
    let (time, date) = use_live_clock();

    rsx! {
        main {
            class: "relative h-screen w-full overflow-hidden bg-black text-white font-mono flex flex-col transition-opacity duration-500 ease-in",
            class: if loaded() { "opacity-100" } else { "opacity-0" },
            onmounted: move |_| {
                spawn(async move {
                    document::eval(WINDOW_MANAGER_JS).await.ok();
                    document::eval(
                            "await new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));",
                        )
                        .await
                        .ok();
                    loaded.set(true);
                });
            },

            Navbar { time, date, on_toggle }

            div { class: "relative flex-1 min-h-0",
                for ow in open_windows() {
                    WindowFrame {
                        key: "{ow.id.key()}",
                        id: ow.id,
                        z: ow.z,
                        maximized: ow.maximized,
                        minimized: ow.minimized,
                        open_windows,
                        next_z,
                        AppContent { id: ow.id }
                    }
                }
            }

            Dock { open_windows, next_z }
        }
    }
}
