mod chart;
mod components;
mod data;
mod scroll;
mod util;

use dioxus::prelude::*;

use components::{ChronicleSection, EndOfStream, Footer, Hero, PinnedSection};
use data::AppData;
use crate::shared::{PageMeta, PLAYFAIR_FONT_URL};

/// Fonts that only this route uses (kept out of `app.rs` so `/` and
/// `/deisktify` don't download them). Inter 600 was dropped: nothing here
/// uses `font-semibold`.
const INTER_FONT_URL: &str = "https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500&display=swap";
const MATERIAL_SYMBOLS_URL: &str = "https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap";

/// Bundled at compile time so the page always has something to render
/// immediately, and as a fallback if the live fetch below fails for any
/// reason (Worker cold-started wrong, network hiccup, rate-limited, etc.)
/// — the page should never show a broken/empty state.
const SAMPLE_DATA: &str = include_str!("../../data/journal.sample.json");

/// The Cloudflare Worker that now does what the old `fetch-stats` CI job +
/// `data.json` used to: run the GitHub GraphQL query and hand back JSON in
/// this exact shape, cached at the edge for a few minutes. See
/// `worker/README.md` for setup/deploy instructions.
///
/// This has to be a full absolute URL (unlike the old relative
/// `data.json` path) since it points at a different origin than the
/// GitHub Pages site itself. Update this after your first
/// `wrangler deploy` in `worker/`.
const STATS_API_URL: &str = "https://github-journal-stats.msalmanrafadhlih.workers.dev/api/stats";

/// Where the numbers on the page came from. The page keeps rendering when
/// the live fetch fails, but it must say so: the bundled sample is dummy
/// data and must never be mistaken for real GitHub stats.
#[derive(Clone, Copy, PartialEq)]
enum DataSource {
    /// Fresh from the stats Worker.
    Live,
    /// Bundled `journal.sample.json` (dummy numbers), used because the live
    /// fetch failed.
    Snapshot,
}

#[derive(Clone, PartialEq)]
struct Loaded {
    data: AppData,
    source: DataSource,
}

async fn load_data() -> Result<Loaded, String> {
    if let Ok(resp) = crate::platform::http::get(STATS_API_URL).await {
        if resp.ok() {
            if let Ok(data) = serde_json::from_str::<AppData>(&resp.body) {
                return Ok(Loaded { data, source: DataSource::Live });
            }
        }
    }
    serde_json::from_str::<AppData>(SAMPLE_DATA)
        .map(|data| Loaded { data, source: DataSource::Snapshot })
        .map_err(|e| e.to_string())
}

#[component]
pub fn JournalPage() -> Element {
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
                "Failed to load data: {err}"
            }
        } else {
            div { class: "min-h-screen flex items-center justify-center bg-paper text-primary font-mono text-sm",
                "Loading…"
            }
        }
    }
}

#[component]
fn Page(data: AppData, source: DataSource) -> Element {
    let today = util::today();
    let date_label = format!("{} {}", today.month_upper, today.day);
    let build_number = format!("NO. {}", today.day_of_year);

    // Runs once, right after this page's real DOM (all `[data-reveal]`
    // sections) is mounted — see `scroll::init_scroll_reveal` for how the
    // actual observing/animating works.
    use_effect(move || {
        scroll::init_scroll_reveal();
    });

    rsx! {
        PageMeta {
            title: "GitHub Journal",
            description: "A journal of my GitHub activity: pinned repositories, contribution volume, streaks, languages and stack.",
            path: "/profile",
        }
        div { class: "bg-paper text-primary font-sans antialiased flex flex-col selection:bg-accent selection:text-paper overflow-x-hidden",
            if source == DataSource::Snapshot {
                div {
                    role: "status",
                    class: "w-full bg-accent text-paper font-mono text-xs text-center px-4 py-2",
                    "Offline snapshot: live GitHub stats are unavailable right now, so the numbers below are sample data."
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
