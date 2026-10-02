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

const REVEAL_JS: &str = r#"
const elements = document.querySelectorAll('[data-reveal]');
if (elements.length === 0) { return; }
// Trigger a little before the element's bottom edge fully reaches the
// viewport's bottom edge, and once ~10% of it is visible.
const observer = new IntersectionObserver((entries, obs) => {
  for (const entry of entries) {
    if (entry.isIntersecting) {
      entry.target.classList.add('is-visible');
      obs.unobserve(entry.target);
    }
  }
}, { rootMargin: '0px 0px -10% 0px', threshold: 0.1 });
elements.forEach((el) => observer.observe(el));
"#;

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
