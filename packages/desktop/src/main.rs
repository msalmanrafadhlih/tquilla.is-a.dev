use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;

use ui::Booting;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut is_booting = use_signal(|| true);

    use_effect(move || {
        spawn(async move {
            TimeoutFuture::new(5500).await;
            is_booting.set(false);
        });
    });

    rsx! {
        // Global app resources
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        if is_booting() {
            Booting {}
        } else {
            p { "Last login: Thu Jul 31 10:30:42 2025 from 192.168.1.100" }
            p { "Welcome to Lv Linux!" }
        }

    }
}
