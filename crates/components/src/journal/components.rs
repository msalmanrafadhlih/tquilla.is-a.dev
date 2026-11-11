use dioxus::prelude::*;

use api_types::journal::{Chronicle, Hero as HeroData, Repo};

use super::chart::build_chart_svg;
use super::util::{
    build_month_labels, fill_segments, format_growth, format_short_date, format_stars,
    format_volume, month_index,
};
use crate::languages::use_strings;
use crate::shared::{AVATAR, CONTACT_EMAIL};

#[component]
pub fn Hero(hero: HeroData, date_label: String, build_number: String, updated: String) -> Element {
    let s = use_strings();
    rsx! {
        header { class: "hero",
            div { class: "hero__inner",
                section { class: "hero__left",
                    div { class: "hero__portrait-col",
                        div { class: "hero__portrait",
                            img {
                                class: "hero__portrait-img",
                                alt: s.journal.profile_alt,
                                src: AVATAR,
                            }
                        }
                        div { class: "hero__stats",
                            div { class: "hero__stat",
                                span { class: "hero__stat-value", "{hero.total_repos}" }
                                span { "{s.journal.repos}" }
                            }
                            div { class: "hero__stat-divider" }
                            div { class: "hero__stat",
                                span { class: "hero__stat-value", "{format_stars(hero.total_stars)}" }
                                span { "{s.journal.stars}" }
                            }
                            div { class: "hero__stat-divider" }
                            div { class: "hero__stat",
                                span { class: "hero__stat-value", "{hero.total_followers}" }
                                span { "{s.journal.readers}" }
                            }
                        }
                    }
                }
                section { class: "hero__right",
                    div { class: "hero__corner",
                        div { class: "hero__corner-row",
                            span { "{date_label}" }
                            span { class: "hero__corner-line" }
                            span { "{build_number}" }
                        }
                        span { class: "hero__corner-text", "{updated}" }
                    }
                    div { class: "hero__content",
                        p { class: "hero__eyebrow", "{s.journal.eyebrow}" }
                        h2 { class: "hero__title",
                            "{s.journal.headline_1} "
                            br {}
                            "{s.journal.headline_2} "
                            span { class: "hero__title-em", "{s.journal.headline_3}" }
                        }
                        div { class: "hero__cols",
                            div { class: "hero__col",
                                p { class: "hero__col-title",
                                    "{s.journal.byline}"
                                }
                                div { class: "hero__links",
                                    a {
                                        class: "hero__link",
                                        href: "#",
                                        "{CONTACT_EMAIL}"
                                    }
                                }
                            }
                            div { class: "hero__col--wide",
                                p { class: "hero__lede",
                                    "{s.journal.quote}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// An empty description (GitHub repo without one) shows a neutral placeholder.
fn description_or_placeholder(description: &str, s: &crate::languages::Strings) -> String {
    if description.is_empty() {
        s.journal.no_description.to_string()
    } else {
        description.to_owned()
    }
}

/// Bar color when GitHub gave no color for a language.
const FALLBACK_LANGUAGE_COLOR: &str = "#D1CDC5";

#[component]
pub fn ArticleCard(repo: Option<Repo>, class: String) -> Element {
    let s = use_strings();
    rsx! {
        article { class: "{class}", "data-reveal": "true",
            if let Some(repo) = repo {
                div { class: "card__head",
                    span { class: "card__label",
                        "{repo.topic.clone().unwrap_or_else(|| s.journal.default_topic.to_string())}"
                    }
                    div { class: "card__meta",
                        span { class: "material-symbols-outlined hero__corner-text", "star" }
                        span { class: "mono-text", "{format_stars(repo.stars)}" }
                    }
                }
                div { class: "card__body",
                    h3 { class: "editorial-text card__title", "{repo.title}" }
                    p { class: "card__text",
                        "{description_or_placeholder(&repo.description, s)}"
                    }
                    span { class: "mono-text card__date",
                        "{repo.language.clone().unwrap_or_else(|| s.journal.unknown_language.to_string())}"
                    }
                }
            }
        }
    }
}

#[component]
pub fn PinnedSection(pinned: Vec<Repo>) -> Element {
    let s = use_strings();
    let feat = pinned.first().cloned();
    let sec1 = pinned.get(1).cloned();
    let sec2 = pinned.get(2).cloned();
    let ter1 = pinned.get(3).cloned();
    let ter2 = pinned.get(4).cloned();
    let ter3 = pinned.get(5).cloned();

    rsx! {
        section { class: "hero",
            div { class: "pinned",
                aside { class: "pinned__rail",
                    span { class: "editorial-text pinned__rail-text", "01." }
                }
                section { class: "pinned__main",
                    div { class: "pinned__top",
                        div {
                            class: "pinned__feature",
                            "data-reveal": "true",
                            if let Some(repo) = &feat {
                                div { class: "pinned__feature-head",
                                    span { class: "pinned__feature-label",
                                        "{repo.topic.clone().unwrap_or_else(|| s.journal.default_topic.to_string())}"
                                    }
                                    div { class: "pinned__feature-meta",
                                        span { class: "material-symbols-outlined hero__corner-text", "star" }
                                        span { class: "mono-text", "{format_stars(repo.stars)}" }
                                    }
                                }
                                div { class: "pinned__feature-body",
                                    h2 { class: "editorial-text pinned__feature-title", "{repo.title}" }
                                    p { class: "pinned__feature-text",
                                        "{description_or_placeholder(&repo.description, s)}"
                                    }
                                    div { class: "pinned__tags" }
                                }
                            }
                        }
                        div { class: "pinned__side",
                            ArticleCard {
                                repo: sec1,
                                class: "pinned__side-card".to_string(),
                            }
                            ArticleCard {
                                repo: sec2,
                                class: "pinned__side-card pinned__side-card--last".to_string(),
                            }
                        }
                    }
                    div { class: "pinned__bottom",
                        ArticleCard {
                            repo: ter1,
                            class: "pinned__tile pinned__tile--divided".to_string(),
                        }
                        ArticleCard {
                            repo: ter2,
                            class: "pinned__tile pinned__tile--divided".to_string(),
                        }
                        ArticleCard {
                            repo: ter3,
                            class: "pinned__tile".to_string(),
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ChronicleSection(chronicle: Chronicle, fetched_at: String) -> Element {
    let s = use_strings();
    let stack = chronicle.stack.clone();
    let months = build_month_labels(&chronicle.timeline, s);
    let chart_svg = build_chart_svg(&chronicle, s);
    let peak_day = chronicle
        .top_activities
        .first()
        .map(|d| format_short_date(&d.date))
        .unwrap_or_else(|| s.journal.no_peak.to_string());
    let weekday = s
        .calendar
        .weekday(usize::from(chronicle.most_productive_weekday).min(6));

    // The focus sentence is data + a UI template, never server-made HTML:
    // highlighted words become plain `span`s below.
    let focus_segments = match (&chronicle.monthly_focus, month_index(&fetched_at)) {
        (Some(focus), Some(month)) => fill_segments(
            s.journal.focus_sentence,
            &[
                ("month", s.calendar.month(month), false),
                (
                    "activity",
                    s.journal.activity_for(focus.language.as_deref()),
                    false,
                ),
                (
                    "language",
                    focus
                        .language
                        .as_deref()
                        .unwrap_or(s.journal.focus_language_fallback),
                    true,
                ),
                ("repo", focus.repo.as_str(), true),
            ],
        ),
        _ => fill_segments(
            s.journal.focus_empty,
            &[("ideas", s.journal.focus_ideas, true)],
        ),
    };

    // The original JS animates each dialect bar from width:0 to its target
    // percent 100ms after render. Reproduce that with a signal flip.
    let mut bars_visible = use_signal(|| false);
    use_effect(move || {
        spawn(async move {
            crate::platform::TimeoutFuture::new(100).await;
            bars_visible.set(true);
        });
    });

    rsx! {
        section { class: "chronicle",
            div { class: "chronicle__inner",
                div { class: "chronicle__wrap",
                    div {
                        class: "chronicle__head",
                        "data-reveal": "true",
                        div { class: "hero__content",
                            span { class: "chronicle__eyebrow", "{s.journal.section_label}" }
                            h1 { class: "chronicle__title",
                                "{s.journal.chronicle_title}"
                            }
                            p { class: "chronicle__lede",
                                "{s.journal.chronicle_subtitle}"
                            }
                        }
                        div { class: "chronicle__counter",
                            span { class: "chronicle__counter-label", "{s.journal.total_volume}" }
                            span { class: "chronicle__counter-value", "{format_volume(chronicle.total_contribution_volume)}" }
                            span { class: "chronicle__counter-delta",
                                span { class: "material-symbols-outlined", "trending_up" }
                                " {s.journal.growth_vs_last_year(&format_growth(chronicle.growth_percent))}"
                            }
                        }
                    }
                    div { class: "chronicle__grid",
                        div { class: "chronicle__col", "data-reveal": "true",
                            div { class: "chronicle__col-head",
                                span { class: "material-symbols-outlined chronicle__col-icon", "code" }
                                h3 { class: "chronicle__col-title", "{s.journal.dialects}" }
                            }
                            div { class: "chronicle__langs",
                                for lang in chronicle.languages.iter() {
                                    {
                                        let bar_scale = if bars_visible() { lang.percent / 100.0 } else { 0.0 };
                                        rsx! {
                                            div { class: "group chronicle__lang", key: "{lang.name}",
                                                div { class: "chronicle__lang-head",
                                                    span { class: "chronicle__lang-name", "{lang.name}" }
                                                    span { class: "chronicle__lang-pct", "{lang.percent}%" }
                                                }
                                                div { class: "chronicle__bar",
                                                    div {
                                                        class: "chronicle__bar-fill",
                                                        style: "transform: scaleX({bar_scale}); background-color: {lang.color.as_deref().unwrap_or(FALLBACK_LANGUAGE_COLOR)};",
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if !stack.is_empty() {
                                div { class: "chronicle__stack",
                                    h4 { class: "chronicle__stack-title", "{s.journal.core_stack}" }
                                    div { class: "chronicle__stack-tags",
                                        for item in stack.iter() {
                                            span { class: "chronicle__stack-tag", key: "{item}", "{item}" }
                                        }
                                    }
                                }
                            }
                        }
                        div {
                            class: "chronicle__col--wide",
                            "data-reveal": "true",
                            div { class: "chronicle__chart-head",
                                div { class: "chronicle__chart-label",
                                    span { class: "material-symbols-outlined chronicle__col-icon", "timeline" }
                                    h3 { class: "chronicle__col-title", "{s.journal.timeline}" }
                                }
                                div { class: "chronicle__chart-stats",
                                    div { class: "chronicle__chart-stat",
                                        span { class: "chronicle__chart-stat-label", "{s.journal.current_streak}" }
                                        span { class: "chronicle__chart-stat-value", "{s.journal.streak_days(chronicle.current_streak)}" }
                                    }
                                    div { class: "chronicle__chart-stat",
                                        span { class: "chronicle__chart-stat-label", "{s.journal.peak_activity}" }
                                        span { class: "chronicle__chart-stat-value", "{peak_day}" }
                                    }
                                }
                            }
                            div { class: "chronicle__chart",
                                div { class: "chronicle__chart-grid",
                                    div { class: "chronicle__chart-line" }
                                    div { class: "chronicle__chart-line" }
                                    div { class: "chronicle__chart-line" }
                                    div { class: "chronicle__chart-line" }
                                    div { class: "chronicle__chart-baseline" }
                                }
                                div { dangerous_inner_html: "{chart_svg}" }
                            }
                            div { class: "chronicle__chart-axis",
                                for m in months.iter() {
                                    span { key: "{m}", "{m}" }
                                }
                            }
                            div { class: "chronicle__notes",
                                div { class: "chronicle__card", "data-reveal": "true",
                                    h4 { class: "chronicle__note-title", "{s.journal.monthly_focus}" }
                                    p { class: "chronicle__note-text",
                                        for (i, seg) in focus_segments.iter().enumerate() {
                                            if seg.highlight {
                                                span { key: "{i}", class: "chronicle__note-em", "{seg.text}" }
                                            } else {
                                                span { key: "{i}", "{seg.text}" }
                                            }
                                        }
                                    }
                                }
                                div {
                                    class: "chronicle__card chronicle__card--stat",
                                    "data-reveal": "true",
                                    div { class: "card__head",
                                        h4 { class: "chronicle__stat-title", "{s.journal.most_productive}" }
                                        span { class: "material-symbols-outlined chronicle__stat-icon", "bolt" }
                                    }
                                    div {
                                        div { class: "chronicle__stat-value", "{weekday}" }
                                        div { class: "chronicle__stat-label", "{s.journal.based_on_commits}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn Footer(year: i32) -> Element {
    let s = use_strings();
    rsx! {
        footer { class: "journal-footer",
            div { class: "journal-footer__diamond" }
            div { class: "journal-footer__inner",
                div { class: "journal-footer__stack", "data-reveal": "true",
                    div { class: "journal-footer__block",
                        span { class: "material-symbols-outlined journal-footer__icon", "public" }
                        h4 { class: "journal-footer__heading",
                            "{s.journal.published_in}"
                        }
                    }
                    div { class: "journal-footer__quote-box",
                        p { class: "journal-footer__quote",
                            "{s.journal.tagline}"
                        }
                    }
                    div { class: "journal-footer__bottom",
                        div { class: "chronicle__chart-label",
                            span { class: "journal-footer__meta", "{s.journal.built_with}" }
                            span { class: "journal-footer__line" }
                            span { class: "journal-footer__meta", "v1.0.1" }
                        }
                        div { class: "journal-footer__center",
                            p { class: "journal-footer__tagline", "{s.journal.copyright(year)}" }
                        }
                    }
                }
            }
            div { class: "journal-footer__bar" }
        }
    }
}

#[component]
pub fn EndOfStream() -> Element {
    let s = use_strings();
    rsx! {
        div { class: "end-of-stream",
            div { class: "end-of-stream__inner",
                div { class: "end-of-stream__line" }
                p { class: "end-of-stream__text", "{s.journal.end_of_stream}" }
            }
        }
    }
}
