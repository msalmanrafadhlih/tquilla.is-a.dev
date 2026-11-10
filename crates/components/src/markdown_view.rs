//! The tree from [`markdown::parse`] as Dioxus elements.
//!
//! Every piece of text goes through Dioxus as a text node (escaped), every
//! `href` was already vetted by [`markdown::safe_href`], and nothing here ever
//! sets inner HTML. Recursion is bounded by the caps in the parent module.

use dioxus::prelude::*;

use crate::languages::use_strings;
use markdown::images::{image_host, is_icon_sized, is_trusted_image};
use markdown::{parse, Alert, Align, Block, Inline, Item, Style};

/// Hosts whose images the reader chose to load, for this visit only (memory,
/// never saved): one click on "Load image from <host>" opens every image from
/// that host, so a row of badges needs one click, not five.
static LOADED_HOSTS: GlobalSignal<Vec<String>> = Signal::global(Vec::new);

const LINK_CLASS: &str = "md__link";
// `flow-root`: a floated image (`<img align="left">`) must stay inside the block
// that holds it instead of spilling over the next one.
const TEXT_CLASS: &str = "md__text";
const CODE_BG: &str = "md__code-bg";

/// Renders Markdown `source` (an entry's text). Re-parses only when `source`
/// changes, because Dioxus skips components whose props are equal.
#[component]
pub fn Markdown(source: String) -> Element {
    let blocks = parse(&source);
    rsx! {
        div { class: "md",
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
        Some(Align::Center) => "md-align--center",
        Some(Align::Right) => "md-align--right",
        _ => "md-align--left",
    }
}

/// Alignment of a `<div align=...>` group (inline content and images follow
/// `text-align`).
fn group_class(align: Align) -> &'static str {
    match align {
        Align::Center => "md-align--center",
        Align::Right => "md-align--right",
        Align::Left => "md-align--left",
        Align::Default => "",
    }
}

