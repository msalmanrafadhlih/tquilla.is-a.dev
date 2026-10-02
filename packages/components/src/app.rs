use dioxus::prelude::*;

use crate::home::Home;
use crate::journal::JournalPage;
use crate::nixdesk::DesktopPage;

const FONTS_URL: &str = "https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600&family=Playfair+Display:ital,wght@0,400;0,700;1,400;1,700&family=JetBrains+Mono:wght@400&display=swap";
const MATERIAL_SYMBOLS_URL: &str = "https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap";
const HLS_JS_URL: &str = "https://cdn.jsdelivr.net/npm/hls.js@1/dist/hls.min.js";

// Plain CSS owned by this crate. Tailwind is *not* here: its output is
// generated per platform package (each one scans this crate's sources).
const BASE_CSS: Asset = asset!("/assets/base.css");
const GLOBALS_CSS: Asset = asset!("/assets/globals.css");
const STYLEGUIDE_CSS: Asset = asset!("/assets/styleguide.css");
const BOOTING_CSS: Asset = asset!("/assets/booting.css");
const LOGIN_CSS: Asset = asset!("/assets/login.css");

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/profile")]
    JournalPage {},
    #[route("/deisktify")]
    DesktopPage {},
}

#[component]
pub fn App() -> Element {
    rsx! {
        document::Link { rel: "preconnect", href: "https://fonts.googleapis.com" }
        document::Link { rel: "preconnect", href: "https://fonts.gstatic.com", crossorigin: "" }
        document::Stylesheet { href: FONTS_URL }
        document::Stylesheet { href: MATERIAL_SYMBOLS_URL }
        document::Stylesheet { href: BASE_CSS }
        document::Stylesheet { href: GLOBALS_CSS }
        document::Stylesheet { href: STYLEGUIDE_CSS }
        document::Stylesheet { href: BOOTING_CSS }
        document::Stylesheet { href: LOGIN_CSS }
        document::Script { src: HLS_JS_URL }

        Router::<Route> {}
    }
}
