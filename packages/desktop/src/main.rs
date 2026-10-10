//! Desktop entry point (wry/tao webview). All UI lives in `components`.

use components::{App as SharedApp, PlatformServices};
use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| PlatformServices { open_external });

    rsx! {
        SharedApp {}
    }
}

/// External links open in the system browser; the webview stays on the app.
fn open_external(url: &str) {
    let _ = open::that(url);
}
