//! Skeleton placeholders: grey shapes that stand where content will be while it
//! loads, so the layout does not jump when it arrives. Shared by every view
//! (old and new); nothing here knows about any one screen.
//!
//! - [`SkeletonGroup`] wraps a loading region. It is the only part a screen
//!   reader meets: one polite "loading" status, with the shapes inside hidden
//!   from it.
//! - [`Skeleton`] (any rectangle, sized with classes), [`SkeletonCircle`],
//!   [`SkeletonText`] (lines of text) and [`SkeletonRows`] / [`SkeletonEntries`]
//!   (a list of bars, a list of avatar + text entries) are the pieces.
//!
//! ```ignore
//! SkeletonGroup { label: s.radio.loading,
//!     SkeletonRows { rows: 5 }
//! }
//! ```
//!
//! The colour follows the surrounding text colour (so it works on the dark
//! desktop and on the paper-coloured journal) and the pulse stops for readers
//! who ask for reduced motion.

use dioxus::prelude::*;

use crate::languages::use_strings;

const SHAPE_CLASS: &str = "skeleton";

/// Widths for the lines of a text placeholder: ragged like real text, and the
/// last line shorter (a paragraph rarely ends at the margin). One line is full
/// width, since there is no "last" to shorten.
pub fn line_width_class(index: usize, lines: usize) -> &'static str {
    const RAGGED: [&str; 4] = [
        "skeleton__line--full",
        "skeleton__line--w11",
        "skeleton__line--full",
        "skeleton__line--w10",
    ];
    if lines > 1 && index + 1 == lines {
        "skeleton__line--last"
    } else {
        RAGGED[index % RAGGED.len()]
    }
}

/// A grey rectangle. Give it a size with a class from `skeleton.css`: `Skeleton { class: "skeleton__row" }`.
#[component]
pub fn Skeleton(#[props(default, into)] class: String) -> Element {
    rsx! {
        span { class: "{SHAPE_CLASS} {class}", "aria-hidden": "true" }
    }
}

/// A grey disc of `size` pixels (an avatar).
#[component]
pub fn SkeletonCircle(#[props(default = 32)] size: u32) -> Element {
    rsx! {
        span {
            class: "{SHAPE_CLASS} skeleton__circle",
            style: "width:{size}px;height:{size}px",
            "aria-hidden": "true",
        }
    }
}

/// `lines` lines of text, the last one shorter.
#[component]
pub fn SkeletonText(#[props(default = 3)] lines: usize) -> Element {
    rsx! {
        span { class: "skeleton__text", "aria-hidden": "true",
            for i in 0..lines {
                Skeleton { key: "{i}", class: "skeleton__text-line {line_width_class(i, lines)}" }
            }
        }
    }
}

/// `rows` full-width bars (a list of stations, bookmarks, sounds, ...).
#[component]
pub fn SkeletonRows(#[props(default = 5)] rows: usize) -> Element {
    rsx! {
        span { class: "skeleton__rows", "aria-hidden": "true",
            for i in 0..rows {
                Skeleton { key: "{i}", class: "skeleton__row" }
            }
        }
    }
}

/// `rows` entries of an avatar next to a name and two lines of text (the
/// guestbook, any feed). `avatar: false` drops the avatar.
#[component]
pub fn SkeletonEntries(
    #[props(default = 4)] rows: usize,
    #[props(default = true)] avatar: bool,
) -> Element {
    rsx! {
        span { class: "skeleton__entries", "aria-hidden": "true",
            for i in 0..rows {
                span { key: "{i}", class: "skeleton__entry",
                    if avatar {
                        SkeletonCircle { size: 32 }
                    }
                    span { class: "skeleton__entry-body",
                        Skeleton { class: "skeleton__entry-title" }
                        SkeletonText { lines: 2 }
                    }
                }
            }
        }
    }
}

/// The loading region: announces `label` (default: the generic "Loading…") once,
/// politely, and holds the shapes. `class` lays it out (padding, height, ...).
#[component]
pub fn SkeletonGroup(
    #[props(default)] label: Option<&'static str>,
    #[props(default, into)] class: String,
    children: Element,
) -> Element {
    let s = use_strings();
    let label = label.unwrap_or(s.remote.loading);
    rsx! {
        div {
            class: "skeleton__group {class}",
            role: "status",
            "aria-live": "polite",
            "aria-busy": "true",
            span { class: "visually-hidden", "{label}" }
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_last_line_is_shorter_when_there_are_several() {
        let widths: Vec<_> = (0..3).map(|i| line_width_class(i, 3)).collect();
        assert_eq!(
            widths,
            [
                "skeleton__line--full",
                "skeleton__line--w11",
                "skeleton__line--last"
            ]
        );
        assert_eq!(line_width_class(4, 5), "skeleton__line--last");
    }

    #[test]
    fn a_single_line_is_full_width_and_long_runs_do_not_panic() {
        assert_eq!(line_width_class(0, 1), "skeleton__line--full");
        for lines in 0..40 {
            for i in 0..lines {
                assert!(!line_width_class(i, lines).is_empty());
            }
        }
        // Only the last of several lines is short; the rest follow the pattern.
        assert_eq!(line_width_class(1, 10), "skeleton__line--w11");
        assert_eq!(line_width_class(3, 10), "skeleton__line--w10");
        assert_eq!(line_width_class(4, 10), "skeleton__line--full");
    }

    #[test]
    fn the_shape_class_honours_reduced_motion_and_the_text_colour() {
        // The look now lives in `assets/styles/skeleton.css` (`@apply`), so check the rule there.
        let css = include_str!("../assets/styles/skeleton.css");
        let rule = css
            .split(&format!(".{SHAPE_CLASS} {{"))
            .nth(1)
            .and_then(|r| r.split('}').next())
            .expect("skeleton shape rule");
        assert!(rule.contains("animate-pulse"));
        assert!(rule.contains("motion-reduce:animate-none"));
        assert!(rule.contains("currentColor"));
    }
}
