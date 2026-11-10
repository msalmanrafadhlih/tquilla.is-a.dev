use crate::languages::use_strings;
use crate::platform::TimeoutFuture;
use crate::shared::{AVATAR, USER_NAME};
use dioxus::prelude::*;

const TYPE_DELAY_MS: u32 = 150;
const DEMO_PASSWORD: &str = "msalmanrafadhlih";

#[derive(Clone, Copy, PartialEq)]
enum TypingMode {
    Auto,
    Manual,
}

#[component]
pub fn Login(on_unlocked: EventHandler<()>) -> Element {
    let s = use_strings();
    let mut loaded = use_signal(|| false);
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
        main {
            class: "login__root",
            class: if loaded() { "login__fade--in" } else { "login__fade--out" },
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

            section { class: "login__screen",

                // Login view
                div {
                    class: if unlocking() { "login__identity login__identity--out" } else { "login__identity" },

                    img {
                        src: AVATAR,
                        alt: s.login.avatar_alt,
                        class: "login__avatar",
                    }

                    div { class: "login__names",
                        p { class: "login__name", "{USER_NAME}" }
                        p { class: "login__hint", "{s.login.keyboard}" }
                    }

                    form {
                        class: "login__form",
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
                                "login__row login__row--shake"
                            } else {
                                "login__row"
                            },

                            if shake() {
                                button {
                                    r#type: "submit",
                                    class: "login__btn login__btn--cancel",
                                    "{s.login.wrong}"
                                }
                            } else {
                                button {
                                    r#type: "submit",
                                    class: "login__btn login__btn--ok",
                                    "{s.login.unlock}"
                                }
                            }

                            div { class: "login__field",
                                label { class: "visually-hidden", r#for: "password-input", "{s.login.password_label}" }
                                input {
                                    id: "password-input",
                                    r#type: "password",
                                    autocomplete: "current-password",
                                    class: "peer login__input",
                                    disabled: unlocking(),
                                    onfocus: move |_| {
                                        focused.set(true);
                                        // Clicking/tabbing into the field is a
                                        // "distraction" too — stop the demo.
                                        // The demo only drew dots (the field is
                                        // not bound to `value`), so the visitor
                                        // starts from an empty password.
                                        if mode() == TypingMode::Auto {
                                            mode.set(TypingMode::Manual);
                                            value.set(String::new());
                                        }
                                    },
                                    onblur: move |_| focused.set(false),
                                    oninput: move |evt: FormEvent| {
                                        mode.set(TypingMode::Manual);
                                        value.set(evt.value());
                                    },
                                }
                                div {
                                    class: "login__mask",

                                    div { class: "login__chars",
                                        for (i , _ch) in value().chars().enumerate() {
                                            span {
                                                key: "{i}",
                                                class: "login__char",
                                                "*"
                                            }
                                        }
                                        span {
                                            class: if focused() && value().is_empty() { "login__caret" } else { "login__caret login__caret--hidden" },
                                        }
                                    }
                                }
                            }
                        }

                        p {
                            class: "login__error",
                            if mode() == TypingMode::Auto {
                                "{s.login.auto_login}"
                            } else {
                                "{s.login.press_enter}"
                            }
                        }
                    }

                    div { class: "login__welcome",
                        p { class: "login__welcome-title", "{s.login.lock_date}" }
                        p { class: "login__welcome-sub", "[ 19:35 ]" }
                    }
                }

                div {
                    class: if unlocking() { "login__unlock login__unlock--on" } else { "login__unlock" },

                    div { class: "login__unlock-ring",
                        svg {
                            width: "28",
                            height: "28",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2.5",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            class: "login__unlock-icon",
                            polyline { points: "20 6 9 17 4 12" }
                        }
                    }

                    p { class: "login__unlock-title", "{s.login.welcome_back(USER_NAME)}" }
                    p { class: "login__unlock-sub", "{s.login.access_granted}" }
                }
            }

            // status bar
            div { class: "login__footer",
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
                span { class: "login__footer-text", "100%" }
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
