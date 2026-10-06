mod clock;
mod pages;
mod gui;
mod js_util;

use dioxus::prelude::*;
use crate::platform::TimeoutFuture;

use pages::*;
use crate::shared::FAVICON;

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
        // Global app resources
        document::Link { rel: "icon", href: FAVICON }
        document::Title { "Deisktify" }

        if is_booting() {
            Booting {}
        } else if is_logged_in() {
            MainPage {}
        } else {
            Login { on_unlocked: move |_| is_logged_in.set(true) }
        }
    }
}
