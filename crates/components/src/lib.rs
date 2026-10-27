//! Shared UI for every platform crate (`web`, `desktop`, `mobile`).
//!
//! The platform crates (`platform/*`) only do three things:
//!   1. launch Dioxus,
//!   2. load their own generated Tailwind stylesheet,
//!   3. provide [`PlatformServices`] (how to open external links, ...),
//!
//! and then render [`App`].

pub mod api;
mod app;
mod guestbook;
mod home;
mod journal;
pub mod languages;
mod markdown_view;
mod nixdesk;
pub mod platform;
mod remote;
mod shared;
pub mod skeleton;
mod time;

pub use app::{App, Route};
pub use platform::PlatformServices;
