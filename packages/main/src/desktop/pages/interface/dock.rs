use dioxus::prelude::*;

use crate::desktop::gui::{AppIcon, AppId};

/// The desktop dock: nix logo, separator, app icons, separator, a decorative
/// settings icon — rendered horizontally in the footer. Hidden below `md`,
/// where `MainArea` shows a vertical version of `DockIcons` instead.
#[component]
pub fn Dock(active: Signal<Option<AppId>>, on_pick: EventHandler<AppId>) -> Element {
    let mut show_settings_hint = use_signal(|| false);
    let settings_hint_class = if show_settings_hint() {
        "pointer-events-none absolute bottom-full right-0 z-20 mb-1 whitespace-nowrap rounded-md border border-white/30 bg-black px-2 py-1 text-[11px] text-white/60 opacity-100 transition-opacity duration-150 font-['JetBrains_Mono']"
    } else {
        "pointer-events-none absolute bottom-full right-0 z-20 mb-1 whitespace-nowrap rounded-md border border-white/30 bg-black px-2 py-1 text-[11px] text-white/60 opacity-0 transition-opacity duration-150 font-['JetBrains_Mono']"
    };

    rsx! {
        div { class: "hidden shrink-0 items-end gap-5 border-b border-white/50 bg-black px-4 py-2 md:flex",
            svg { width: "31", height: "31", view_box: "0 0 31 31", fill: "none", xmlns: "http://www.w3.org/2000/svg",
                path { d: "M1 14.25H7.5625M10.1687 10.5L3.90625 20.8125", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                path { d: "M7.5625 3L10.375 7.6875M16.9375 8.625H4.84375", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                path { d: "M22.5625 3L19.75 8.625M21.6438 13.5L16 3", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                path { d: "M29.125 16.1625L22.5625 16.125M19.975 19.875L26.3125 9.5625", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                path { d: "M23.5 27.375L18.8125 21.75M13.2438 21.7875L25.3938 21.75", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                path { d: "M7.5625 27.375L10.375 21.75M8.55625 17.0625L14.125 27.375", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
            }
            div { class: "h-8 w-px bg-white/50" }

            DockIcons { active, on_pick, size: 40, vertical: false }

            div { class: "h-8 w-px bg-white/50" }
            div { class: "relative",
                button {
                    r#type: "button",
                    class: "grid place-items-center rounded-md p-1 transition-all duration-150 ease-out hover:bg-white/10 active:scale-90",
                    title: "Pengaturan",
                    onclick: move |_| show_settings_hint.set(!show_settings_hint()),
                    svg { width: "24", height: "24", view_box: "0 0 30 30", fill: "none", xmlns: "http://www.w3.org/2000/svg",
                        path { d: "M15 10.7143V5M7.85714 19.2857V25M22.1429 22.1429V25M15 25V16.4286M7.85714 5V13.5714M22.1429 5V16.4286M12.1429 10.7143H17.8571M5 19.2857H10.7143M19.2857 22.1429H25", stroke: "white", stroke_linecap: "round" }
                    }
                }
                div { class: settings_hint_class, "Panel pengaturan belum tersedia." }
            }
        }
    }
}

/// Row (desktop) or column (mobile) of app-launcher buttons. A small dot
/// lights up under whichever app currently has an open window. Shared so
/// the desktop and mobile docks always stay in sync.
#[component]
pub fn DockIcons(
    active: Signal<Option<AppId>>,
    on_pick: EventHandler<AppId>,
    size: u32,
    vertical: bool,
) -> Element {
    let btn_class = if vertical {
        "group flex flex-col items-center gap-1 rounded-lg p-1.5 transition-all duration-150 ease-out hover:-translate-x-1 hover:bg-white/10 active:scale-90 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-white/70"
    } else {
        "group flex flex-col items-center gap-1 rounded-lg p-1.5 transition-all duration-150 ease-out hover:-translate-y-1.5 hover:bg-white/10 active:scale-90 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-white/70"
    };

    rsx! {
        for app in AppId::ALL {
            button {
                key: "{app:?}",
                r#type: "button",
                class: btn_class,
                title: app.label(),
                onclick: move |_| on_pick.call(app),
                AppIcon { app, size }
                span {
                    class: if active() == Some(app) { "block h-1 w-1 rounded-full bg-white" } else { "block h-1 w-1 rounded-full bg-transparent" },
                }
            }
        }
    }
}
