//! Mobile entry point (Android / iOS webview). All UI lives in `components`.

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

/// External links open in the system browser (Android: `Intent.ACTION_VIEW`,
/// iOS: `openURL`), so the app stays where it is and Back returns to it.
/// Callers only pass `http(s)` URLs (`home::link_target` filters them).
fn open_external(url: &str) {
    if let Err(err) = webbrowser::open(url) {
        dioxus::logger::tracing::warn!("could not open the system browser: {err}");
    }
}
