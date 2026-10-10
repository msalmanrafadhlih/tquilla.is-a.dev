mod clock;
mod gui;
mod js_util;
mod pages;

use crate::platform::TimeoutFuture;
use dioxus::prelude::*;

use crate::shared::PageMeta;
use pages::*;

#[component]
pub fn DesktopPage() -> Element {
    let mut is_booting = use_signal(|| true);
    let mut is_logged_in = use_signal(|| false);

    use_effect(move || {
        spawn(async move {
            TimeoutFuture::new(5000).await;
            is_booting.set(false);
        });
    });

    rsx! {
        PageMeta {
            title: "Deisktify",
            description: "Deisktify: a desktop operating system simulated in the browser, with a window manager, calculator, radio, ambient sounds, AI chat and live chat. Built with Rust and Dioxus.",
            path: "/deisktify",
        }

        if is_booting() {
            Booting { on_skip: move |_| is_booting.set(false) }
        } else if is_logged_in() {
            MainPage {}
        } else {
            Login { on_unlocked: move |_| is_logged_in.set(true) }
        }
    }
}