/// Border/label color for each GitHub alert kind.
fn alert_class(alert: Alert) -> &'static str {
    match alert {
        Alert::Note => "md-alert--note",
        Alert::Tip => "md-alert--tip",
        Alert::Important | Alert::Warning => "md-alert--warning",
        Alert::Caution => "md-alert--caution",
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
                1 => "md__h1",
                2 => "md__h2",
                _ => "md__h3",
            };
            rsx! {
                p {
                    role: "heading",
                    "aria-level": "{level}",
                    class: "{size} {TEXT_CLASS} md__lead",
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
                None => ("md-alert--none", None),
            };
            rsx! {
                blockquote { class: "md__quote {frame}",
                    if let Some(label) = label {
                        p { class: "md__alert-title", "{label}" }
                    }
                    Blocks { blocks }
                }
            }
        }
        Block::List { start, items } => match start {
            Some(n) => rsx! {
                ol {
                    "start": "{n}",
                    class: "md__ol",
                    for (i , item) in items.into_iter().enumerate() {
                        ItemView { key: "{i}", item }
                    }
                }
            },
            None => rsx! {
                ul { class: "md__ul",
                    for (i , item) in items.into_iter().enumerate() {
                        ItemView { key: "{i}", item }
                    }
                }
            },
        },
        Block::Code { lang, code } => rsx! {
            div { class: "md__code-block {CODE_BG}",
                if let Some(lang) = lang {
                    span { class: "md__code-lang",
                        "{lang}"
                    }
                }
                pre { class: "md__pre",
                    code { class: "md__code", "{code}" }
                }
            }
        },
        Block::Rule => rsx! {
            hr { class: "md__hr" }
        },
        Block::Aligned { align, blocks } => rsx! {
            div { class: "md__group {group_class(align)}", Blocks { blocks } }
        },
        Block::Details {
            summary,
            open,
            blocks,
        } => rsx! {
            details {
                open,
                class: "md__details",
                summary { class: "md__summary",
                    if summary.is_empty() {
                        "{s.guestbook.details_label}"
                    } else {
                        Inlines { nodes: summary }
                    }
                }
                div { class: "md__details-body", Blocks { blocks } }
            }
        },
        Block::Table { align, head, rows } => rsx! {
            div { class: "md__table-wrap",
                table { class: "md__table",
                    thead {
                        tr {
                            for (i , cell) in head.into_iter().enumerate() {
                                th {
                                    key: "{i}",
                                    class: "md__th {align_class(align.get(i))}",
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
                                        class: "md__td {align_class(align.get(i))}",
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
            li { class: "md__task",
                input {
                    r#type: "checkbox",
                    checked: done,
                    disabled: true,
                    class: "md__task-box",
                }
                div { class: "md__task-body", Blocks { blocks } }
            }
        },
        None => rsx! {
            li { class: "md__item",
                div { class: "md__item-body", Blocks { blocks } }
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
            code { class: "md__icode-pad {CODE_BG} md__icode-font", "{c}" }
        },
        Inline::Emph(nodes) => rsx! {
            em { Inlines { nodes } }
        },
        Inline::Strong(nodes) => rsx! {
            strong { class: "md__strong", Inlines { nodes } }
        },
        Inline::Strike(nodes) => rsx! {
            del { class: "md__del", Inlines { nodes } }
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
                mark { class: "md__mark", Inlines { nodes } }
            },
            Style::Sub => rsx! {
                sub { Inlines { nodes } }
            },
            Style::Sup => rsx! {
                sup { Inlines { nodes } }
            },
            Style::Kbd => rsx! {
                kbd { class: "md__kbd",
                    Inlines { nodes }
                }
            },
            Style::Code => rsx! {
                code { class: "md__icode-pad {CODE_BG} md__icode-font", Inlines { nodes } }
            },
        },
        Inline::Image {
            alt,
            href,
            width,
            height,
            align,
        } => match href {
            // Only `https` is loaded; see the module docs for why.
            Some(src) if src.starts_with("https://") => rsx! {
                Picture {
                    key: "{src}",
                    src,
                    alt,
                    width,
                    height,
                    align,
                }
            },
            other => {
                let label = image_label(s, &alt);
                let title = other.unwrap_or_default();
                rsx! {
                    span { title: "{title}", class: "md__muted", "{label}" }
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

/// Where a picture sits: inline by default, floated to an edge for
/// `align="left|right"` (text wraps around it), or alone on a centered line.
fn picture_place(align: Align) -> &'static str {
    match align {
        Align::Left => "md-float--left",
        Align::Right => "md-float--right",
        Align::Center => "md-float--center",
        Align::Default => "md-float--inline",
    }
}

const ASK_CLASS: &str = "md__ask";

/// An image from an entry. Images from hosts on the list in
/// [`markdown::images`] load at once; every other one waits behind a button,
/// so the reader's browser does not contact a stranger's server unasked. Loaded
/// lazily, without a `Referer`, and capped in size; if it fails to load, its
/// alt text is shown instead.
#[component]
fn Picture(
    src: String,
    alt: String,
    width: Option<u16>,
    height: Option<u16>,
    align: Align,
) -> Element {
    let s = use_strings();
    let mut failed = use_signal(|| false);
    // Only for an address whose host cannot be read: it cannot be remembered
    // per host, so this one image is opened on its own.
    let mut opened = use_signal(|| false);
    let host = image_host(&src);
    let allowed = is_trusted_image(&src)
        || opened()
        || host
            .as_ref()
            .is_some_and(|h| LOADED_HOSTS.read().contains(h));
    if !allowed {
        let label = image_label(s, &alt);
        let shown_host = host
            .clone()
            .unwrap_or_else(|| s.guestbook.image_unknown_host.to_string());
        let ask = s.guestbook.image_load(&shown_host);
        let place = picture_place(align);
        let on_click = move |_| match &host {
            Some(h) => LOADED_HOSTS.write().push(h.clone()),
            None => opened.set(true),
        };
        // Icons and badges: a small square, not a full button.
        return if is_icon_sized(width, height) {
            let side = width.or(height).unwrap_or(20).max(16);
            rsx! {
                button {
                    r#type: "button",
                    title: "{ask}",
                    "aria-label": "{ask}",
                    style: "width:{side}px;height:{side}px",
                    class: "{place} {ASK_CLASS} md__ask-reset",
                    onclick: on_click,
                    "\u{25A3}"
                }
            }
        } else {
            rsx! {
                button {
                    r#type: "button",
                    title: "{src}",
                    class: "{place} {ASK_CLASS}",
                    onclick: on_click,
                    span { class: "md__ask-line", "{label}" }
                    span { class: "md__ask-action", "{ask}" }
                }
            }
        };
    }
    if failed() {
        let label = image_label(s, &alt);
        return rsx! {
            span { title: "{src}", class: "md__muted", "{label}" }
        };
    }
    // One side is enough: the other follows, so the picture keeps its shape
    // even when `max-w-full` shrinks it.
    let (w, h, fit) = match (width, height) {
        (Some(w), _) => (Some(w), None, "md-size--h-auto"),
        (None, Some(h)) => (None, Some(h), "md-size--w-auto"),
        (None, None) => (None, None, ""),
    };
    let place = picture_place(align);
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
            class: "{place} md__img {fit}",
            onerror: move |_| failed.set(true),
        }
    }
}
