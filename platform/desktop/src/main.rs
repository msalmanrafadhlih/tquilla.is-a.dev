//! Desktop entry point (wry/tao webview). All UI lives in `components`.

use components::{App as SharedApp, PlatformServices};
use dioxus::prelude::*;

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    // The UI is a full desktop of its own (top bar, dock, windows), so the
    // toolkit's default menu bar (Window / Edit / Help) only duplicates it
    // and eats a row of the window. `None` removes it on every OS.
    let config = dioxus::desktop::Config::new().with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(App);
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
