use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;

const CSS: Asset = asset!("/assets/login.css");

const TYPE_DELAY_MS: u32 = 150;
const DEMO_PASSWORD: &str = "msalmanrafadhlih";

#[derive(Clone, Copy, PartialEq)]
enum TypingMode {
    Auto,
    Manual,
}

#[component]
pub fn Login(on_unlocked: EventHandler<()>) -> Element {
    let mut value = use_signal(String::new);
    let mut mode = use_signal(|| TypingMode::Auto);
    let mut focused = use_signal(|| false);
    let unlocking = use_signal(|| false);
    let shake = use_signal(|| false);

    // Auto-type the password once, on mount. Bails out the moment the
    // visitor takes over (mode flips to Manual) so it never fights a real
    // keystroke.
    use_effect(move || {
        spawn(async move {
            let mut typed = String::new();
            for ch in DEMO_PASSWORD.chars() {
                TimeoutFuture::new(TYPE_DELAY_MS).await;
                if mode() != TypingMode::Auto {
                    return;
                }
                typed.push(ch);
                value.set(typed.clone());
            }
            TimeoutFuture::new(300).await;
            if mode() == TypingMode::Auto {
                submit(value, unlocking, shake, on_unlocked).await;
            }
        });
    });

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }

        main {
            class: "relative min-h-screen w-full bg-black text-white font-mono antialiased overflow-hidden selection:bg-green-500/30",

            section { class: "relative min-h-screen w-full flex flex-col items-center justify-center gap-6 sm:gap-7 px-6 py-16",

                // Login view
                div {
                    class: "flex flex-col items-center gap-6 sm:gap-7 transition-opacity duration-500 ease-out",
                    style: if unlocking() { "opacity: 0; pointer-events: none;" } else { "opacity: 1;" },

                    img {
                        src: "https://avatars.githubusercontent.com/u/141149698?v=4",
                        alt: "Profile",
                        class: "w-20 h-20 sm:w-24 sm:h-24 outline outline-2 outline-white/90",
                    }

                    div { class: "flex flex-col items-center gap-1.5 sm:gap-2",
                        p { class: "text-zinc-300 text-base sm:text-lg tracking-[0.2em]", "Tquilla" }
                        p { class: "text-white/40 text-xs sm:text-sm font-extralight", "Keyboard: EN" }
                    }

                    form {
                        class: "flex flex-col items-center gap-2",
                        onsubmit: move |evt: FormEvent| {
                            evt.prevent_default();
                            if unlocking() {
                                return;
                            }
                            if mode() == TypingMode::Auto {
                                // First Unlock press mid-animation just stops the
                                // demo — it does not count as a real attempt.
                                mode.set(TypingMode::Manual);
                                return;
                            }
                            spawn(submit(value, unlocking, shake, on_unlocked));
                        },

                        div {
                            class: if shake() {
                                "flex flex-col sm:flex-row items-center gap-2 sm:gap-5 login-shake"
                            } else {
                                "flex flex-col sm:flex-row items-center gap-2 sm:gap-5"
                            },

                            if shake() {
                                button {
                                    r#type: "submit",
                                    class: "text-red-500 text-base sm:text-lg font-medium tracking-widest whitespace-nowrap rounded-sm focus:outline-none focus-visible:ring-2 focus-visible:ring-red-500/70 focus-visible:ring-offset-2 focus-visible:ring-offset-black",
                                    "[ Wrong ]"
                                }
                            } else {
                                button {
                                    r#type: "submit",
                                    class: "text-green-500 text-base sm:text-lg font-medium tracking-widest whitespace-nowrap rounded-sm focus:outline-none focus-visible:ring-2 focus-visible:ring-green-500/70 focus-visible:ring-offset-2 focus-visible:ring-offset-black",
                                    "[ Unlock ]"
                                }
                            }

                            div { class: "relative w-[100px] sm:w-[110px] h-10",
                                label { class: "sr-only", r#for: "password-input", "Password" }
                                input {
                                    id: "password-input",
                                    r#type: "password",
                                    autocomplete: "current-password",
                                    class: "peer absolute inset-0 z-10 w-full h-full opacity-0 text-base text-center cursor-text",
                                    value: "{value}",
                                    disabled: unlocking(),
                                    onfocus: move |_| {
                                        focused.set(true);
                                        // Clicking/tabbing into the field is a
                                        // "distraction" too — stop the demo.
                                        if mode() == TypingMode::Auto {
                                            mode.set(TypingMode::Manual);
                                        }
                                    },
                                    onblur: move |_| focused.set(false),
                                    oninput: move |evt: FormEvent| {
                                        mode.set(TypingMode::Manual);
                                        value.set(evt.value());
                                    },
                                }
                                div {
                                    class: "absolute inset-0 flex items-center overflow-hidden rounded-sm transition-shadow duration-150 peer-focus-visible:ring-2 peer-focus-visible:ring-green-500/60 peer-focus-visible:ring-offset-2 peer-focus-visible:ring-offset-black",
                                    style: "mask-image:linear-gradient(to right, transparent, black, transparent); -webkit-mask-image:linear-gradient(to right, transparent, black, transparent);",

                                    div { class: "flex items-center justify-center w-full overflow-x-hidden whitespace-nowrap",
                                        for (i , _ch) in value().chars().enumerate() {
                                            span {
                                                key: "{i}",
                                                class: "login-char item-center inline-block text-white text-lg tracking-[3px]",
                                                "*"
                                            }
                                        }
                                        span {
                                            class: if focused() && value().is_empty() { "login-caret inline-block w-[2px] h-5 bg-green-500 align-middle" } else { "login-caret invisible inline-block w-[2px] h-5 bg-green-500 align-middle" },
                                        }
                                    }
                                }
                            }
                        }

                        p {
                            class: "text-white/30 text-[11px] text-center sm:text-xs h-4 transition-opacity duration-200",
                            if mode() == TypingMode::Auto {
                                "logging in automatically…"
                            } else {
                                "press enter, or click [ Unlock ]"
                            }
                        }
                    }

                    div { class: "flex flex-col items-center gap-1 mt-1",
                        p { class: "text-white text-center text-[14px] sm:text-lg font-extralight", "Thu, 03 Sep 2026" }
                        p { class: "text-white text-[11px] sm:text-sm font-extralight", "[ 19:35 ]" }
                    }
                }

                div {
                    class: "absolute inset-0 flex flex-col items-center justify-center gap-4 transition-opacity duration-500 ease-out",
                    style: if unlocking() { "opacity: 1;" } else { "opacity: 0; pointer-events: none;" },

                    div { class: "w-16 h-16 rounded-full border-2 border-green-500 flex items-center justify-center",
                        svg {
                            width: "28",
                            height: "28",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2.5",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            class: "text-green-500",
                            polyline { points: "20 6 9 17 4 12" }
                        }
                    }

                    p { class: "text-white text-base text-center sm:text-lg tracking-wide", "Welcome back, Tquilla" }
                    p { class: "text-white/40 text-xs text-center sm:text-sm font-extralight", "access granted" }
                }
            }

            // status bar
            div { class: "fixed bottom-4 left-4 sm:bottom-6 sm:left-6 flex items-center gap-2 opacity-50",
                svg {
                    width: "22",
                    height: "14",
                    view_box: "0 0 22 14",
                    fill: "none",
                    xmlns: "http://www.w3.org/2000/svg",
                    rect { x: "0.75", y: "0.75", width: "18.5", height: "12.5", rx: "2", stroke: "white", stroke_width: "1.5" }
                    rect { x: "20.5", y: "4.5", width: "1.5", height: "5", rx: "0.75", fill: "white" }
                    rect { x: "2.5", y: "2.5", width: "15", height: "8.5", rx: "1", fill: "white" }
                }
                span { class: "text-white text-sm font-normal", "100%" }
            }
        }
    }
}

/// Validates the current input against [`DEMO_PASSWORD`]. On success, plays
/// the "access granted" beat and hands control back to the parent via
/// `on_unlocked`. On failure, shakes the row and clears the field so a
/// wrong guess doesn't linger.
async fn submit(
    value: Signal<String>,
    mut unlocking: Signal<bool>,
    mut shake: Signal<bool>,
    on_unlocked: EventHandler<()>,
) {
    if value() == DEMO_PASSWORD {
        unlocking.set(true);
        TimeoutFuture::new(2000).await;
        on_unlocked.call(());
    } else {
        shake.set(true);
        TimeoutFuture::new(400).await;
        shake.set(false);
        // value.set(String::new());
    }
}
