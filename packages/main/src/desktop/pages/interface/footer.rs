use dioxus::prelude::*;

use super::dock::Dock;
use crate::desktop::gui::AppId;

/// Bottom bar: keyboard toggle on the left, the dock in the middle (desktop)
/// or a show/hide toggle for the mobile floating dock, and a "Live Chat"
/// contact popup on the right.
#[component]
pub fn Footer(active_app: Signal<Option<AppId>>, dock_open: Signal<bool>) -> Element {
    let mut keyboard_open = use_signal(|| false);
    let mut chat_open = use_signal(|| false);

    let keyboard_wrap_class = if keyboard_open() {
        "grid grid-rows-[1fr] transition-[grid-template-rows] duration-200 ease-out"
    } else {
        "grid grid-rows-[0fr] transition-[grid-template-rows] duration-200 ease-out"
    };

    rsx! {
        div { class: "w-full shrink-0",
            div { class: keyboard_wrap_class,
                div { class: "overflow-hidden", VirtualKeyboardPanel {} }
            }

            div { class: "flex w-full items-stretch justify-center gap-2.5 border-t border-white/10",
                button {
                    r#type: "button",
                    class: "flex flex-1 items-center justify-start gap-2.5 self-stretch p-3 transition-colors duration-150 ease-out hover:bg-white/5 sm:p-5",
                    onclick: move |_| keyboard_open.set(!keyboard_open()),
                    svg {
                        class: "h-7 w-7 shrink-0 sm:h-10 sm:w-10",
                        width: "40", height: "40", view_box: "0 0 40 40", fill: "none", xmlns: "http://www.w3.org/2000/svg",
                        path { d: "M6.78333 12.45V16.225H10.5583V12.45H6.78333ZM12.45 12.45V16.225H16.225V12.45H12.45ZM18.1083 12.45V16.225H21.8917V12.45H18.1083ZM23.775 12.45V16.225H27.55V12.45H23.775ZM29.4417 12.45V16.225H33.2167V12.45H29.4417ZM6.78333 18.1083V21.8916H10.5583V18.1083H6.78333ZM12.45 18.1083V21.8916H16.225V18.1083H12.45ZM18.1083 18.1083V21.8916H21.8917V18.1083H18.1083ZM23.775 18.1083V21.8916H27.55V18.1083H23.775ZM29.4417 18.1083V21.8916H33.2167V18.1083H29.4417ZM6.78333 23.775V27.55H10.5583V23.775H6.78333ZM12.45 23.775V27.55H27.55V23.775H12.45ZM29.4417 23.775V27.55H33.2167V23.775H29.4417Z", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                        path { d: "M36.25 29.5833V10.4167C36.25 9.97464 36.0744 9.55072 35.7618 9.23816C35.4493 8.9256 35.0254 8.75 34.5833 8.75H5.41667C4.97464 8.75 4.55072 8.9256 4.23816 9.23816C3.92559 9.55072 3.75 9.97464 3.75 10.4167V29.5833C3.75 30.0254 3.92559 30.4493 4.23816 30.7618C4.55072 31.0744 4.97464 31.25 5.41667 31.25H34.5833C35.0254 31.25 35.4493 31.0744 35.7618 30.7618C36.0744 30.4493 36.25 30.0254 36.25 29.5833Z", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                    }
                    span { class: "hidden text-lg font-['JetBrains_Mono'] sm:inline", "Virt. Keyboard" }
                }

                div { class: "hidden items-center md:flex",
                    Dock { active: active_app, on_pick: move |app| active_app.set(Some(app)) }
                }
                button {
                    r#type: "button",
                    class: "flex items-center justify-center px-4 transition-transform duration-150 ease-out hover:scale-110 active:scale-90 md:hidden",
                    title: "Tampilkan/sembunyikan dock",
                    onclick: move |_| dock_open.set(!dock_open()),
                    svg { width: "26", height: "26", view_box: "0 0 31 31", fill: "none", xmlns: "http://www.w3.org/2000/svg",
                        path { d: "M1 14.25H7.5625M10.1687 10.5L3.90625 20.8125", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                        path { d: "M7.5625 3L10.375 7.6875M16.9375 8.625H4.84375", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                        path { d: "M22.5625 3L19.75 8.625M21.6438 13.5L16 3", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                        path { d: "M29.125 16.1625L22.5625 16.125M19.975 19.875L26.3125 9.5625", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                        path { d: "M23.5 27.375L18.8125 21.75M13.2438 21.7875L25.3938 21.75", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                        path { d: "M7.5625 27.375L10.375 21.75M8.55625 17.0625L14.125 27.375", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                    }
                }

                div { class: "relative flex flex-1 items-center justify-end",
                    button {
                        r#type: "button",
                        class: "flex items-center justify-end gap-2.5 self-stretch p-3 transition-colors duration-150 ease-out hover:bg-white/5 sm:p-5",
                        onclick: move |_| chat_open.set(!chat_open()),
                        span { class: "hidden text-lg font-['JetBrains_Mono'] sm:inline", "Live Chat!" }
                        svg {
                            class: "h-7 w-7 shrink-0 sm:h-10 sm:w-10",
                            width: "40", height: "40", view_box: "0 0 40 40", fill: "none", xmlns: "http://www.w3.org/2000/svg",
                            path { d: "M32.075 5H14.2367C13.0304 5 12.0525 5.97788 12.0525 7.18417V21.3217C12.0525 22.528 13.0304 23.5058 14.2367 23.5058H32.075C33.2813 23.5058 34.2592 22.528 34.2592 21.3217V7.18417C34.2592 5.97788 33.2813 5 32.075 5Z", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                            path { d: "M7.92499 11.3125H25.7633C26.0501 11.3125 26.3341 11.369 26.599 11.4788C26.864 11.5886 27.1047 11.7494 27.3075 11.9523C27.5102 12.1551 27.671 12.3959 27.7807 12.6609C27.8904 12.9259 27.9468 13.2099 27.9467 13.4967V27.6342C27.9467 28.2132 27.7166 28.7686 27.3072 29.178C26.8977 29.5875 26.3424 29.8175 25.7633 29.8175H12.1567L7.23166 34.7425C7.10936 34.8646 6.95363 34.9476 6.78415 34.9812C6.61466 35.0148 6.43903 34.9974 6.27941 34.9313C6.1198 34.8651 5.98338 34.7531 5.88737 34.6095C5.79136 34.4658 5.74007 34.297 5.73999 34.1242V13.4967C5.73988 13.2097 5.79633 12.9256 5.90611 12.6605C6.01588 12.3954 6.17684 12.1545 6.37977 11.9517C6.5827 11.7488 6.82362 11.588 7.08876 11.4783C7.3539 11.3686 7.63806 11.3123 7.92499 11.3125Z", stroke: "white", stroke_linecap: "round", stroke_linejoin: "round" }
                        }
                    }
                    ChatPopup { open: chat_open }
                }
            }
        }
    }
}

#[component]
fn VirtualKeyboardPanel() -> Element {
    const ROW1: &str = "1234567890";
    const ROW2: &str = "QWERTYUIOP";

    let key_class = "h-7 w-7 rounded border border-white/25 text-xs transition-all duration-150 ease-out hover:border-white/70 hover:bg-white/10 active:scale-90 font-['JetBrains_Mono']";

    rsx! {
        div { class: "flex flex-col items-center gap-1.5 border-t border-white/10 px-3 py-3",
            div { class: "flex flex-wrap justify-center gap-1",
                for ch in ROW1.chars() {
                    button { key: "r1-{ch}", r#type: "button", class: key_class, "{ch}" }
                }
            }
            div { class: "flex flex-wrap justify-center gap-1",
                for ch in ROW2.chars() {
                    button { key: "r2-{ch}", r#type: "button", class: key_class, "{ch}" }
                }
            }
            p { class: "text-[10px] text-white/30 font-['JetBrains_Mono']",
                "Keyboard virtual — dekoratif, belum terhubung ke input mana pun."
            }
        }
    }
}

#[component]
fn ChatPopup(open: Signal<bool>) -> Element {
    let popup_class = if open() {
        "pointer-events-auto absolute bottom-full right-0 z-20 mb-2 flex w-56 flex-col gap-2 rounded-md border border-white/25 bg-black p-3 opacity-100 transition-opacity duration-150 ease-out font-['JetBrains_Mono']"
    } else {
        "pointer-events-none absolute bottom-full right-0 z-20 mb-2 flex w-56 flex-col gap-2 rounded-md border border-white/25 bg-black p-3 opacity-0 transition-opacity duration-150 ease-out font-['JetBrains_Mono']"
    };

    rsx! {
        div { class: popup_class,
            p { class: "text-xs text-white/50", "Sapa aku lewat:" }
            a {
                href: "https://github.com/msalmanrafadhlih",
                target: "_blank",
                rel: "noopener noreferrer",
                class: "rounded border border-white/25 px-2 py-1.5 text-sm transition-colors duration-150 ease-out hover:border-white/70 hover:bg-white/10",
                "GitHub ↗",
            }
            p { class: "text-[10px] text-white/30", "Tautan Discord & LinkedIn menyusul." }
        }
    }
}
