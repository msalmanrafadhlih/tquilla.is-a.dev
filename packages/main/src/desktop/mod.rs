mod pages;

use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;

use pages::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

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
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Title { "Deisktify" }

        // Login { on_unlocked: move |_| is_logged_in.set(true) }

        if is_booting() {
            Booting {}
        } else if is_logged_in() {
            MainPage {}
        } else {
            Login { on_unlocked: move |_| is_logged_in.set(true) }
        }
    }
}
