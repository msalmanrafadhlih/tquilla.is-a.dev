mod clock;
mod gui;
mod js_util;
mod pages;

use crate::platform::TimeoutFuture;
use dioxus::prelude::*;

use crate::languages::use_strings;
use crate::shared::PageMeta;
use pages::*;

#[component]
pub fn DesktopPage() -> Element {
    let s = use_strings();
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
            title: s.meta.desktop_title,
            description: s.meta.desktop_description,
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
