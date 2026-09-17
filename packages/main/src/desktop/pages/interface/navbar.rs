use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;

const NIXOS_ICON: Asset = asset!("/assets/Icon-Nixos.svg");
const WIFI_ICON: Asset = asset!("/assets/button-wifi.svg");
const VOLUME_ICON: Asset = asset!("/assets/button-volume.svg");
const BRIGHTNESS_ICON: Asset = asset!("/assets/button-brightness.svg");
const BATTERY_ICON: Asset = asset!("/assets/button-battery.svg");
const NOTIFICATION_ICON: Asset = asset!("/assets/button-Notification.svg");
const TERMINAL_ICON: Asset = asset!("/assets/button-terminal.svg");

// Decorative only for now — a real dropdown per item is more than this
// pass covers, matching classic desktop chrome without the behavior.
const MENU_ITEMS: [&str; 6] = ["File", "Edit", "View", "Go", "Tools", "Settings"];


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

    rsx! {
        div { class: "relative flex items-center justify-between h-fill px-4 py-2 shrink-0 text-xs",
            div { class: "flex min-w-0 items-center gap-3 justify-self-start",
                button {
                    r#type: "button",
                    class: "group relative flex shrink-0 items-center gap-2 rounded-md px-1 py-1 transition-colors duration-150 ease-out hover:bg-white/5 active:scale-95",
                    onclick: move |_| show_about.set(!show_about()),
                    svg { width: "20", height: "20", view_box: "0 0 30 30", fill: "none", xmlns: "http://www.w3.org/2000/svg",
                        path { d: "M0.9375 14.0625H7.5M10.1062 10.3125L3.84375 20.625", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                        path { d: "M7.5 2.8125L10.3125 7.5M16.875 8.4375H4.78125", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                        path { d: "M22.5 2.8125L19.6875 8.4375M21.5813 13.3125L15.9375 2.8125", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                        path { d: "M29.0625 15.975L22.5 15.9375M19.9125 19.6875L26.25 9.375", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                        path { d: "M23.4375 27.1875L18.75 21.5625M13.1813 21.6L25.3313 21.5625", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                        path { d: "M7.5 27.1875L10.3125 21.5625M8.49375 16.875L14.0625 27.1875", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                    }
                    span { class: "hidden text-sm font-normal sm:inline", "NixDesktop" }
                    span { class: "sm:hidden text-xs font-normal sm:inline", "NixMobile" }
                    div { class: about_class, "Desktop simulasi pribadi milik Moch." }
                }

                div { class: "relative hidden items-stretch gap-0.5 md:flex",
                    for (i , label) in MENU_ITEMS.into_iter().enumerate() {
                        button {
                            key: "{label}",
                            r#type: "button",
                            class: if active_menu() == Some(i) { "rounded-sm bg-white/10 px-2 py-1 text-xs text-white transition-colors duration-150 font-['JetBrains_Mono']" } else { "rounded-sm px-2 py-1 text-xs text-white/70 transition-colors duration-150 ease-out hover:bg-white/5 hover:text-white font-['JetBrains_Mono']" },
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

            Clock {}

            div { class: "flex items-center gap-3 shrink-0",
                button {
                    r#type: "button",
                    class: "relative transition-transform duration-150 ease-out hover:scale-125 active:scale-90",
                    title: if wifi_on() { "Wi-Fi aktif" } else { "Wi-Fi nonaktif" },
                    onclick: move |_| wifi_on.set(!wifi_on()),
                    img {
                        src: WIFI_ICON, alt: "Wi-Fi",
                        class: if wifi_on() { "w-4 h-4 opacity-100 transition-opacity duration-150" } else { "w-4 h-4 opacity-30 transition-opacity duration-150" },
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
                    svg {
                        width: "15", height: "16", view_box: "0 0 18 18", fill: "none", xmlns: "http://www.w3.org/2000/svg",
                        class: if muted() { "opacity-30 transition-opacity duration-150" } else { "opacity-100 transition-opacity duration-150" },
                        path { d: "M12.0064 6.2655C12.6694 6.76275 13.2218 8.03325 13.1111 9.41438C13.0009 10.464 12.4485 11.3479 12.0064 11.6794M13.9399 4.11113C15.1001 4.995 16.0391 7.26 15.9285 9.74588C15.7628 11.6243 14.8238 13.1708 13.9399 13.8338M2.0625 6.6525V11.4585H5.2665L9.40988 14.7731V3.22688L5.2665 6.6525H2.0625Z", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
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
                        src: BRIGHTNESS_ICON,
                        alt: "Brightness",
                        class: if brightness() == 0 { "w-4 h-4 opacity-100 transition-opacity duration-150" } else if brightness() == 1 { "w-4 h-4 opacity-70 transition-opacity duration-150" } else { "w-4 h-4 opacity-40 transition-opacity duration-150" },
                    }
                }

                div { class: "relative",
                    button {
                        r#type: "button",
                        class: "grid place-items-center transition-transform duration-150 ease-out hover:scale-110 active:scale-90",
                        title: "Baterai",
                        onclick: move |_| show_battery_tip.set(!show_battery_tip()),
                        div { class: "relative h-4 w-4",
                            div { class: "absolute", style: "left: 1.88px; top: 2.25px;",
                                svg { width: "16", height: "10", view_box: "0 0 16 10", fill: "none", xmlns: "http://www.w3.org/2000/svg",
                                    path { d: "M13.625 6.1175H14.75C14.8495 6.1175 14.9448 6.07799 15.0152 6.00767C15.0855 5.93734 15.125 5.84196 15.125 5.7425L15.125 3.24125C15.125 3.14179 15.0855 3.04641 15.0152 2.97609C14.9448 2.90576 14.8495 2.86625 14.75 2.86625L13.625 2.86625M13.625 6.1175V7.85375C13.625 8.05266 13.546 8.24343 13.4053 8.38408C13.2647 8.52473 13.0739 8.60375 12.875 8.60375L1.25 8.60375C1.05109 8.60375 0.860322 8.52473 0.719669 8.38408C0.579017 8.24343 0.5 8.05266 0.5 7.85375L0.5 1.25C0.5 1.05109 0.579017 0.860322 0.719669 0.71967C0.860322 0.579018 1.05109 0.5 1.25 0.5L12.875 0.5C13.0739 0.5 13.2647 0.579018 13.4053 0.71967C13.546 0.860322 13.625 1.05109 13.625 1.25V2.86625M13.625 6.1175L13.625 2.86625", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                                }
                            }
                            div { class: "absolute", style: "left: 5.06px; top: 4.80px;",
                                svg { width: "8", height: "4", view_box: "0 0 8 4", fill: "none", xmlns: "http://www.w3.org/2000/svg",
                                    path { d: "M6.875 2.7005L3.6875 0.500002L3.6875 3.5L0.5 1.2995", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                                }
                            }
                            div { class: "absolute left-px top-[11px] text-center text-[6px] font-normal font-['JetBrains_Mono']", "100%" }
                        }
                    }
                    div { class: battery_tip_class, "100% · Terisi penuh" }
                }

                button {
                    r#type: "button",
                    class: "transition-transform duration-150 ease-out hover:scale-125 active:scale-90",
                    title: if notif_muted() { "Notifikasi dibisukan" } else { "Notifikasi aktif" },
                    onclick: move |_| notif_muted.set(!notif_muted()),
                    if notif_muted() {
                        svg { width: "15", height: "15", view_box: "0 0 18 18", fill: "none", xmlns: "http://www.w3.org/2000/svg",
                            path { d: "M10.4674 13.6957C10.4674 14.0848 10.3128 14.4581 10.0376 14.7333C9.76241 15.0084 9.38918 15.163 9 15.163C8.61082 15.163 8.23759 15.0084 7.9624 14.7333C7.68721 14.4581 7.53261 14.0848 7.53261 13.6957M2.25 2.25L15.75 15.75M12.2283 12.2283H15.163V12.0815L14.6653 11.5486C13.6533 10.4641 13.0096 9.08782 12.8258 7.61596L12.6286 6.04057C12.5245 5.20425 12.1348 4.42955 11.5253 3.8475C10.9158 3.26545 10.124 2.9118 9.28377 2.84635C8.44355 2.78091 7.60652 3.00769 6.91426 3.48835C6.222 3.969 5.71701 4.67401 5.48472 5.48413M10.1739 12.2283H2.83696V12.0815L3.3347 11.5486C4.34666 10.4641 4.99043 9.08782 5.17422 7.61596L5.21765 7.272", stroke: "white" }
                        }
                    } else {
                        svg { width: "15", height: "15", view_box: "0 0 100 100", fill: "none", xmlns: "http://www.w3.org/2000/svg",
                            path { d: "M74.6896 42.9208C78.2167 61.6187 79.025 62.9292 84.2958 67.0271C89.7396 71.2562 89.5646 81.0062 82.3167 81.0062H60.5187C60.4375 86.7521 55.7667 91.3896 50 91.3896C44.2333 91.3896 39.5646 86.7542 39.4812 81.0042H17.6812C10.4354 81.0042 10.2604 71.2583 15.7062 67.025C20.9771 62.9292 21.7833 61.6187 25.3104 42.9208C28.3062 27.0417 34.875 22.1042 43.0271 19.5208C42.575 18.5396 42.3042 17.4583 42.3042 16.3062C42.3042 14.2652 43.115 12.3077 44.5582 10.8645C46.0015 9.42122 47.9589 8.61041 50 8.61041C52.0411 8.61041 53.9985 9.42122 55.4418 10.8645C56.885 12.3077 57.6958 14.2652 57.6958 16.3062C57.6958 17.4583 57.425 18.5417 56.9729 19.5208C65.1271 22.1021 71.6937 27.0396 74.6896 42.9208Z", stroke: "white", stroke_width: "5.5", stroke_linecap: "round", stroke_linejoin: "round" }
                        }
                    }
                }

                button {
                    r#type: "button",
                    class: "flex items-center hover:bg-white hover:text-black transition-colors duration-150 sm:px-2",
                    onclick: move |_| on_toggle.call(()),
                    svg { class: "sm:hidden", width: "15", height: "15", view_box: "0 0 100 100", fill: "none", xmlns: "http://www.w3.org/2000/svg",
                        path { d: "M61.375 73.1875H38.4583M17.7083 17.5C15.4982 17.5 13.3786 18.378 11.8158 19.9408C10.253 21.5036 9.375 23.6232 9.375 25.8333V74.1667C9.375 76.3768 10.253 78.4964 11.8158 80.0592C13.3786 81.622 15.4982 82.5 17.7083 82.5H82.2917C84.5018 82.5 86.6214 81.622 88.1842 80.0592C89.747 78.4964 90.625 76.3768 90.625 74.1667V25.8333C90.625 23.6232 89.747 21.5036 88.1842 19.9408C86.6214 18.378 84.5018 17.5 82.2917 17.5H17.7083ZM18.75 48.25L34.8333 60.75L18.75 73.1875V48.1875V48.25Z", stroke: "white", stroke_width: "5", stroke_linecap: "round", stroke_linejoin: "round" }
                    }
                    // img { src: TERMINAL_ICON, alt: "", class: "sm:hidden w-4 h-4 stroke-white" }
                    span { 
                        class: "text-[10px] hidden sm:block", 
                        "Switch " 
                        p { "TTY" } 
                    }
                }
            }
        }
    }
}

#[component]
fn Clock() -> Element {
    let mut tick = use_signal(|| 0u32);
    let mut use_24h = use_signal(|| false);

    use_effect(move || {
        spawn(async move {
            loop {
                TimeoutFuture::new(1000).await;
                tick.set(tick() + 1);
            }
        });
    });

    let _ = tick();
    let (time, date) = format_clock(use_24h());

    rsx! {
        button {
            r#type: "button",
            class: "absolute hidden flex-col items-center justify-center rounded-md border border-transparent px-2 py-0.5 transition-colors duration-150 ease-out active:scale-95 md:flex top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2",
            title: "Klik untuk ganti format 12/24 jam",
            onclick: move |_| use_24h.set(!use_24h()),
            span { class: "tracking-wide", "{time}" }
            span { class: "text-[10px] text-white/40", "{date}" }
        }
    }
}

fn format_clock(use_24h: bool) -> (String, String) {
    let now = js_sys::Date::new_0();
    let hours24 = now.get_hours() as u32;
    let minutes = now.get_minutes() as u32;
    let weekday = now.get_day() as usize;
    let day = now.get_date() as u32;
    let month = now.get_month() as usize;
    let year = now.get_full_year() as i32;

    const WEEKDAYS: [&str; 7] =
        ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
    const MONTHS: [&str; 12] = [
        "January", "February", "March", "April", "May", "June",
        "July", "August", "September", "October", "November", "December",
    ];

    let time = if use_24h {
        format!("{hours24:02} : {minutes:02}")
    } else {
        let period = if hours24 >= 12 { "PM" } else { "AM" };
        let hour12 = match hours24 % 12 {
            0 => 12,
            h => h,
        };
        format!("{hour12:02} : {minutes:02} {period}")
    };

    let date = format!("{}, {} {}, {}", WEEKDAYS[weekday], MONTHS[month], day, year);
    (time, date)
}
