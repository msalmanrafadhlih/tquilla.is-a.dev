use dioxus::prelude::*;

use super::chart::build_chart_svg;
use super::data::{Chronicle, Hero as HeroData, Repo};
use super::util::{build_month_labels, format_stars, format_volume};
use crate::languages::use_strings;
use crate::shared::{AVATAR, CONTACT_EMAIL};

#[component]
pub fn Hero(hero: HeroData, date_label: String, build_number: String) -> Element {
    let s = use_strings();
    rsx! {
        header { class: "w-full border-b border-line bg-paper",
            div { class: "max-w-[1400px] mx-auto flex flex-col lg:flex-row border-l border-r border-line",
                section { class: "w-full lg:w-2/5 flex flex-col items-center justify-center border-r border-line bg-paper p-12 lg:p-0",
                    div { class: "flex flex-col items-center gap-12",
                        div { class: "size-72 overflow-hidden bg-primary filter grayscale contrast-125 mix-blend-darken",
                            img {
                                class: "w-full h-full object-cover",
                                alt: s.journal.profile_alt,
                                src: AVATAR,
                            }
                        }
                        div { class: "w-72 flex justify-between items-center border-t border-b border-line py-3 font-mono tracking-wider text-muted uppercase",
                            div { class: "flex flex-col items-center",
                                span { class: "font-bold text-primary", "{hero.total_repos}" }
                                span { "{s.journal.repos}" }
                            }
                            div { class: "w-px h-6 bg-line" }
                            div { class: "flex flex-col items-center",
                                span { class: "font-bold text-primary", "{format_stars(hero.total_stars)}" }
                                span { "{s.journal.stars}" }
                            }
                            div { class: "w-px h-6 bg-line" }
                            div { class: "flex flex-col items-center",
                                span { class: "font-bold text-primary", "{hero.total_followers}" }
                                span { "{s.journal.readers}" }
                            }
                        }
                    }
                }
                section { class: "w-full lg:w-3/5 flex flex-col justify-center px-8 py-16 lg:px-24 lg:py-32 bg-paper relative",
                    div { class: "absolute top-6 right-6 lg:top-10 lg:right-10 flex items-center gap-2 text-muted font-mono",
                        span { "{date_label}" }
                        span { class: "w-8 h-px bg-line" }
                        span { "{build_number}" }
                    }
                    div { class: "max-w-2xl",
                        p { class: "font-mono text-accent text-sm tracking-widest mb-6 uppercase", "{s.journal.eyebrow}" }
                        h2 { class: "font-serif text-5xl sm:text-9xl leading-[0.85] text-primary mb-10 tracking-tighter",
                            "{s.journal.headline_1} "
                            br {}
                            "{s.journal.headline_2} "
                            span { class: "italic font-light", "{s.journal.headline_3}" }
                        }
                        div { class: "flex flex-col sm:flex-row gap-8 sm:gap-12 pt-8 border-t border-line text-sm",
                            div { class: "flex-1 max-w-[280px]",
                                p { class: "font-sans font-medium leading-relaxed mb-1",
                                    "{s.journal.byline}"
                                }
                                div { class: "flex gap-4 mt-4",
                                    a {
                                        class: "font-bold tracking-wider border-b border-transparent pb-0.5",
                                        href: "#",
                                        "{CONTACT_EMAIL}"
                                    }
                                }
                            }
                            div { class: "flex-1",
                                p { class: "font-serif text-xl leading-relaxed text-primary/80 italic",
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

#[component]
pub fn ArticleCard(repo: Option<Repo>, class: String) -> Element {
    let s = use_strings();
    rsx! {
        article { class: "{class}", "data-reveal": "true",
            if let Some(repo) = repo {
                div { class: "flex justify-between items-start mb-4",
                    span { class: "font-sans font-bold uppercase text-sm tracking-[0.15em] text-ink/40",
                        "{repo.topic.clone().unwrap_or_else(|| s.journal.default_topic.to_string())}"
                    }
                    div { class: "flex items-center gap-1 text-ink/40",
                        span { class: "material-symbols-outlined text-xs", "star" }
                        span { class: "mono-text", "{format_stars(repo.stars)}" }
                    }
                }
                div { class: "relative z-10 mt-auto",
                    h3 { class: "editorial-text text-3xl sm:text-4xl font-normal mb-2", "{repo.title}" }
                    p { class: "font-sans text-ink/60 mb-4 line-clamp-2", "{repo.description}" }
                    span { class: "mono-text text-ink/40", "{repo.language}" }
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
        section { class: "w-full border-b border-line bg-paper",
            div { class: "max-w-[1400px] mx-auto flex flex-col md:flex-row border-l border-r border-line",
                aside { class: "w-full md:w-24 md:border-r border-b md:border-b-0 border-line p-6 bg-paper z-40",
                    span { class: "editorial-text text-4xl md:text-2xl italic font-bold text-ink/40", "01." }
                }
                section { class: "flex-1 w-full h-full flex flex-col",
                    div { class: "grid grid-cols-1 lg:grid-cols-12 h-2/3 border-b border-line",
                        div {
                            class: "lg:col-span-8 border-b lg:border-b-0 lg:border-r border-line p-8 md:p-12 lg:p-16 flex flex-col justify-between relative",
                            "data-reveal": "true",
                            if let Some(repo) = &feat {
                                div { class: "flex justify-between items-start mb-8",
                                    span { class: "font-sans font-bold uppercase tracking-[0.15em] text-ink/60",
                                        "{repo.topic.clone().unwrap_or_else(|| s.journal.default_topic.to_string())}"
                                    }
                                    div { class: "flex items-center gap-1 text-ink/60",
                                        span { class: "material-symbols-outlined text-xs", "star" }
                                        span { class: "mono-text", "{format_stars(repo.stars)}" }
                                    }
                                }
                                div { class: "max-w-2xl z-10",
                                    h2 { class: "editorial-text text-4xl sm:text-8xl font-normal leading-none mb-6", "{repo.title}" }
                                    p { class: "font-sans text-xl leading-relaxed text-ink/80 mb-8 border-l-2 border-accent pl-6",
                                        "{repo.description}"
                                    }
                                    div { class: "flex flex-wrap gap-2 mb-8" }
                                }
                            }
                        }
                        div { class: "lg:col-span-4 flex flex-col h-full",
                            ArticleCard {
                                repo: sec1,
                                class: "flex-1 border-b border-line p-8 md:p-10 relative overflow-hidden flex flex-col justify-between".to_string(),
                            }
                            ArticleCard {
                                repo: sec2,
                                class: "flex-1 border-b lg:border-b-0 border-line p-8 md:p-10 relative overflow-hidden flex flex-col justify-between".to_string(),
                            }
                        }
                    }
                    div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 h-1/3",
                        ArticleCard {
                            repo: ter1,
                            class: "border-b md:border-b-0 border-r border-line p-8 flex flex-col justify-between h-full".to_string(),
                        }
                        ArticleCard {
                            repo: ter2,
                            class: "border-b md:border-b-0 border-r border-line p-8 flex flex-col justify-between h-full".to_string(),
                        }
                        ArticleCard {
                            repo: ter3,
                            class: "border-b md:border-b-0 border-line p-8 flex flex-col justify-between h-full".to_string(),
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ChronicleSection(chronicle: Chronicle) -> Element {
    let s = use_strings();
    let stack = chronicle.stack.clone().unwrap_or_else(|| {
        vec![
            "Linux".into(),
            "Neovim".into(),
            "Arch".into(),
            "Hyprland".into(),
        ]
    });
    let months = build_month_labels(&chronicle.stats.timeline, s);
    let chart_svg = build_chart_svg(&chronicle, s);
    let focus_html = chronicle.monthly_focus_html.clone();
    let focus_text = chronicle.monthly_focus.clone();

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
        section { class: "w-full bg-paper border-b border-line",
            div { class: "max-w-[1400px] mx-auto border-l border-r border-line py-24 px-6 md:px-16 flex flex-col items-center",
                div { class: "flex flex-col w-full",
                    div {
                        class: "flex flex-col md:flex-row md:items-end justify-between border-b border-line pb-8 mb-12",
                        "data-reveal": "true",
                        div { class: "max-w-2xl",
                            span { class: "block text-accent font-mono text-sm mb-4", "{s.journal.section_label}" }
                            h1 { class: "text-primary font-serif text-5xl md:text-8xl italic font-bold leading-tight mb-4",
                                "{s.journal.chronicle_title}"
                            }
                            p { class: "text-primary/70 font-serif text-2xl md:text-3xl leading-relaxed max-w-lg",
                                "{s.journal.chronicle_subtitle}"
                            }
                        }
                        div { class: "flex flex-col gap-2 mt-8 md:mt-0 text-right",
                            span { class: "text-sm font-bold uppercase tracking-widest text-primary/40", "{s.journal.total_volume}" }
                            span { class: "text-5xl md:text-6xl font-mono text-primary", "{format_volume(chronicle.total_contribution_volume)}" }
                            span { class: "text-sm font-medium text-accent flex justify-end items-center gap-1",
                                span { class: "material-symbols-outlined", "trending_up" }
                                " {s.journal.growth_vs_last_year(&chronicle.growth_percentage)}"
                            }
                        }
                    }
                    div { class: "grid grid-cols-1 lg:grid-cols-3 gap-0 lg:divide-x divide-line min-h-[600px] border-b border-line",
                        div { class: "p-0 lg:pr-12 pb-12 lg:pb-0", "data-reveal": "true",
                            div { class: "flex items-center gap-3 mb-10",
                                span { class: "material-symbols-outlined text-accent text-xl", "code" }
                                h3 { class: "font-serif text-3xl text-primary italic", "{s.journal.dialects}" }
                            }
                            div { class: "space-y-10",
                                for lang in chronicle.languages.iter() {
                                    {
                                        let bar_width = if bars_visible() { lang.percent } else { 0.0 };
                                        rsx! {
                                            div { class: "group cursor-default", key: "{lang.name}",
                                                div { class: "flex justify-between items-baseline mb-3",
                                                    span { class: "font-display font-bold text-lg text-primary", "{lang.name}" }
                                                    span { class: "font-mono text-sm text-primary/60", "{lang.percent}%" }
                                                }
                                                div { class: "h-[2px] w-full bg-line",
                                                    div {
                                                        class: "dialect-bar h-[2px] transition-all duration-1000 ease-out",
                                                        style: "width: {bar_width}%; background-color: {lang.color};",
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            div { class: "mt-16 pt-8 border-t border-line",
                                h4 { class: "font-mono text-sm uppercase tracking-widest text-primary/40 mb-4", "{s.journal.core_stack}" }
                                div { class: "flex flex-wrap gap-2",
                                    for s in stack.iter() {
                                        span { class: "px-2 py-1 border border-line text-sm font-medium text-primary/70", key: "{s}", "{s}" }
                                    }
                                }
                            }
                        }
                        div {
                            class: "col-span-1 lg:col-span-2 lg:pl-12 pt-12 lg:pt-0 flex flex-col",
                            "data-reveal": "true",
                            div { class: "flex items-center justify-between mb-10",
                                div { class: "flex items-center gap-3",
                                    span { class: "material-symbols-outlined text-accent text-xl", "timeline" }
                                    h3 { class: "font-serif text-3xl text-primary italic", "{s.journal.timeline}" }
                                }
                                div { class: "flex gap-6",
                                    div { class: "text-right",
                                        span { class: "block text-sm font-bold uppercase tracking-widest text-primary/40", "{s.journal.current_streak}" }
                                        span { class: "font-mono text-xl font-bold text-primary", "{s.journal.streak_days(chronicle.current_streak)}" }
                                    }
                                    div { class: "text-right",
                                        span { class: "block text-sm font-bold uppercase tracking-widest text-primary/40", "{s.journal.peak_activity}" }
                                        span { class: "font-mono text-xl font-bold text-primary", "{chronicle.peak_activity_day.date}" }
                                    }
                                }
                            }
                            div { class: "relative flex-1 min-h-[300px] w-full mt-4 group/graph",
                                div { class: "absolute inset-0 flex flex-col justify-between pointer-events-none",
                                    div { class: "w-full border-t border-dashed border-line/50" }
                                    div { class: "w-full border-t border-dashed border-line/50" }
                                    div { class: "w-full border-t border-dashed border-line/50" }
                                    div { class: "w-full border-t border-dashed border-line/50" }
                                    div { class: "w-full border-b border-line" }
                                }
                                div { dangerous_inner_html: "{chart_svg}" }
                            }
                            div { class: "flex justify-between w-full mt-4 pt-4 border-t border-line text-sm font-bold uppercase tracking-widest text-primary/50 font-sans",
                                for m in months.iter() {
                                    span { key: "{m}", "{m}" }
                                }
                            }
                            div { class: "grid grid-cols-1 md:grid-cols-2 gap-8 mt-12",
                                div { class: "p-6 bg-white border border-line", "data-reveal": "true",
                                    h4 { class: "font-serif italic text-2xl mb-2 text-primary", "{s.journal.monthly_focus}" }
                                    if let Some(html) = &focus_html {
                                        p { class: "text-sm leading-relaxed text-primary/70", dangerous_inner_html: "{html}" }
                                    } else {
                                        p { class: "text-sm leading-relaxed text-primary/70", "{s.journal.focus_text(&focus_text)}" }
                                    }
                                }
                                div {
                                    class: "p-6 bg-white border border-line flex flex-col justify-between",
                                    "data-reveal": "true",
                                    div { class: "flex justify-between items-start mb-4",
                                        h4 { class: "font-serif italic text-2xl text-primary", "{s.journal.most_productive}" }
                                        span { class: "material-symbols-outlined text-accent", "bolt" }
                                    }
                                    div {
                                        div { class: "text-4xl font-serif font-bold italic text-primary", "{chronicle.most_productive_day}" }
                                        div { class: "text-sm uppercase tracking-widest text-primary/50 mt-1", "{s.journal.based_on_commits}" }
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
        footer { class: "w-full relative border-t border-line mt-auto bg-paper",
            div { class: "absolute top-0 left-1/2 -translate-x-1/2 -translate-y-1/2 w-3 h-3 bg-primary rotate-45" }
            div { class: "max-w-[1400px] mx-auto py-16 px-4 md:px-12 border-l border-r border-line",
                div { class: "flex flex-col items-center justify-center text-center space-y-12", "data-reveal": "true",
                    div { class: "space-y-4",
                        span { class: "material-symbols-outlined text-primary text-3xl mb-2", "public" }
                        h4 { class: "font-display text-sm md:text-sm font-bold tracking-[0.2em] uppercase text-muted",
                            "{s.journal.published_in}"
                        }
                    }
                    div { class: "max-w-lg",
                        p { class: "font-serif text-4xl italic text-ink leading-relaxed",
                            "{s.journal.tagline}"
                        }
                    }
                    div { class: "pt-12 mt-4 w-full flex flex-col md:flex-row justify-between items-center border-t border-line/30 gap-6",
                        div { class: "flex items-center gap-3",
                            span { class: "font-mono text-muted uppercase tracking-tight", "{s.journal.built_with}" }
                            span { class: "h-px w-8 bg-line" }
                            span { class: "font-mono text-muted uppercase tracking-tight", "v1.0.1" }
                        }
                        div { class: "text-center",
                            p { class: "font-display text-muted tracking-wide", "{s.journal.copyright(year)}" }
                        }
                    }
                }
            }
            div { class: "h-2 w-full bg-ink dark:bg-white/10" }
        }
    }
}

#[component]
pub fn EndOfStream() -> Element {
    let s = use_strings();
    rsx! {
        div { class: "flex-grow flex items-center justify-center py-10 opacity-20 pointer-events-none select-none",
            div { class: "text-center space-y-4",
                div { class: "h-24 w-px bg-line mx-auto" }
                p { class: "font-serif italic text-3xl text-primary", "{s.journal.end_of_stream}" }
            }
        }
    }
}
