//! Shared UI for every platform package (`web`, `desktop`, `mobile`).
//!
//! The platform packages only do three things:
//!   1. launch Dioxus,
//!   2. load their own generated Tailwind stylesheet,
//!   3. provide [`PlatformServices`] (how to open external links, ...),
//!
//! and then render [`App`].

mod app;
mod home;
mod journal;
mod nixdesk;
pub mod platform;
mod shared;
mod time;

pub use app::{App, Route};
pub use platform::PlatformServices;
