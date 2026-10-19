//! Scroll-triggered reveal animations.
//!
//! Any element marked `data-reveal` in `components.rs` fades/slides into
//! place the first time it scrolls into the viewport. Implemented with a
//! native `IntersectionObserver`, driven through `document::eval` so the
//! exact same code runs on web, desktop and mobile (every one of them
//! renders in a browser engine). The actual transition (opacity/transform)
//! is plain CSS in `assets/base.css`; this module only flips the
//! `is-visible` class.

use dioxus::prelude::*;

const REVEAL_JS: &str = include_str!("../../js/scroll_reveal.js");

/// Finds every `[data-reveal]` element currently in the DOM and observes it.
/// The first time one crosses into the viewport, `is-visible` is added to
/// its class list (which triggers the CSS transition) and that element is
/// then unobserved — a one-shot reveal, not a repeat-on-every-scroll effect.
///
/// Call once after the real data has rendered (see `Page`'s `use_effect` in
/// `mod.rs`). Elements that don't exist yet at call time are simply never
/// observed, so this is safe to call exactly once per full page render.
pub fn init_scroll_reveal() {
    spawn(async move {
        document::eval(REVEAL_JS).await.ok();
    });
}
