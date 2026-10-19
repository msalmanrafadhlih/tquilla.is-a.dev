use dioxus::prelude::*;

use crate::home::Home;
use crate::journal::JournalPage;
use crate::languages::use_language_provider;
use crate::nixdesk::DesktopPage;

// Hanya font yang dipakai di SEMUA route (boot menu, desktop). Font yang
// cuma dipakai satu fitur dimuat oleh fitur itu sendiri:
//   - Inter + Playfair Display + Material Symbols -> `journal` (`/profile`)
//   - Playfair Display (italic)                    -> kalkulator (keypad)
//   - hls.js                                       -> `AUDIO_JS` (saat stream .m3u8 diputar)
const FONTS_URL: &str =
    "https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400&display=swap";

// Plain CSS owned by this crate. Tailwind is *not* here: its output is
// generated per platform crate (each one scans this crate's sources).
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
    // Bahasa UI aktif, dibaca lewat `use_strings()` di semua route.
    use_language_provider();

    rsx! {
        document::Link { rel: "preconnect", href: "https://fonts.googleapis.com" }
        document::Link { rel: "preconnect", href: "https://fonts.gstatic.com", crossorigin: "" }
        document::Stylesheet { href: FONTS_URL }
        document::Stylesheet { href: BASE_CSS }
        document::Stylesheet { href: GLOBALS_CSS }
        document::Stylesheet { href: STYLEGUIDE_CSS }
        document::Stylesheet { href: BOOTING_CSS }
        document::Stylesheet { href: LOGIN_CSS }

        Router::<Route> {}
    }
}
