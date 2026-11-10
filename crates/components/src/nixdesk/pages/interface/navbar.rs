use crate::languages::use_strings;
use crate::nixdesk::clock::use_clock_format;
use dioxus::prelude::*;

const NIXOS_ICON: Asset = asset!("/assets/Icon-nixos.svg");
const WIFI_ICON: Asset = asset!("/assets/icon-wifi.svg");
const VOLUME_ICON: Asset = asset!("/assets/icon-volume.svg");
const BRIGHTNESS_ICON: Asset = asset!("/assets/icon-color-scheme.svg");
const BATTERY_ICON: Asset = asset!("/assets/icon-battery.svg");
const NOTIFICATION_ICON: Asset = asset!("/assets/icon-notification.svg");
#[allow(dead_code)]
const NOTIFICATION_ICON_SILENT: Asset = asset!("/assets/icon-notification-silent.svg");
const TOGGLE_ICON: Asset = asset!("/assets/icon-toggle-popup.svg");
const ICON_CLOSE: Asset = asset!("/assets/icon-close.svg");

// Item menu (File, Edit, ...) hanya dekorasi untuk saat ini: dropdown
// sungguhan belum ada. Labelnya ada di `Strings::navbar.menu_items()`.

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
pub fn Navbar(
    time: Signal<String>,
    date: Signal<String>,
    on_toggle: EventHandler<()>,
    brightness: Signal<u8>,
) -> Element {
    let s = use_strings();
    let menu_items = s.navbar.menu_items();
    let mut clock_format = use_clock_format();
    let mut show_about = use_signal(|| false);
    let mut active_menu = use_signal(|| Option::<usize>::None);
    let mut nav_menu_open = use_signal(|| false);
    let mut wifi_on = use_signal(|| true);
    let mut muted = use_signal(|| false);
    let mut notif_muted = use_signal(|| false);
    let mut show_battery_tip = use_signal(|| false);
    let mut status_menu_open = use_signal(|| false);

    let menu_hint = match active_menu() {
        Some(i) => s.navbar.menu_hint(menu_items[i]),
        None => String::new(),
    };

    let about_class = if show_about() {
        "navbar__tip navbar__tip--about navbar__tip--on-xl"
    } else {
        "navbar__tip navbar__tip--about"
    };

    let menu_hint_class = if active_menu().is_some() {
        "navbar__tip navbar__tip--hint navbar__tip--on"
    } else {
        "navbar__tip navbar__tip--hint navbar__tip--off"
    };

    let battery_tip_class = if show_battery_tip() {
        "navbar__tip navbar__tip--battery navbar__tip--on"
    } else {
        "navbar__tip navbar__tip--battery navbar__tip--off"
    };

    let menu_item_class = |active: bool| -> &'static str {
        if active {
            "navbar__menu-item navbar__menu-item--active"
        } else {
            "navbar__menu-item navbar__menu-item--idle"
        }
    };

    rsx! {
        header { class: "navbar",

            // Centered clock — always visible; only the date line drops below `sm`.
            // It is a button: click toggles 24-hour <-> 12-hour.
            button {
                r#type: "button",
                class: "navbar__clock",
                title: clock_format().switch_label(s),
                "aria-label": "{time}. {clock_format().switch_label(s)}",
                onclick: move |_| clock_format.set(clock_format().toggled()),
                span { class: "navbar__clock-time", "{time}" }
                span { class: "navbar__clock-date", "{date}" }
            }

            // LEFT — logo + title + menu bar. jika button `NIXOS_ICON` di klik, maka akan memunculkan tag nav/list button di bawahnya!
            div { class: "navbar__left",
                button {
                    r#type: "button",
                    class: "group navbar__menu-btn",
                    "aria-label": s.navbar.open_main_menu,
                    "aria-expanded": if nav_menu_open() { "true" } else { "false" },
                    onclick: move |_| {
                        show_about.set(!show_about());
                        nav_menu_open.set(!nav_menu_open());
                    },
                    img { src: NIXOS_ICON, alt: s.navbar.logo_alt, class: "navbar__logo" }
                    span { class: "navbar__name--wide", "{s.navbar.brand_desktop}" }
                    span { class: "navbar__name--narrow", "{s.navbar.brand_mobile}" }
                    div { class: about_class, "{s.navbar.about_tip}" }
                }

                div {
                    class: if nav_menu_open() {
                        "navbar__menu navbar__menu--open"
                    } else {
                        "navbar__menu navbar__menu--closed"
                    },
                    div { class: "navbar__menu-inner",
                        nav {
                            class: "navbar__menu-list",
                            "aria-label": s.navbar.main_navigation,
                            for (i , label) in menu_items.into_iter().enumerate() {
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
                }
            }

            div { class: "navbar__spacer", "aria-hidden": "true" }

            // RIGHT — system-status icons. jika button `TOGGLE ICON` di klik, maka akan memunculkan tag div/list button di bawahnya!
            div { class: "navbar__right",
                button {
                    r#type: "button",
                    "aria-label": s.navbar.open_status_menu,
                    "aria-expanded": if status_menu_open() { "true" } else { "false" },
                    class: "navbar__status-toggle",
                    onclick: move |_| status_menu_open.set(!status_menu_open()),
                    img {
                        src: if status_menu_open() { ICON_CLOSE } else { TOGGLE_ICON },
                        alt: s.navbar.toggle_alt,
                        class: if status_menu_open() {
                            "navbar__chevron--open"
                        } else {
                            "navbar__chevron--closed"
                        }
                    }
                }

                div {
                    class: if status_menu_open() {
                        "navbar__status navbar__status--open"
                    } else {
                        "navbar__status navbar__status--closed"
                    },
                    div { class: "navbar__status-inner",
                        div { class: "navbar__status-list", "aria-label": s.navbar.system_status,
                            button {
                                r#type: "button",
                                class: "navbar__btn navbar__btn--relative",
                                title: if wifi_on() { s.navbar.wifi_on } else { s.navbar.wifi_off },
                                onclick: move |_| wifi_on.set(!wifi_on()),
                                img {
                                    src: WIFI_ICON, alt: s.navbar.wifi_alt,
                                    class: if wifi_on() { "navbar__icon navbar__icon--on" } else { "navbar__icon navbar__icon--off" },
                                }
                                span {
                                    class: if !wifi_on() { "navbar__slash navbar__slash--on" } else { "navbar__slash navbar__slash--off" },
                                }
                            }

                            button {
                                r#type: "button",
                                class: "navbar__btn navbar__btn--relative",
                                title: if muted() { s.navbar.sound_muted } else { s.navbar.sound_on },
                                onclick: move |_| muted.set(!muted()),
                                img {
                                    src: VOLUME_ICON, alt: s.navbar.volume_alt,
                                    class: if muted() { "navbar__icon navbar__icon--off" } else { "navbar__icon navbar__icon--on" },
                                }
                                span {
                                    class: if muted() { "navbar__slash navbar__slash--on" } else { "navbar__slash navbar__slash--off" },
                                }
                            }

                            button {
                                r#type: "button",
                                class: "navbar__btn",
                                title: s.navbar.brightness_title,
                                onclick: move |_| brightness.set((brightness() + 1) % 3),
                                img {
                                    src: BRIGHTNESS_ICON, alt: s.navbar.brightness_alt,
                                    class: if brightness() == 0 { "navbar__icon navbar__icon--on" } else if brightness() == 1 { "navbar__icon navbar__icon--dim" } else { "navbar__icon navbar__icon--dimmer" },
                                }
                            }

                            button {
                                r#type: "button",
                                class: "navbar__btn navbar__btn--relative",
                                title: if notif_muted() { s.navbar.notifications_muted } else { s.navbar.notifications_on },
                                onclick: move |_| notif_muted.set(!notif_muted()),
                                img {
                                    src: NOTIFICATION_ICON, alt: s.navbar.notification_alt,
                                    class: if notif_muted() { "navbar__icon navbar__icon--off" } else { "navbar__icon navbar__icon--on" },
                                }
                                span {
                                    class: if notif_muted() { "navbar__slash navbar__slash--on" } else { "navbar__slash navbar__slash--off" },
                                }
                            }

                            div { class: "navbar__item",
                                button {
                                    r#type: "button",
                                    class: "navbar__lang-btn",
                                    title: s.navbar.battery_title,
                                    onclick: move |_| show_battery_tip.set(!show_battery_tip()),
                                    img { src: BATTERY_ICON, alt: s.navbar.battery_alt, class: "navbar__lang-flag" }
                                    span { class: "navbar__lang-code", "100%" }
                                }
                                div { class: battery_tip_class, "{s.navbar.battery_tip}" }
                            }
                        }
                    }
                }
            }

            // Terminal-mode switch — icon-only on mobile, label from `sm` up.
            button {
                r#type: "button",
                class: "group navbar__battery-btn",
                onclick: move |_| on_toggle.call(()),
                svg {
                    class: "navbar__btn navbar__battery-icon",
                    width: "20", height: "20", view_box: "0 0 100 100", fill: "none", xmlns: "http://www.w3.org/2000/svg",
                    path {
                        d: "M61.375 73.1875H38.4583M17.7083 17.5C15.4982 17.5 13.3786 18.378 11.8158 19.9408C10.253 21.5036 9.375 23.6232 9.375 25.8333V74.1667C9.375 76.3768 10.253 78.4964 11.8158 80.0592C13.3786 81.622 15.4982 82.5 17.7083 82.5H82.2917C84.5018 82.5 86.6214 81.622 88.1842 80.0592C89.747 78.4964 90.625 76.3768 90.625 74.1667V25.8333C90.625 23.6232 89.747 21.5036 88.1842 19.9408C86.6214 18.378 84.5018 17.5 82.2917 17.5H17.7083ZM18.75 48.25L34.8333 60.75L18.75 73.1875V48.1875V48.25Z",
                        stroke: "white", stroke_width: "5", stroke_linecap: "round", stroke_linejoin: "round",
                    }
                }
                span { class: "navbar__battery-text",
                    "{s.navbar.switch}"
                    p { "{s.navbar.tty}" }
                }
            }
        }
    }
}
