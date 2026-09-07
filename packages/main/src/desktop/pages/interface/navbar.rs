use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;

const MENU_ITEMS: [&str; 6] = ["File", "Edit", "View", "Go", "Tools", "Settings"];

/// Top bar: logo + menu on the left, a live clock centered (desktop only),
/// status icons + the terminal-mode switch on the right. Every element that
/// was tagged `button - *` in the original design is a real, hoverable,
/// clickable `button` here.
#[component]
pub fn Navbar(on_toggle: EventHandler<()>, brightness: Signal<u8>) -> Element {
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
        nav { class: "w-full shrink-0 border-b border-white/10 px-2 py-2 sm:px-3.5",
            div { class: "grid w-full grid-cols-[1fr_auto_1fr] items-center gap-2",

                // ---------- left: logo + menu ----------
                div { class: "flex min-w-0 items-center gap-3 justify-self-start",
                    button {
                        r#type: "button",
                        class: "group relative flex shrink-0 items-center gap-2 rounded-md px-1 py-1 transition-colors duration-150 ease-out hover:bg-white/5 active:scale-95",
                        onclick: move |_| show_about.set(!show_about()),
                        svg { width: "26", height: "26", view_box: "0 0 30 30", fill: "none", xmlns: "http://www.w3.org/2000/svg",
                            path { d: "M0.9375 14.0625H7.5M10.1062 10.3125L3.84375 20.625", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                            path { d: "M7.5 2.8125L10.3125 7.5M16.875 8.4375H4.78125", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                            path { d: "M22.5 2.8125L19.6875 8.4375M21.5813 13.3125L15.9375 2.8125", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                            path { d: "M29.0625 15.975L22.5 15.9375M19.9125 19.6875L26.25 9.375", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                            path { d: "M23.4375 27.1875L18.75 21.5625M13.1813 21.6L25.3313 21.5625", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                            path { d: "M7.5 27.1875L10.3125 21.5625M8.49375 16.875L14.0625 27.1875", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                        }
                        span { class: "hidden text-base font-normal sm:inline font-['JetBrains_Mono']", "NixDesktop" }
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

                // ---------- center: live clock ----------
                Clock {}

                // ---------- right: status icons + switch mode ----------
                div { class: "flex items-center gap-3 justify-self-end",

                    button {
                        r#type: "button",
                        class: "relative transition-transform duration-150 ease-out hover:scale-125 active:scale-90",
                        title: if wifi_on() { "Wi-Fi aktif" } else { "Wi-Fi nonaktif" },
                        onclick: move |_| wifi_on.set(!wifi_on()),
                        svg {
                            width: "18", height: "18", view_box: "0 0 18 18", fill: "none", xmlns: "http://www.w3.org/2000/svg",
                            class: if wifi_on() { "opacity-100 transition-opacity duration-150" } else { "opacity-30 transition-opacity duration-150" },
                            path { d: "M2.19145 8.08275C3.04739 7.12486 4.09552 6.35804 5.26757 5.83223C6.43962 5.30642 7.7093 5.03342 8.99389 5.03101C10.2785 5.02861 11.5492 5.29685 12.7232 5.81827C13.8972 6.33968 14.9482 7.10257 15.8077 8.05725C16.1347 8.41425 16.6635 7.8825 16.3379 7.527C15.4087 6.50268 14.275 5.68458 13.01 5.12551C11.7451 4.56644 10.3769 4.27883 8.99391 4.28123C7.61091 4.28363 6.24377 4.576 4.98074 5.13946C3.71772 5.70292 2.58687 6.52495 1.6612 7.5525C1.3372 7.90875 1.86595 8.4405 2.19145 8.08275Z", fill: "white" }
                            path { d: "M4.24055 9.87675C4.84454 9.21651 5.57908 8.68892 6.39763 8.32741C7.21618 7.96591 8.1009 7.77837 8.99572 7.77667C9.89055 7.77498 10.776 7.95918 11.5959 8.31758C12.4158 8.67599 13.1523 9.2008 13.7588 9.85875C14.0866 10.215 14.6161 9.68325 14.2891 9.3285C13.6124 8.60154 12.793 8.02209 11.8821 7.62642C10.9713 7.23076 9.98849 7.02741 8.99538 7.0291C8.00226 7.03079 7.02019 7.23748 6.11065 7.63624C5.20111 8.035 4.38369 8.61724 3.70955 9.3465C3.3848 9.702 3.91355 10.2345 4.24055 9.87675Z", fill: "white" }
                            path { d: "M6.52501 11.619C6.83846 11.2746 7.22023 10.9992 7.64599 10.8105C8.07175 10.6218 8.53215 10.5239 8.99786 10.523C9.46356 10.522 9.92435 10.6182 10.3508 10.8052C10.7773 10.9922 11.1602 11.2661 11.475 11.6092C11.8035 11.9647 12.3323 11.433 12.0053 11.079C11.6203 10.6668 11.1546 10.3383 10.6371 10.114C10.1196 9.88975 9.5615 9.77447 8.99751 9.77539C8.43352 9.7763 7.87576 9.89339 7.35901 10.1194C6.84227 10.3453 6.37761 10.6753 5.99401 11.0887C5.66851 11.4442 6.19801 11.976 6.52501 11.619Z", fill: "white" }
                            path { d: "M8.9993 13.719C9.46322 13.719 9.8393 13.3429 9.8393 12.879C9.8393 12.4151 9.46322 12.039 8.9993 12.039C8.53538 12.039 8.1593 12.4151 8.1593 12.879C8.1593 13.3429 8.53538 13.719 8.9993 13.719Z", fill: "white" }
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
                            width: "18", height: "18", view_box: "0 0 18 18", fill: "none", xmlns: "http://www.w3.org/2000/svg",
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
                        svg {
                            width: "18", height: "18", view_box: "0 0 18 18", fill: "none", xmlns: "http://www.w3.org/2000/svg",
                            class: if brightness() == 0 { "opacity-100 transition-opacity duration-150" } else if brightness() == 1 { "opacity-70 transition-opacity duration-150" } else { "opacity-40 transition-opacity duration-150" },
                            path { d: "M9.00002 1.8C9.11937 1.8 9.23383 1.84741 9.31822 1.9318C9.40261 2.01619 9.45002 2.13065 9.45002 2.25V3.15C9.45002 3.26935 9.40261 3.38381 9.31822 3.4682C9.23383 3.55259 9.11937 3.6 9.00002 3.6C8.88068 3.6 8.76622 3.55259 8.68182 3.4682C8.59743 3.38381 8.55002 3.26935 8.55002 3.15V2.25C8.55002 2.13065 8.59743 2.01619 8.68182 1.9318C8.76622 1.84741 8.88068 1.8 9.00002 1.8ZM9.00002 12.6C9.9548 12.6 10.8705 12.2207 11.5456 11.5456C12.2207 10.8705 12.6 9.95478 12.6 9C12.6 8.04522 12.2207 7.12955 11.5456 6.45441C10.8705 5.77928 9.9548 5.4 9.00002 5.4C8.04524 5.4 7.12957 5.77928 6.45444 6.45441C5.77931 7.12955 5.40002 8.04522 5.40002 9C5.40002 9.95478 5.77931 10.8705 6.45444 11.5456C7.12957 12.2207 8.04524 12.6 9.00002 12.6ZM9.00002 11.7C8.28394 11.7 7.59718 11.4155 7.09084 10.9092C6.58449 10.4028 6.30002 9.71608 6.30002 9C6.30002 8.28392 6.58449 7.59716 7.09084 7.09081C7.59718 6.58446 8.28394 6.3 9.00002 6.3C9.71611 6.3 10.4029 6.58446 10.9092 7.09081C11.4156 7.59716 11.7 8.28392 11.7 9C11.7 9.71608 11.4156 10.4028 10.9092 10.9092C10.4029 11.4155 9.71611 11.7 9.00002 11.7ZM15.75 9.45C15.8694 9.45 15.9838 9.40259 16.0682 9.3182C16.1526 9.23381 16.2 9.11935 16.2 9C16.2 8.88065 16.1526 8.76619 16.0682 8.6818C15.9838 8.59741 15.8694 8.55 15.75 8.55H14.85C14.7307 8.55 14.6162 8.59741 14.5318 8.6818C14.4474 8.76619 14.4 8.88065 14.4 9C14.4 9.11935 14.4474 9.23381 14.5318 9.3182C14.6162 9.40259 14.7307 9.45 14.85 9.45H15.75ZM9.00002 14.4C9.11937 14.4 9.23383 14.4474 9.31822 14.5318C9.40261 14.6162 9.45002 14.7307 9.45002 14.85V15.75C9.45002 15.8693 9.40261 15.9838 9.31822 16.0682C9.23383 16.1526 9.11937 16.2 9.00002 16.2C8.88068 16.2 8.76622 16.1526 8.68182 16.0682C8.59743 15.9838 8.55002 15.8693 8.55002 15.75V14.85C8.55002 14.7307 8.59743 14.6162 8.68182 14.5318C8.76622 14.4474 8.88068 14.4 9.00002 14.4ZM3.15002 9.45C3.26937 9.45 3.38383 9.40259 3.46822 9.3182C3.55261 9.23381 3.60002 9.11935 3.60002 9C3.60002 8.88065 3.55261 8.76619 3.46822 8.6818C3.38383 8.59741 3.26937 8.55 3.15002 8.55H2.21672C2.09738 8.55 1.98292 8.59741 1.89853 8.6818C1.81413 8.76619 1.76672 8.88065 1.76672 9C1.76672 9.11935 1.81413 9.23381 1.89853 9.3182C1.98292 9.40259 2.09738 9.45 2.21672 9.45H3.15002ZM3.73142 3.7314C3.77322 3.68949 3.82288 3.65624 3.87755 3.63356C3.93222 3.61087 3.99083 3.59919 4.05002 3.59919C4.10921 3.59919 4.16782 3.61087 4.22249 3.63356C4.27716 3.65624 4.32682 3.68949 4.36862 3.7314L5.26862 4.6314C5.35312 4.7159 5.40059 4.8305 5.40059 4.95C5.40059 5.0695 5.35312 5.1841 5.26862 5.2686C5.18413 5.3531 5.06952 5.40057 4.95002 5.40057C4.83053 5.40057 4.71592 5.3531 4.63142 5.2686L3.73142 4.3686C3.68952 4.3268 3.65627 4.27714 3.63358 4.22247C3.6109 4.1678 3.59922 4.10919 3.59922 4.05C3.59922 3.99081 3.6109 3.9322 3.63358 3.87753C3.65627 3.82286 3.68952 3.7732 3.73142 3.7314ZM4.36862 14.2686C4.28413 14.3531 4.16952 14.4006 4.05002 14.4006C3.93053 14.4006 3.81592 14.3531 3.73142 14.2686C3.64693 14.1841 3.59946 14.0695 3.59946 13.95C3.59946 13.8305 3.64693 13.7159 3.73142 13.6314L4.63142 12.7314C4.71592 12.6469 4.83053 12.5994 4.95002 12.5994C5.06952 12.5994 5.18413 12.6469 5.26862 12.7314C5.35312 12.8159 5.40059 12.9305 5.40059 13.05C5.40059 13.1695 5.35312 13.2841 5.26862 13.3686L4.36862 14.2686ZM14.2686 3.7314C14.2268 3.68949 14.1772 3.65624 14.1225 3.63356C14.0678 3.61087 14.0092 3.59919 13.95 3.59919C13.8908 3.59919 13.8322 3.61087 13.7776 3.63356C13.7229 3.65624 13.6732 3.68949 13.6314 3.7314L12.7314 4.6314C12.6469 4.7159 12.5995 4.8305 12.5995 4.95C12.5995 5.0695 12.6469 5.1841 12.7314 5.2686C12.8159 5.3531 12.9305 5.40057 13.05 5.40057C13.1695 5.40057 13.2841 5.3531 13.3686 5.2686L14.2686 4.3686C14.3105 4.3268 14.3438 4.27714 14.3665 4.22247C14.3891 4.1678 14.4008 4.10919 14.4008 4.05C14.4008 3.99081 14.3891 3.9322 14.3665 3.87753C14.3438 3.82286 14.3105 3.7732 14.2686 3.7314ZM13.6314 14.2686C13.7159 14.3531 13.8305 14.4006 13.95 14.4006C14.0695 14.4006 14.1841 14.3531 14.2686 14.2686C14.3531 14.1841 14.4006 14.0695 14.4006 13.95C14.4006 13.8305 14.3531 13.7159 14.2686 13.6314L13.3686 12.7314C13.2841 12.6469 13.1695 12.5994 13.05 12.5994C12.9305 12.5994 12.8159 12.6469 12.7314 12.7314C12.6469 12.8159 12.5995 12.9305 12.5995 13.05C12.5995 13.1695 12.6469 13.2841 12.7314 13.3686L13.6314 14.2686Z", fill: "white" }
                        }
                    }

                    div { class: "relative",
                        button {
                            r#type: "button",
                            class: "grid place-items-center transition-transform duration-150 ease-out hover:scale-110 active:scale-90",
                            title: "Baterai",
                            onclick: move |_| show_battery_tip.set(!show_battery_tip()),
                            div { class: "relative h-4 w-4 overflow-hidden",
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
                            svg { width: "18", height: "18", view_box: "0 0 18 18", fill: "none", xmlns: "http://www.w3.org/2000/svg",
                                path { d: "M10.4674 13.6957C10.4674 14.0848 10.3128 14.4581 10.0376 14.7333C9.76241 15.0084 9.38918 15.163 9 15.163C8.61082 15.163 8.23759 15.0084 7.9624 14.7333C7.68721 14.4581 7.53261 14.0848 7.53261 13.6957M2.25 2.25L15.75 15.75M12.2283 12.2283H15.163V12.0815L14.6653 11.5486C13.6533 10.4641 13.0096 9.08782 12.8258 7.61596L12.6286 6.04057C12.5245 5.20425 12.1348 4.42955 11.5253 3.8475C10.9158 3.26545 10.124 2.9118 9.28377 2.84635C8.44355 2.78091 7.60652 3.00769 6.91426 3.48835C6.222 3.969 5.71701 4.67401 5.48472 5.48413M10.1739 12.2283H2.83696V12.0815L3.3347 11.5486C4.34666 10.4641 4.99043 9.08782 5.17422 7.61596L5.21765 7.272", stroke: "white" }
                            }
                        } else {
                            svg { width: "18", height: "18", view_box: "0 0 100 100", fill: "none", xmlns: "http://www.w3.org/2000/svg",
                                path { d: "M74.6896 42.9208C78.2167 61.6187 79.025 62.9292 84.2958 67.0271C89.7396 71.2562 89.5646 81.0062 82.3167 81.0062H60.5187C60.4375 86.7521 55.7667 91.3896 50 91.3896C44.2333 91.3896 39.5646 86.7542 39.4812 81.0042H17.6812C10.4354 81.0042 10.2604 71.2583 15.7062 67.025C20.9771 62.9292 21.7833 61.6187 25.3104 42.9208C28.3062 27.0417 34.875 22.1042 43.0271 19.5208C42.575 18.5396 42.3042 17.4583 42.3042 16.3062C42.3042 14.2652 43.115 12.3077 44.5582 10.8645C46.0015 9.42122 47.9589 8.61041 50 8.61041C52.0411 8.61041 53.9985 9.42122 55.4418 10.8645C56.885 12.3077 57.6958 14.2652 57.6958 16.3062C57.6958 17.4583 57.425 18.5417 56.9729 19.5208C65.1271 22.1021 71.6937 27.0396 74.6896 42.9208Z", stroke: "white", stroke_width: "5.5", stroke_linecap: "round", stroke_linejoin: "round" }
                            }
                        }
                    }

                    button {
                        id: "toggle",
                        r#type: "button",
                        class: "flex items-center justify-center gap-1 rounded-md border border-transparent px-2 py-1 transition-all duration-150 ease-out hover:border-white/30 hover:bg-white/5 active:scale-90",
                        title: "Pindah ke Terminal Mode",
                        onclick: move |_| on_toggle.call(()),
                        svg { class: "md:hidden", width: "18", height: "18", view_box: "0 0 100 100", fill: "none", xmlns: "http://www.w3.org/2000/svg",
                            path { d: "M61.375 73.1875H38.4583M17.7083 17.5C15.4982 17.5 13.3786 18.378 11.8158 19.9408C10.253 21.5036 9.375 23.6232 9.375 25.8333V74.1667C9.375 76.3768 10.253 78.4964 11.8158 80.0592C13.3786 81.622 15.4982 82.5 17.7083 82.5H82.2917C84.5018 82.5 86.6214 81.622 88.1842 80.0592C89.747 78.4964 90.625 76.3768 90.625 74.1667V25.8333C90.625 23.6232 89.747 21.5036 88.1842 19.9408C86.6214 18.378 84.5018 17.5 82.2917 17.5H17.7083ZM18.75 48.25L34.8333 60.75L18.75 73.1875V48.1875V48.25Z", stroke: "white", stroke_width: "5", stroke_linecap: "round", stroke_linejoin: "round" }
                        }
                        div { class: "hidden text-center text-[8px] font-normal leading-tight md:block font-['JetBrains_Mono']",
                            "Switch"
                            p { "TTY" }
                        }
                    }
                }
            }
        }
    }
}

/// Live clock, ticking every second in-browser via `js_sys::Date` (no
/// server round-trip). Click toggles between 12-hour and 24-hour display.
/// Hidden on narrow screens, matching the mobile reference design.
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
            class: "hidden flex-col items-center justify-center rounded-md border border-transparent px-2 py-0.5 transition-colors duration-150 ease-out hover:border-white/20 hover:bg-white/5 active:scale-95 md:flex justify-self-center",
            title: "Klik untuk ganti format 12/24 jam",
            onclick: move |_| use_24h.set(!use_24h()),
            span { class: "text-sm font-light font-['JetBrains_Mono']", "{time}" }
            span { class: "text-xs font-light text-white/70 font-['JetBrains_Mono']", "{date}" }
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
