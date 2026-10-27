mod chart;
mod components;
mod scroll;
mod util;

use api_types::journal::{Snapshot, Source};
use dioxus::prelude::*;

use crate::languages::use_strings;
use crate::shared::{PageMeta, PLAYFAIR_FONT_URL};
use crate::skeleton::{Skeleton, SkeletonGroup, SkeletonText};
use components::{ChronicleSection, EndOfStream, Footer, Hero, PinnedSection};

/// Fonts that only this route uses (kept out of `app.rs` so `/` and
/// `/deisktify` don't download them). Inter 600 was dropped: nothing here
/// uses `font-semibold`.
const INTER_FONT_URL: &str =
    "https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500&display=swap";
const MATERIAL_SYMBOLS_URL: &str = "https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap";

/// The page reads the latest snapshot that the API's Cron Trigger stored
/// (`GET /api/v1/journal`). There is no bundled fallback: when the API is
/// unreachable, or no snapshot exists yet, the page says so instead of showing
/// numbers that look real. A `seed` snapshot (local development only) is
/// flagged with a banner.
#[component]
pub fn JournalPage() -> Element {
    let s = use_strings();
    let mut data: Signal<Option<Snapshot>> = use_signal(|| None);
    let mut load_error: Signal<Option<String>> = use_signal(|| None);

    use_effect(move || {
        spawn(async move {
            match crate::api::journal().await {
                Ok(snapshot) => data.set(Some(snapshot)),
                Err(e) => load_error.set(Some(e.to_string())),
            }
        });
    });

    rsx! {
        // Di level halaman (bukan di `Page`) supaya font mulai diunduh
        // selagi data masih di-fetch, bukan setelah data tiba.
        document::Stylesheet { href: INTER_FONT_URL }
        document::Stylesheet { href: PLAYFAIR_FONT_URL }
        document::Stylesheet { href: MATERIAL_SYMBOLS_URL }

        if let Some(snapshot) = data() {
            Page { snapshot }
        } else if let Some(err) = load_error() {
            div { class: "min-h-screen flex items-center justify-center bg-paper text-primary font-mono text-sm",
                "{s.journal.load_failed(&err)}"
            }
        } else {
            div { class: "min-h-screen bg-paper text-primary",
                SkeletonGroup { label: s.journal.loading, class: "mx-auto max-w-3xl p-8",
                    Skeleton { class: "h-10 w-2/3" }
                    SkeletonText { lines: 3 }
                    Skeleton { class: "h-40 w-full" }
                    SkeletonText { lines: 4 }
                }
            }
        }
    }
}

#[component]
fn Page(snapshot: Snapshot) -> Element {
    let s = use_strings();
    let today = util::today(s);
    let date_label = format!("{} {}", today.month_upper, today.day);
    let build_number = s.journal.build_number(today.day_of_year);
    let updated = s
        .journal
        .updated(&crate::time::relative_time(&snapshot.fetched_at, s));
    let data = snapshot.data;

    // Runs once, right after this page's real DOM (all `[data-reveal]`
    // sections) is mounted — see `scroll::init_scroll_reveal` for how the
    // actual observing/animating works.
    use_effect(move || {
        scroll::init_scroll_reveal();
    });

    rsx! {
        PageMeta {
            title: s.meta.journal_title,
            description: s.meta.journal_description,
            path: "/profile",
        }
        div { class: "bg-paper text-primary font-sans antialiased flex flex-col selection:bg-accent selection:text-paper overflow-x-hidden",
            if snapshot.source == Source::Seed {
                div {
                    role: "status",
                    class: "w-full bg-accent text-paper font-mono text-xs text-center px-4 py-2",
                    "{s.journal.sample_data}"
                }
            }
            Hero { hero: data.hero, date_label, build_number, updated }
            PinnedSection { pinned: data.pinned.clone() }
            ChronicleSection { chronicle: data.chronicle.clone(), fetched_at: snapshot.fetched_at.clone() }
            EndOfStream {}
            Footer { year: today.year }
        }
    }
}
