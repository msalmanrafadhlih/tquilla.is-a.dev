//! Two tiny states shared by every view that loads data from the API:
//! "still loading" and "failed, try again".
//!
//! The text of an error comes from [`ApiError::message`], which never contains
//! URLs or response bodies, so it is safe to put on screen.

use dioxus::prelude::*;

use crate::api::ApiError;
use crate::languages::use_strings;

/// "Loading…" placeholder. `label` is announced to screen readers; pass a
/// more specific text than the default where one exists.
#[component]
pub fn Loading(#[props(default)] label: Option<&'static str>) -> Element {
    let s = use_strings();
    let label = label.unwrap_or(s.remote.loading);
    rsx! {
        div {
            class: "flex h-full w-full items-center justify-center p-4 text-center text-xs text-[var(--fg-secondary,#808080)] font-mono",
            role: "status",
            "aria-live": "polite",
            "{label}"
        }
    }
}

/// Error box with a "Try again" button (only when trying again can help:
/// network problem, 5xx, 429; a missing `API_BASE_URL` will not fix itself).
///
/// `headline` is what failed; the reason is [`ApiError::message`].
#[component]
pub fn LoadFailed(
    error: ApiError,
    #[props(default)] headline: Option<&'static str>,
    on_retry: EventHandler<()>,
    #[props(default)] children: Element,
) -> Element {
    let s = use_strings();
    let headline = headline.unwrap_or(s.remote.failed);
    let reason = error.message(s);
    let can_retry = error.is_retryable();
    rsx! {
        div {
            class: "flex h-full w-full flex-col items-center justify-center gap-3 p-4 text-center text-xs text-[var(--fg-secondary,#808080)] font-mono",
            role: "alert",
            p { class: "text-[var(--fg-main,#fff)]", "{headline}" }
            p { "{reason}" }
            if can_retry {
                button {
                    r#type: "button",
                    class: "appearance-none px-3 py-1 border border-solid border-[var(--fg-secondary,#808080)] bg-[var(--bg-secondary,#000)] text-[var(--fg-main,#fff)] cursor-pointer hover:bg-[var(--fg-main,#fff)] hover:text-[var(--bg-secondary,#000)]",
                    onclick: move |_| on_retry.call(()),
                    "{s.remote.retry}"
                }
            }
            {children}
        }
    }
}
