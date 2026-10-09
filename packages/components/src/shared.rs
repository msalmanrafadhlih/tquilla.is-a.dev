//! Constants shared across more than one module.
//!
//! These used to be redeclared locally in every file that needed them
//! (`FAVICON` in 3 places, the social links in 2). That meant updating a
//! URL required grepping the whole crate and hoping you caught every copy.
//! Centralizing them here means there's exactly one place to edit.

use dioxus::prelude::*;

/// Foto profil lokal (`assets/avatar.png`), dipakai sebagai avatar login,
/// avatar `/profile`, dan favicon. Sebelumnya diambil dari
/// `avatars.githubusercontent.com` / `github.com/<user>.png` saat runtime:
/// satu request pihak ketiga tambahan per halaman, dan gambar rusak kalau
/// GitHub lambat atau diblokir.
///
/// File yang ada di repo hanya placeholder. Timpa dengan foto aslimu:
///   curl -L https://avatars.githubusercontent.com/u/141149698 \
///        -o packages/components/assets/avatar.png
pub const AVATAR: Asset = asset!("/assets/avatar.png");

/// Favicon di setiap route (`/`, `/profile`, `/deisktify`).
pub const FAVICON: Asset = AVATAR;

/// Google Fonts yang hanya dibutuhkan fitur tertentu (font global ada di
/// `app.rs`). Dimuat oleh komponen yang memakainya, bukan oleh setiap route.
pub const PLAYFAIR_FONT_URL: &str = "https://fonts.googleapis.com/css2?family=Playfair+Display:ital,wght@0,400;0,700;1,400;1,700&display=swap";

pub const GITHUB_URL: &str = "https://github.com/msalmanrafadhlih";

// TODO(moch): swap in the real LinkedIn / Discord profile URLs — the
// Github one is confirmed from data/browser.json, the other two are
// placeholders until you hand them over.
pub const LINKEDIN_URL: &str = "https://www.linkedin.com/feed";
pub const DISCORD_URL: &str = "https://discord.com/invite/motionime";
