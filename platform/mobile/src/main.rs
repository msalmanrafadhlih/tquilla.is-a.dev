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

/// TODO(mobile): this navigates the webview itself, so the user leaves the
/// app until they press Back. Replace with an Android `Intent.ACTION_VIEW`
/// / iOS `openURL` call (JNI / objc) to open the system browser instead.
fn open_external(url: &str) {
    let url = url.replace('\\', "\\\\").replace('\'', "\\'");
    spawn(async move {
        document::eval(&format!("window.location.href = '{url}';"))
            .await
            .ok();
    });
}
