//! Desktop entry point (wry/tao webview). All UI lives in `components`.

use components::{App as SharedApp, PlatformServices};
use dioxus::prelude::*;

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| PlatformServices { open_external });

    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        SharedApp {}
    }
}

/// External links open in the system browser; the webview stays on the app.
fn open_external(url: &str) {
    let _ = open::that(url);
}
