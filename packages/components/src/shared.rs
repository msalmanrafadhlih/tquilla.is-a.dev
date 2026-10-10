//! Constants and tiny components shared across more than one module.
//!
//! These used to be redeclared locally in every file that needed them
//! (`FAVICON` in 3 places, the social links in 2). That meant updating a
//! URL required grepping the whole crate and hoping you caught every copy.
//! Centralizing them here means there's exactly one place to edit.

use dioxus::prelude::*;

/// Foto profil lokal (`assets/avatar.jpg`), dipakai sebagai avatar login,
/// avatar `/profile`, favicon, dan gambar preview saat link dibagikan.
/// Sebelumnya diambil dari `avatars.githubusercontent.com` /
/// `github.com/<user>.png` saat runtime: satu request pihak ketiga tambahan
/// per halaman, dan gambar rusak kalau GitHub lambat atau diblokir.
pub const AVATAR: Asset = asset!("/assets/avatar.jpg");

/// Favicon di setiap route (`/`, `/profile`, `/deisktify`).
pub const FAVICON: Asset = AVATAR;

/// Google Fonts yang hanya dibutuhkan fitur tertentu (font global ada di
/// `app.rs`). Dimuat oleh komponen yang memakainya, bukan oleh setiap route.
pub const PLAYFAIR_FONT_URL: &str = "https://fonts.googleapis.com/css2?family=Playfair+Display:ital,wght@0,400;0,700;1,400;1,700&display=swap";

pub const SITE_URL: &str = "https://tquilla.is-a.dev";
pub const SITE_NAME: &str = "tquilla.is-a.dev";

pub const GITHUB_URL: &str = "https://github.com/msalmanrafadhlih";
pub const LINKEDIN_URL: &str = "https://linkedin.com/in/msalmanrafadhlih";
// Sengaja: ini invite server komunitas "Motion IME" (sama seperti di
// data/browser.json), bukan profil Discord pribadi. Jangan diganti.
pub const DISCORD_URL: &str = "https://discord.com/invite/motionime";

/// Judul tab + metadata satu route, dirender ke `<head>` lewat
/// `document::*`. Semua route memakai ini supaya format judulnya seragam
/// ("<Halaman> | tquilla.is-a.dev") dan tag Open Graph tidak terlewat.
///
/// Catatan: crawler preview link (WhatsApp, Discord, Facebook) tidak
/// menjalankan JS/WASM, jadi mereka hanya membaca tag statis yang
/// disuntikkan `scripts/postbuild.py` ke `index.html` saat build. Tag di sini
/// melengkapi untuk browser/crawler yang menjalankan JS dan saat navigasi
/// antar-route di dalam aplikasi.
#[component]
pub fn PageMeta(title: String, description: String, path: String) -> Element {
    let full_title = format!("{title} | {SITE_NAME}");
    let url = format!("{SITE_URL}{path}");
    let image = format!("{SITE_URL}/og-image.jpg");

    rsx! {
        document::Title { "{full_title}" }
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "canonical", href: "{url}" }
        document::Meta { name: "description", content: "{description}" }
        document::Meta { property: "og:type", content: "website" }
        document::Meta { property: "og:site_name", content: SITE_NAME }
        document::Meta { property: "og:title", content: "{full_title}" }
        document::Meta { property: "og:description", content: "{description}" }
        document::Meta { property: "og:url", content: "{url}" }
        document::Meta { property: "og:image", content: "{image}" }
        document::Meta { name: "twitter:card", content: "summary_large_image" }
        document::Meta { name: "twitter:image", content: "{image}" }
    }
}
