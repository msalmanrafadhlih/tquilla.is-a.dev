//! Constants shared across more than one module.
//!
//! These used to be redeclared locally in every file that needed them
//! (`FAVICON` in 3 places, the social links in 2). That meant updating a
//! URL required grepping the whole crate and hoping you caught every copy.
//! Centralizing them here means there's exactly one place to edit.

/// GitHub avatar used as the favicon on every route (`/`, `/profile`, `/deisktify`).
pub const FAVICON: &str = "https://avatars.githubusercontent.com/u/141149698";

pub const GITHUB_URL: &str = "https://github.com/msalmanrafadhlih";

// TODO(moch): swap in the real LinkedIn / Discord profile URLs — the
// Github one is confirmed from data/browser.json, the other two are
// placeholders until you hand them over.
pub const LINKEDIN_URL: &str = "https://www.linkedin.com/feed";
pub const DISCORD_URL: &str = "https://discord.com/invite/motionime";
