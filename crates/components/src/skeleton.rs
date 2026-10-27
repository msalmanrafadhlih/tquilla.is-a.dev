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
//! SkeletonGroup { label: s.radio.loading, class: "p-3",
//!     SkeletonRows { rows: 5 }
//! }
//! ```
//!
//! The colour follows the surrounding text colour (so it works on the dark
//! desktop and on the paper-coloured journal) and the pulse stops for readers
//! who ask for reduced motion.

use dioxus::prelude::*;

use crate::languages::use_strings;

const SHAPE_CLASS: &str = "block shrink-0 animate-pulse motion-reduce:animate-none bg-[color-mix(in_srgb,currentColor_14%,transparent)]";

/// Widths for the lines of a text placeholder: ragged like real text, and the
/// last line shorter (a paragraph rarely ends at the margin). One line is full
/// width, since there is no "last" to shorten.
pub fn line_width_class(index: usize, lines: usize) -> &'static str {
    const RAGGED: [&str; 4] = ["w-full", "w-11/12", "w-full", "w-10/12"];
    if lines > 1 && index + 1 == lines {
        "w-2/3"
    } else {
        RAGGED[index % RAGGED.len()]
    }
}

/// A grey rectangle. Give it a size with classes: `Skeleton { class: "h-4 w-32" }`.
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
            class: "{SHAPE_CLASS} rounded-full",
            style: "width:{size}px;height:{size}px",
            "aria-hidden": "true",
        }
    }
}

/// `lines` lines of text, the last one shorter.
#[component]
pub fn SkeletonText(#[props(default = 3)] lines: usize) -> Element {
    rsx! {
        span { class: "flex w-full flex-col gap-1.5", "aria-hidden": "true",
            for i in 0..lines {
                Skeleton { key: "{i}", class: "h-3 {line_width_class(i, lines)}" }
            }
        }
    }
}

/// `rows` full-width bars (a list of stations, bookmarks, sounds, ...).
#[component]
pub fn SkeletonRows(#[props(default = 5)] rows: usize) -> Element {
    rsx! {
        span { class: "flex w-full flex-col gap-2", "aria-hidden": "true",
            for i in 0..rows {
                Skeleton { key: "{i}", class: "h-8 w-full" }
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
        span { class: "flex w-full flex-col gap-4", "aria-hidden": "true",
            for i in 0..rows {
                span { key: "{i}", class: "flex w-full items-start gap-3",
                    if avatar {
                        SkeletonCircle { size: 32 }
                    }
                    span { class: "flex min-w-0 flex-1 flex-col gap-2",
                        Skeleton { class: "h-3 w-24" }
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
            class: "flex w-full flex-col gap-3 {class}",
            role: "status",
            "aria-live": "polite",
            "aria-busy": "true",
            span { class: "sr-only", "{label}" }
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
        assert_eq!(widths, ["w-full", "w-11/12", "w-2/3"]);
        assert_eq!(line_width_class(4, 5), "w-2/3");
    }

    #[test]
    fn a_single_line_is_full_width_and_long_runs_do_not_panic() {
        assert_eq!(line_width_class(0, 1), "w-full");
        for lines in 0..40 {
            for i in 0..lines {
                assert!(!line_width_class(i, lines).is_empty());
            }
        }
        // Only the last of several lines is short; the rest follow the pattern.
        assert_eq!(line_width_class(1, 10), "w-11/12");
        assert_eq!(line_width_class(3, 10), "w-10/12");
        assert_eq!(line_width_class(4, 10), "w-full");
    }

    #[test]
    fn the_shape_class_honours_reduced_motion_and_the_text_colour() {
        assert!(SHAPE_CLASS.contains("animate-pulse"));
        assert!(SHAPE_CLASS.contains("motion-reduce:animate-none"));
        assert!(SHAPE_CLASS.contains("currentColor"));
    }
}
