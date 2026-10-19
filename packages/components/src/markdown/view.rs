//! The tree from [`super::parse`] as Dioxus elements.
//!
//! Every piece of text goes through Dioxus as a text node (escaped), every
//! `href` was already vetted by [`super::safe_href`], and nothing here ever
//! sets inner HTML. Recursion is bounded by the caps in the parent module.

use dioxus::prelude::*;

use super::{parse, Alert, Align, Block, Inline, Item, Style};
use crate::languages::use_strings;

const LINK_CLASS: &str = "underline decoration-dotted decoration-[var(--fg-secondary)] underline-offset-4 break-words hover:decoration-solid hover:decoration-[var(--fg-main)]";
const TEXT_CLASS: &str = "whitespace-pre-wrap break-words";
const CODE_BG: &str = "bg-[color-mix(in_srgb,var(--fg-main)_12%,transparent)]";

/// Renders Markdown `source` (an entry's text). Re-parses only when `source`
/// changes, because Dioxus skips components whose props are equal.
#[component]
pub fn Markdown(source: String) -> Element {
    let blocks = parse(&source);
    rsx! {
        div { class: "flex flex-col gap-2 min-w-0 break-words text-xs leading-relaxed text-[var(--fg-main)] [font:'JetBrains_Mono-Regular',Helvetica]",
            for (i , block) in blocks.into_iter().enumerate() {
                BlockView { key: "{i}", block }
            }
        }
    }
}

#[component]
fn Blocks(blocks: Vec<Block>) -> Element {
    rsx! {
        for (i , block) in blocks.into_iter().enumerate() {
            BlockView { key: "{i}", block }
        }
    }
}

#[component]
fn Inlines(nodes: Vec<Inline>) -> Element {
    rsx! {
        for (i , node) in nodes.into_iter().enumerate() {
            InlineView { key: "{i}", node }
        }
    }
}

fn align_class(align: Option<&Align>) -> &'static str {
    match align {
        Some(Align::Center) => "text-center",
        Some(Align::Right) => "text-right",
        _ => "text-left",
    }
}

/// Alignment of a `<div align=...>` group (inline content and images follow
/// `text-align`).
fn group_class(align: Align) -> &'static str {
    match align {
        Align::Center => "text-center",
        Align::Right => "text-right",
        Align::Left => "text-left",
        Align::Default => "",
    }
}

/// Border/label color for each GitHub alert kind.
fn alert_class(alert: Alert) -> &'static str {
    match alert {
        Alert::Note => "border-[var(--fg-main)] text-[var(--fg-main)]",
        Alert::Tip => "border-[var(--green)] text-[var(--green)]",
        Alert::Important | Alert::Warning => "border-[var(--yellow)] text-[var(--yellow)]",
        Alert::Caution => "border-[var(--red)] text-[var(--red)]",
    }
}

