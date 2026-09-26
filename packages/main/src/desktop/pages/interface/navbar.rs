use dioxus::prelude::*;

const NIXOS_ICON: Asset = asset!("/assets/Icon-nixos.svg");
const WIFI_ICON: Asset = asset!("/assets/icon-wifi.svg");
const VOLUME_ICON: Asset = asset!("/assets/icon-volume.svg");
const BRIGHTNESS_ICON: Asset = asset!("/assets/icon-color-scheme.svg");
const BATTERY_ICON: Asset = asset!("/assets/icon-battery.svg");
const NOTIFICATION_ICON: Asset = asset!("/assets/icon-notification.svg");
const NOTIFICATION_ICON_SILENT: Asset = asset!("/assets/icon-notification-silent.svg");
const TOGGLE_ICON: Asset = asset!("/assets/icon-toggle-popup.svg");

/// Decorative only for now — a real dropdown per item is more than this
/// pass covers, matching classic desktop chrome without the behavior.
const MENU_ITEMS: [&str; 6] = ["File", "Edit", "View", "Go", "Tools", "Settings"];

/// Mirrors the "Top Bar" of the Desktop + GUIs Figma frame: logo + menu on
/// the left, a centered clock, system-status icons and the terminal-mode
/// switch on the right.
///
/// The logo/menu column and the status-icon column each collapse from a
/// row into a stack below their own breakpoint (`lg` for the menu, `md`
/// for the status icons — matching the reference export), which is what
/// turns this into the two-column "NixMobile" layout instead of clipping
/// or simply disappearing on narrow screens.
#[component]
pub fn Navbar(time: Signal<String>, date: Signal<String>, on_toggle: EventHandler<()>, brightness: Signal<u8>) -> Element {
    let mut show_about = use_signal(|| false);
    let mut active_menu = use_signal(|| Option::<usize>::None);
    let mut wifi_on = use_signal(|| true);
    let mut muted = use_signal(|| false);
    let mut notif_muted = use_signal(|| true);
    let mut show_battery_tip = use_signal(|| false);

    let menu_hint = match active_menu() {
        Some(i) => format!("Menu \"{}\" — masih dekorasi, belum ada isinya.", MENU_ITEMS[i]),
        None => String::new(),
    };

    let about_class = if show_about() {
        "pointer-events-none absolute left-0 top-full z-20 mt-1 w-max max-w-[220px] rounded-md border border-white/30 bg-black px-2 py-1 text-[11px] text-white/70 opacity-100 transition-opacity duration-150 font-['JetBrains_Mono']"
    } else {
        "pointer-events-none absolute left-0 top-full z-20 mt-1 w-max max-w-[220px] rounded-md border border-white/30 bg-black px-2 py-1 text-[11px] text-white/70 opacity-0 transition-opacity duration-150 font-['JetBrains_Mono']"
    };

    let menu_hint_class = if active_menu().is_some() {
        "pointer-events-none absolute left-0 top-full z-20 mt-1 whitespace-nowrap rounded-md border border-white/30 bg-black px-2.5 py-1.5 text-[11px] text-white/60 opacity-100 transition-opacity duration-150 font-['JetBrains_Mono']"
    } else {
        "pointer-events-none absolute left-0 top-full z-20 mt-1 whitespace-nowrap rounded-md border border-white/30 bg-black px-2.5 py-1.5 text-[11px] text-white/60 opacity-0 transition-opacity duration-150 font-['JetBrains_Mono']"
    };

    let battery_tip_class = if show_battery_tip() {
        "pointer-events-none absolute right-0 top-full z-20 mt-1 whitespace-nowrap rounded-md border border-white/30 bg-black px-2 py-1 text-[10px] text-white/70 opacity-100 transition-opacity duration-150 font-['JetBrains_Mono']"
    } else {
        "pointer-events-none absolute right-0 top-full z-20 mt-1 whitespace-nowrap rounded-md border border-white/30 bg-black px-2 py-1 text-[10px] text-white/70 opacity-0 transition-opacity duration-150 font-['JetBrains_Mono']"
    };

    let menu_item_class = |active: bool| -> &'static str {
        if active {
            "w-full border-b border-white bg-white/10 px-2 py-2 text-left text-xs text-white transition-colors duration-150 font-['JetBrains_Mono'] lg:w-auto lg:rounded-sm lg:border-b-0 lg:px-2 lg:py-1"
        } else {
            "w-full border-b border-white px-2 py-2 text-left text-xs text-white/70 transition-colors duration-150 ease-out hover:bg-white/5 hover:text-white font-['JetBrains_Mono'] lg:w-auto lg:rounded-sm lg:border-b-0 lg:px-2 lg:py-1"
        }
    };

    rsx! {
        header { class: "relative z-[999] flex w-full shrink-0 items-center justify-center gap-2.5 px-[15px] py-2 text-xs rounded-3xl border-t-[0.5px] border-x border-white backdrop-blur-sm [background:linear-gradient(180deg,rgba(37,37,37,1)_10%,transparent_100%)]",

            // Centered clock — always visible; only the date line drops below `sm`.
            div { class: "absolute left-1/2 top-1/2 z-[4] flex -translate-x-1/2 -translate-y-1/2 select-none flex-col items-center gap-1",
                span { class: "text-center text-sm font-light", "{time}" }
                span { class: "hidden text-[12px] text-white/40 sm:block", "{date}" }
            }

            // LEFT — logo + title + menu bar.
            div { class: "flex min-w-0 flex-col items-start gap-3 h-[35px] lg:flex-row lg:items-center",
                button {
                    r#type: "button",
                    class: "group relative flex shrink-0 items-center gap-2 rounded-md px-1 py-1 transition-colors duration-150 ease-out hover:bg-white/5 active:scale-95",
                    onclick: move |_| show_about.set(!show_about()),
                    img { src: NIXOS_ICON, alt: "NixDesktop logo", class: "h-[26px] w-[26px]" }
                    span { class: "hidden text-sm font-normal lg:inline", "NixDesktop" }
                    span { class: "text-sm font-normal lg:hidden", "NixMobile" }
                    div { class: about_class, "Desktop simulasi pribadi milik Moch." }
                }

                nav {
                    class: "relative flex w-full flex-col items-stretch gap-0.5 lg:w-auto lg:flex-row",
                    "aria-label": "Main navigation",
                    for (i , label) in MENU_ITEMS.into_iter().enumerate() {
                        button {
                            key: "{label}",
                            r#type: "button",
                            class: menu_item_class(active_menu() == Some(i)),
                            onclick: move |_| {
                                let next = if active_menu() == Some(i) { None } else { Some(i) };
                                active_menu.set(next);
                            },
                            "{label}"
                        }
                    }
                    div { class: menu_hint_class, "{menu_hint}" }
                }
            }

            div { class: "h-[30px] flex-1", "aria-hidden": "true" }

            // RIGHT — system-status icons.
            div { class: "flex flex-none flex-col h-[20px] items-center gap-[25px] md:flex-row",
                img { src: TOGGLE_ICON, alt: "Toggle", class: "w-[20px] cursor-pointer sm:hidden" }

                div { class: "flex flex-col items-center gap-3 md:flex-row", "aria-label": "System status",
                    button {
                        r#type: "button",
                        class: "relative transition-transform duration-150 ease-out hover:scale-125 active:scale-90",
                        title: if wifi_on() { "Wi-Fi aktif" } else { "Wi-Fi nonaktif" },
                        onclick: move |_| wifi_on.set(!wifi_on()),
                        img {
                            src: WIFI_ICON, alt: "Wi-Fi",
                            class: if wifi_on() { "h-4 w-4 opacity-100 transition-opacity duration-150" } else { "h-4 w-4 opacity-30 transition-opacity duration-150" },
                        }
                        span {
                            class: if !wifi_on() { "pointer-events-none absolute left-[-1px] top-1/2 h-px w-5 -translate-y-1/2 rotate-45 bg-white" } else { "pointer-events-none absolute left-[-1px] top-1/2 h-px w-5 -translate-y-1/2 rotate-45 bg-transparent" },
                        }
                    }

                    button {
                        r#type: "button",
                        class: "relative transition-transform duration-150 ease-out hover:scale-125 active:scale-90",
                        title: if muted() { "Suara dibisukan" } else { "Suara aktif" },
                        onclick: move |_| muted.set(!muted()),
                        img {
                            src: VOLUME_ICON, alt: "Volume",
                            class: if muted() { "h-4 w-4 opacity-30 transition-opacity duration-150" } else { "h-4 w-4 opacity-100 transition-opacity duration-150" },
                        }
                        span {
                            class: if muted() { "pointer-events-none absolute left-[-1px] top-1/2 h-px w-5 -translate-y-1/2 rotate-45 bg-white" } else { "pointer-events-none absolute left-[-1px] top-1/2 h-px w-5 -translate-y-1/2 rotate-45 bg-transparent" },
                        }
                    }

                    button {
                        r#type: "button",
                        class: "transition-transform duration-150 ease-out hover:scale-125 active:scale-90",
                        title: "Kecerahan layar",
                        onclick: move |_| brightness.set((brightness() + 1) % 3),
                        img {
                            src: BRIGHTNESS_ICON, alt: "Brightness",
                            class: if brightness() == 0 { "h-4 w-4 opacity-100 transition-opacity duration-150" } else if brightness() == 1 { "h-4 w-4 opacity-70 transition-opacity duration-150" } else { "h-4 w-4 opacity-40 transition-opacity duration-150" },
                        }
                    }

                    button {
                        r#type: "button",
                        class: "transition-transform duration-150 ease-out hover:scale-125 active:scale-90",
                        title: if notif_muted() { "Notifikasi dibisukan" } else { "Notifikasi aktif" },
                        onclick: move |_| notif_muted.set(!notif_muted()),
                        if notif_muted() {
                            img {
                                src: NOTIFICATION_ICON_SILENT, alt: "Notifications",
                                class: "h-4 w-4 opacity-100"
                            }
                        } else {
                            img {
                                src: NOTIFICATION_ICON, alt: "Notifications",
                                class: "h-4 w-4 opacity-100"
                            }
                        }
                    }

                    div { class: "relative",
                        button {
                            r#type: "button",
                            class: "flex flex-col items-center gap-0.5 transition-transform duration-150 ease-out hover:scale-110 active:scale-90",
                            title: "Baterai",
                            onclick: move |_| show_battery_tip.set(!show_battery_tip()),
                            img { src: BATTERY_ICON, alt: "Battery", class: "h-2 w-4.5 object-contain" }
                            span { class: "text-[8px] font-normal font-['JetBrains_Mono']", "100%" }
                        }
                        div { class: battery_tip_class, "100% · Terisi penuh" }
                    }
                }
            }

            // Terminal-mode switch — icon-only on mobile, label from `sm` up.
            button {
                r#type: "button",
                class: "group flex flex-none items-center gap-1",
                onclick: move |_| on_toggle.call(()),
                svg {
                    class: "sm:hidden transition-transform duration-150 ease-out hover:scale-125 active:scale-90",
                    width: "20", height: "20", view_box: "0 0 100 100", fill: "none", xmlns: "http://www.w3.org/2000/svg",
                    path {
                        d: "M61.375 73.1875H38.4583M17.7083 17.5C15.4982 17.5 13.3786 18.378 11.8158 19.9408C10.253 21.5036 9.375 23.6232 9.375 25.8333V74.1667C9.375 76.3768 10.253 78.4964 11.8158 80.0592C13.3786 81.622 15.4982 82.5 17.7083 82.5H82.2917C84.5018 82.5 86.6214 81.622 88.1842 80.0592C89.747 78.4964 90.625 76.3768 90.625 74.1667V25.8333C90.625 23.6232 89.747 21.5036 88.1842 19.9408C86.6214 18.378 84.5018 17.5 82.2917 17.5H17.7083ZM18.75 48.25L34.8333 60.75L18.75 73.1875V48.1875V48.25Z",
                        stroke: "white", stroke_width: "5", stroke_linecap: "round", stroke_linejoin: "round",
                    }
                }
                span { class: "hidden text-[10px] text-center sm:block transition-transform duration-150 ease-out hover:scale-110 active:scale-90 sm:h-full sm:px-2",
                    "Switch "
                    p { "TTY" }
                }
            }
        }
    }
}
