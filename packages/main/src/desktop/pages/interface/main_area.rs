use dioxus::prelude::*;

use super::dock::DockIcons;
use crate::desktop::gui::{AppId, GuiApps};

/// The bordered "application window" region. Shows the active app's window
/// when one is open; otherwise shows a quiet empty state on desktop, or the
/// floating vertical dock on mobile (where the dock normally lives centered
/// on screen rather than in the footer — see the Figma mobile reference).
#[component]
pub fn MainArea(active_app: Signal<Option<AppId>>, dock_open: Signal<bool>) -> Element {
    let body = match active_app() {
        Some(app) => rsx! {
            GuiApps { app, on_close: move |_| active_app.set(None) }
        },
        None => rsx! {
            EmptyOrDock { active_app, dock_open }
        },
    };

    rsx! {
        div { class: "flex min-h-0 flex-1 items-center justify-center gap-2.5 self-stretch p-2.5",
            div {
                class: "relative min-h-0 flex-1 self-stretch overflow-hidden rounded-[5px] border border-white/25",
                {body}
            }
        }
    }
}

#[component]
fn EmptyOrDock(active_app: Signal<Option<AppId>>, dock_open: Signal<bool>) -> Element {
    let dock_class = if dock_open() {
        "flex h-full w-full flex-col items-center justify-center gap-3 md:hidden"
    } else {
        "hidden h-full w-full flex-col items-center justify-center gap-3 md:hidden"
    };
    let hint_class = if dock_open() {
        "hidden h-full w-full flex-col items-center justify-center gap-2 text-center md:hidden"
    } else {
        "flex h-full w-full flex-col items-center justify-center gap-2 text-center md:hidden"
    };

    rsx! {
        Fragment {
            // Desktop: quiet empty state.
            div { class: "hidden h-full w-full flex-col items-center justify-center gap-2 text-center md:flex",
                p { class: "text-sm text-white/40 font-['JetBrains_Mono']", "Belum ada aplikasi yang dibuka." }
                p { class: "text-xs text-white/25 font-['JetBrains_Mono']", "Pilih salah satu dari dock di bawah." }
            }

            // Mobile: floating vertical dock, shown until an app is picked.
            div { class: dock_class,
                div { class: "h-6 w-px bg-white/40" }
                DockIcons {
                    active: active_app,
                    on_pick: move |app| {
                        active_app.set(Some(app));
                        dock_open.set(false);
                    },
                    size: 36,
                    vertical: true,
                }
                div { class: "h-6 w-px bg-white/40" }
            }

            // Mobile: shown instead of the dock once it's been dismissed.
            div { class: hint_class,
                p { class: "text-sm text-white/40 font-['JetBrains_Mono']", "Dock disembunyikan." }
                p { class: "text-xs text-white/25 font-['JetBrains_Mono']", "Ketuk logo NixOS di footer untuk memunculkannya lagi." }
            }
        }
    }
}