#[component]
fn BlockView(block: Block) -> Element {
    let s = use_strings();
    match block {
        Block::Paragraph(nodes) => rsx! {
            p { class: TEXT_CLASS, Inlines { nodes } }
        },
        Block::Tight(nodes) => rsx! {
            div { class: TEXT_CLASS, Inlines { nodes } }
        },
        Block::Heading { level, content } => {
            let size = match level {
                1 => "text-sm font-bold border-b border-solid border-[var(--fg-secondary)] pb-1",
                2 => {
                    "text-[13px] font-bold border-b border-solid border-[var(--fg-secondary)] pb-1"
                }
                _ => "text-xs font-bold",
            };
            rsx! {
                p {
                    role: "heading",
                    "aria-level": "{level}",
                    class: "{size} {TEXT_CLASS} mt-1",
                    Inlines { nodes: content }
                }
            }
        }
        Block::Quote { alert, blocks } => {
            let (frame, label) = match alert {
                Some(kind) => {
                    let label = match kind {
                        Alert::Note => s.guestbook.alert_note,
                        Alert::Tip => s.guestbook.alert_tip,
                        Alert::Important => s.guestbook.alert_important,
                        Alert::Warning => s.guestbook.alert_warning,
                        Alert::Caution => s.guestbook.alert_caution,
                    };
                    (alert_class(kind), Some(label))
                }
                None => (
                    "border-[var(--fg-secondary)] text-[var(--fg-main)]/80",
                    None,
                ),
            };
            rsx! {
                blockquote { class: "flex flex-col gap-1.5 border-0 border-l-2 border-solid pl-2.5 m-0 {frame}",
                    if let Some(label) = label {
                        p { class: "m-0 font-bold", "{label}" }
                    }
                    Blocks { blocks }
                }
            }
        }
        Block::List { start, items } => match start {
            Some(n) => rsx! {
                ol {
                    "start": "{n}",
                    class: "m-0 pl-5 list-decimal flex flex-col gap-1",
                    for (i , item) in items.into_iter().enumerate() {
                        ItemView { key: "{i}", item }
                    }
                }
            },
            None => rsx! {
                ul { class: "m-0 pl-5 list-disc flex flex-col gap-1",
                    for (i , item) in items.into_iter().enumerate() {
                        ItemView { key: "{i}", item }
                    }
                }
            },
        },
        Block::Code { lang, code } => rsx! {
            div { class: "flex flex-col border border-solid border-[var(--fg-secondary)] {CODE_BG}",
                if let Some(lang) = lang {
                    span { class: "px-2 py-0.5 text-[10px] text-[var(--fg-secondary)] border-0 border-b border-solid border-[var(--fg-secondary)]",
                        "{lang}"
                    }
                }
                pre { class: "m-0 p-2 overflow-x-auto [scrollbar-width:thin]",
                    code { class: "whitespace-pre [font:inherit]", "{code}" }
                }
            }
        },
        Block::Rule => rsx! {
            hr { class: "w-full m-0 border-0 border-t border-solid border-[var(--fg-secondary)]" }
        },
        Block::Aligned { align, blocks } => rsx! {
            div { class: "flex flex-col gap-1.5 min-w-0 {group_class(align)}", Blocks { blocks } }
        },
        Block::Details {
            summary,
            open,
            blocks,
        } => rsx! {
            details {
                open,
                class: "border border-solid border-[var(--fg-secondary)] px-2 py-1.5",
                summary { class: "cursor-pointer font-bold",
                    if summary.is_empty() {
                        "{s.guestbook.details_label}"
                    } else {
                        Inlines { nodes: summary }
                    }
                }
                div { class: "flex flex-col gap-1.5 pt-1.5", Blocks { blocks } }
            }
        },
        Block::Table { align, head, rows } => rsx! {
            div { class: "overflow-x-auto [scrollbar-width:thin]",
                table { class: "border-collapse text-xs",
                    thead {
                        tr {
                            for (i , cell) in head.into_iter().enumerate() {
                                th {
                                    key: "{i}",
                                    class: "px-2 py-1 font-bold border border-solid border-[var(--fg-secondary)] {align_class(align.get(i))}",
                                    Inlines { nodes: cell }
                                }
                            }
                        }
                    }
                    tbody {
                        for (r , row) in rows.into_iter().enumerate() {
                            tr { key: "{r}",
                                for (i , cell) in row.into_iter().enumerate() {
                                    td {
                                        key: "{i}",
                                        class: "px-2 py-1 border border-solid border-[var(--fg-secondary)] {align_class(align.get(i))}",
                                        Inlines { nodes: cell }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
    }
}

#[component]
fn ItemView(item: Item) -> Element {
    let Item { checked, blocks } = item;
    match checked {
        Some(done) => rsx! {
            li { class: "list-none -ml-5 flex items-start gap-2",
                input {
                    r#type: "checkbox",
                    checked: done,
                    disabled: true,
                    class: "mt-[3px] shrink-0 accent-[var(--fg-main)]",
                }
                div { class: "flex flex-col gap-1 min-w-0", Blocks { blocks } }
            }
        },
        None => rsx! {
            li { class: "pl-0.5 min-w-0",
                div { class: "flex flex-col gap-1", Blocks { blocks } }
            }
        },
    }
}

#[component]
fn InlineView(node: Inline) -> Element {
    let s = use_strings();
    match node {
        Inline::Text(t) => rsx! { "{t}" },
        Inline::Break => rsx! {
            br {}
        },
        Inline::Code(c) => rsx! {
            code { class: "px-1 py-px {CODE_BG} [font:inherit] break-words", "{c}" }
        },
        Inline::Emph(nodes) => rsx! {
            em { Inlines { nodes } }
        },
        Inline::Strong(nodes) => rsx! {
            strong { class: "font-bold", Inlines { nodes } }
        },
        Inline::Strike(nodes) => rsx! {
            del { class: "opacity-70", Inlines { nodes } }
        },
        Inline::Link { href, children } => {
            let web = href.starts_with("http");
            rsx! {
                a {
                    href: "{href}",
                    // The real destination is always one hover away, whatever the text says.
                    title: "{href}",
                    target: if web { "_blank" } else { "_self" },
                    rel: "noopener noreferrer nofollow ugc",
                    class: LINK_CLASS,
                    Inlines { nodes: children }
                }
            }
        }
        Inline::Styled(style, nodes) => match style {
            Style::Underline => rsx! {
                u { Inlines { nodes } }
            },
            Style::Mark => rsx! {
                mark { class: "bg-[var(--yellow)] px-0.5 text-black", Inlines { nodes } }
            },
            Style::Sub => rsx! {
                sub { Inlines { nodes } }
            },
            Style::Sup => rsx! {
                sup { Inlines { nodes } }
            },
            Style::Kbd => rsx! {
                kbd { class: "border border-solid border-[var(--fg-secondary)] px-1 [font:inherit]",
                    Inlines { nodes }
                }
            },
            Style::Code => rsx! {
                code { class: "px-1 py-px {CODE_BG} [font:inherit] break-words", Inlines { nodes } }
            },
        },
        Inline::Image {
            alt,
            href,
            width,
            height,
        } => match href {
            // Only `https` is loaded; see the module docs for why.
            Some(src) if src.starts_with("https://") => rsx! {
                Picture {
                    key: "{src}",
                    src,
                    alt,
                    width,
                    height,
                }
            },
            other => {
                let label = image_label(s, &alt);
                let title = other.unwrap_or_default();
                rsx! {
                    span { title: "{title}", class: "text-[var(--fg-secondary)]", "{label}" }
                }
            }
        },
    }
}

/// What stands in for an image that is not shown.
fn image_label(s: &crate::languages::Strings, alt: &str) -> String {
    if alt.is_empty() {
        s.guestbook.image_label.to_string()
    } else {
        s.guestbook.image_link(alt)
    }
}

/// An image from an entry. Loaded lazily, without a `Referer`, and capped in
/// size; if it fails to load, its alt text is shown instead.
#[component]
fn Picture(src: String, alt: String, width: Option<u16>, height: Option<u16>) -> Element {
    let s = use_strings();
    let mut failed = use_signal(|| false);
    if failed() {
        let label = image_label(s, &alt);
        return rsx! {
            span { title: "{src}", class: "text-[var(--fg-secondary)]", "{label}" }
        };
    }
    // One side is enough: the other follows, so the picture keeps its shape
    // even when `max-w-full` shrinks it.
    let (w, h, fit) = match (width, height) {
        (Some(w), _) => (Some(w), None, "h-auto"),
        (None, Some(h)) => (None, Some(h), "w-auto"),
        (None, None) => (None, None, ""),
    };
    rsx! {
        img {
            src: "{src}",
            alt: "{alt}",
            title: if alt.is_empty() { None } else { Some(alt.clone()) },
            width: w.map(|w| w.to_string()),
            height: h.map(|h| h.to_string()),
            loading: "lazy",
            decoding: "async",
            referrerpolicy: "no-referrer",
            class: "inline-block align-middle max-w-full max-h-80 object-contain {fit}",
            onerror: move |_| failed.set(true),
        }
    }
}
