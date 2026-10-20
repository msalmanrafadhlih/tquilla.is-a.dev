mod chart;
mod components;
mod data;
mod scroll;
mod stats_url;
mod util;

use dioxus::prelude::*;

use crate::languages::use_strings;
use crate::shared::{PageMeta, PLAYFAIR_FONT_URL};
use components::{ChronicleSection, EndOfStream, Footer, Hero, PinnedSection};
use data::AppData;

/// Fonts that only this route uses (kept out of `app.rs` so `/` and
/// `/deisktify` don't download them). Inter 600 was dropped: nothing here
/// uses `font-semibold`.
const INTER_FONT_URL: &str =
    "https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500&display=swap";
const MATERIAL_SYMBOLS_URL: &str = "https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap";

/// Bundled at compile time so the page always has something to render
/// immediately, and as a fallback if the live fetch below fails for any
/// reason (Worker cold-started wrong, network hiccup, rate-limited, etc.)
/// — the page should never show a broken/empty state.
const SAMPLE_DATA: &str = include_str!("../../data/journal.sample.json");

/// Address of the stats endpoint, compiled in from the `STATS_API_URL` build
/// variable with `option_env!` (not `env!`: a missing variable must not break
/// `cargo test`, `nix build` or CI, which do not set it). Public address, not
/// a secret. Unset or invalid means "not configured": the page says so
/// instead of guessing a host (see `stats_url::normalize`).
const STATS_API_URL: Option<&str> = option_env!("STATS_API_URL");

/// Where the numbers on the page came from. The page keeps rendering when
/// the live fetch fails, but it must say so: the bundled sample is dummy
/// data and must never be mistaken for real GitHub stats.
#[derive(Clone, Copy, PartialEq)]
enum DataSource {
    /// Fresh from the stats Worker.
    Live,
    /// Bundled sample because `STATS_API_URL` is unset or invalid in this
    /// build; no request was made.
    NotConfigured,
    /// Bundled `journal.sample.json` (dummy numbers), used because the live
    /// fetch failed.
    Snapshot,
}

#[derive(Clone, PartialEq)]
struct Loaded {
    data: AppData,
    source: DataSource,
}

fn sample(source: DataSource) -> Result<Loaded, String> {
    serde_json::from_str::<AppData>(SAMPLE_DATA)
        .map(|data| Loaded { data, source })
        .map_err(|e| e.to_string())
}

async fn load_data() -> Result<Loaded, String> {
    let Some(url) = stats_url::normalize(STATS_API_URL) else {
        return sample(DataSource::NotConfigured);
    };
    if let Ok(resp) = crate::platform::http::get(&url).await {
        if resp.ok() {
            if let Ok(data) = serde_json::from_str::<AppData>(&resp.body) {
                return Ok(Loaded {
                    data,
                    source: DataSource::Live,
                });
            }
        }
    }
    sample(DataSource::Snapshot)
}

#[component]
pub fn JournalPage() -> Element {
    let s = use_strings();
    let mut data: Signal<Option<Loaded>> = use_signal(|| None);
    let mut load_error: Signal<Option<String>> = use_signal(|| None);

    use_effect(move || {
        spawn(async move {
            match load_data().await {
                Ok(loaded) => data.set(Some(loaded)),
                Err(e) => load_error.set(Some(e)),
            }
        });
    });

    rsx! {
        // Di level halaman (bukan di `Page`) supaya font mulai diunduh
        // selagi data masih di-fetch, bukan setelah data tiba.
        document::Stylesheet { href: INTER_FONT_URL }
        document::Stylesheet { href: PLAYFAIR_FONT_URL }
        document::Stylesheet { href: MATERIAL_SYMBOLS_URL }

        if let Some(loaded) = data() {
            Page { data: loaded.data, source: loaded.source }
        } else if let Some(err) = load_error() {
            div { class: "min-h-screen flex items-center justify-center bg-paper text-primary font-mono text-sm",
                "{s.journal.load_failed(&err)}"
            }
        } else {
            div { class: "min-h-screen flex items-center justify-center bg-paper text-primary font-mono text-sm",
                "{s.journal.loading}"
            }
        }
    }
}

#[component]
fn Page(data: AppData, source: DataSource) -> Element {
    let s = use_strings();
    let today = util::today(s);
    let date_label = format!("{} {}", today.month_upper, today.day);
    let build_number = s.journal.build_number(today.day_of_year);

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
            if source != DataSource::Live {
                div {
                    role: "status",
                    class: "w-full bg-accent text-paper font-mono text-xs text-center px-4 py-2",
                    if source == DataSource::NotConfigured {
                        "{s.journal.stats_not_configured}"
                    } else {
                        "{s.journal.offline_snapshot}"
                    }
                }
            }
            Hero { hero: data.hero.clone(), date_label, build_number }
            PinnedSection { pinned: data.pinned.clone() }
            ChronicleSection { chronicle: data.chronicle.clone() }
            EndOfStream {}
            Footer { year: today.year }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_sample_matches_the_live_api_shape() {
        // If AppData changes, the offline-snapshot fallback must still parse.
        let data: AppData = serde_json::from_str(SAMPLE_DATA)
            .expect("journal.sample.json no longer matches AppData");
        assert!(!data.pinned.is_empty());
    }
}
