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
            class: "remote__loading",
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
            class: "remote__failed",
            role: "alert",
            p { class: "remote__failed-title", "{headline}" }
            p { "{reason}" }
            if can_retry {
                button {
                    r#type: "button",
                    class: "remote__retry",
                    onclick: move |_| on_retry.call(()),
                    "{s.remote.retry}"
                }
            }
            {children}
        }
    }
}
