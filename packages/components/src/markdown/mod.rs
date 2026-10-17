//! Markdown for Guestbook entries ("README engine").
//!
//! The server stores the text exactly as written; this module is the only place
//! that gives it meaning. The design is driven by one rule: **text from
//! strangers never becomes markup**. So there is no HTML string anywhere:
//!
//! 1. `pulldown-cmark` turns the source into a stream of events;
//! 2. [`build`] folds the events into a small tree ([`Block`] / [`Inline`]);
//! 3. [`view`] turns the tree into Dioxus elements (text nodes are escaped by
//!    Dioxus itself).
//!
//! What is supported: paragraphs, headings, emphasis, strong, ~~strikethrough~~,
//! inline code, fenced and indented code blocks, block quotes (and GitHub
//! alerts: `> [!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, `[!CAUTION]`),
//! ordered / unordered / nested lists, task lists, tables with alignment,
//! horizontal rules, links, and bare URLs (`https://...`, `www....`).
//!
//! What is deliberately not rendered:
//! - **Raw HTML** is shown as literal text, like a code listing would.
//! - **Images** are shown as a link to the image, never loaded: an image in an
//!   entry would make every visitor's browser contact a stranger's server.
//! - **Link targets** other than `http(s)://` and `mailto:` lose the link and
//!   keep the text (`javascript:`, `data:`, relative paths, ...).
//! - Footnotes, math, definition lists, heading ids, and so on are off.
//!
//! Hostile input is expected: container nesting and inline nesting are capped
//! ([`MAX_BLOCK_DEPTH`], [`MAX_INLINE_DEPTH`]); anything deeper is flattened,
//! with its text kept. The builder never panics on odd event sequences.

use api_types::validate::is_http_url;
use pulldown_cmark::{
    Alignment, BlockQuoteKind, CodeBlockKind, Event, LinkType, Options, Parser, Tag, TagEnd,
};

mod view;

pub use view::Markdown;

/// How many block quotes / lists may be nested. The input is capped at 2,000
/// characters, but `>>>>>>...` could still nest a thousand levels deep and
/// every level is a component on the stack.
pub const MAX_BLOCK_DEPTH: usize = 6;
/// How many emphasis / link spans may be nested.
pub const MAX_INLINE_DEPTH: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Default,
    Left,
    Center,
    Right,
}

/// GitHub-style alert kinds (`> [!NOTE]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alert {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

/// A table cell: inline content only.
pub type Cell = Vec<Inline>;

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Paragraph(Vec<Inline>),
    /// Text of a tight list item: a paragraph without paragraph spacing.
    Tight(Vec<Inline>),
    Heading {
        level: u8,
        content: Vec<Inline>,
    },
    Quote {
        alert: Option<Alert>,
        blocks: Vec<Block>,
    },
    List {
        start: Option<u64>,
        items: Vec<Item>,
    },
    Code {
        /// Language label from the info string, already restricted to a safe,
        /// short alphabet (it is only ever shown as text).
        lang: Option<String>,
        code: String,
    },
    Rule,
    Table {
        align: Vec<Align>,
        head: Vec<Cell>,
        rows: Vec<Vec<Cell>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    /// `Some` for task list items (`- [ ]` / `- [x]`).
    pub checked: Option<bool>,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    Text(String),
    Code(String),
    /// A line break (soft breaks count too: entries read like comments, where
    /// pressing Enter starts a new line).
    Break,
    Emph(Vec<Inline>),
    Strong(Vec<Inline>),
    Strike(Vec<Inline>),
    /// `href` is always `http(s)://...` or `mailto:...` (see [`safe_href`]).
    Link {
        href: String,
        children: Vec<Inline>,
    },
    /// An image, shown as a link. `href` is `None` when the target is unsafe.
    Image {
        alt: String,
        href: Option<String>,
    },
}

/// Parses `source` into blocks. Never fails and never panics.
pub fn parse(source: &str) -> Vec<Block> {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_GFM;
    build(Parser::new_ext(source, options))
}

// ---------------------------------------------------------------------------
// Links
// ---------------------------------------------------------------------------

/// The only link targets an entry may produce: absolute `http(s)://` (the
/// shared server/client rule) and plain `mailto:` addresses. Everything else
/// (`javascript:`, `data:`, `vbscript:`, `file:`, relative paths, protocol-
/// relative `//host`, uppercase schemes) is refused, and the link text stays.
pub fn safe_href(dest: &str) -> Option<String> {
    let dest = dest.trim();
    if is_http_url(dest) {
        return Some(dest.to_owned());
    }
    let addr = dest.strip_prefix("mailto:")?;
    let plain = !addr.is_empty()
        && addr.len() <= 254
        && addr.matches('@').count() == 1
        && !addr.starts_with('@')
        && !addr.ends_with('@')
        && addr
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '+' | '-' | '_' | '@'));
    plain.then(|| dest.to_owned())
}

/// Characters a bare URL may not end with (GFM "extended autolink" rule).
const TRAILING: &[char] = &['?', '!', '.', ',', ':', '*', '_', '~', '\'', '"'];

/// Splits plain text into text and bare-URL links (`https://x.y`, `http://x.y`,
/// `www.x.y`). The URL ends at whitespace or `<`; trailing punctuation and an
/// unbalanced `)` are not part of it, so "(see https://a.dev)." works.
pub fn autolink(text: &str) -> Vec<Inline> {
    let mut out: Vec<Inline> = Vec::new();
    let mut plain_from = 0;
    let mut cursor = 0;
    while cursor < text.len() {
        let Some(rel) = text[cursor..].find(['h', 'w']) else {
            break;
        };
        let at = cursor + rel;
        cursor = at + 1;
        let rest = &text[at..];
        if !(rest.starts_with("http://")
            || rest.starts_with("https://")
            || rest.starts_with("www."))
        {
            continue;
        }
        let boundary = text[..at]
            .chars()
            .next_back()
            .is_none_or(|c| c.is_whitespace() || matches!(c, '*' | '_' | '~' | '('));
        if !boundary {
            continue;
        }
        let end = rest
            .find(|c: char| c.is_whitespace() || c == '<')
            .unwrap_or(rest.len());
        let mut raw = &rest[..end];
        loop {
            let before = raw.len();
            raw = raw.trim_end_matches(TRAILING);
            if raw.ends_with(')') && raw.matches(')').count() > raw.matches('(').count() {
                raw = &raw[..raw.len() - 1];
            }
            if raw.len() == before {
                break;
            }
        }
        let href = if let Some(domain) = raw.strip_prefix("www.") {
            // "www." alone, or without a dot after it, is not a domain.
            if domain.is_empty() || !domain.contains('.') && domain.len() < 2 {
                continue;
            }
            format!("https://{raw}")
        } else {
            raw.to_owned()
        };
        if !is_http_url(&href) {
            continue;
        }
        if plain_from < at {
            out.push(Inline::Text(text[plain_from..at].to_owned()));
        }
        out.push(Inline::Link {
            href,
            children: vec![Inline::Text(raw.to_owned())],
        });
        plain_from = at + raw.len();
        cursor = plain_from;
    }
    if plain_from < text.len() {
        out.push(Inline::Text(text[plain_from..].to_owned()));
    }
    out
}

// ---------------------------------------------------------------------------
// Plain text (Terminal preview, alt text)
// ---------------------------------------------------------------------------

fn inline_text(nodes: &[Inline], out: &mut String) {
    for node in nodes {
        match node {
            Inline::Text(t) | Inline::Code(t) => out.push_str(t),
            Inline::Break => out.push(' '),
            Inline::Emph(c) | Inline::Strong(c) | Inline::Strike(c) => inline_text(c, out),
            Inline::Link { children, .. } => inline_text(children, out),
            Inline::Image { alt, .. } => out.push_str(alt),
        }
    }
}

fn block_text(blocks: &[Block], out: &mut String) {
    let gap = |out: &mut String| {
        if !out.is_empty() && !out.ends_with(' ') {
            out.push(' ');
        }
    };
    for block in blocks {
        gap(out);
        match block {
            Block::Paragraph(c) | Block::Tight(c) | Block::Heading { content: c, .. } => {
                inline_text(c, out);
            }
            Block::Quote { blocks, .. } => block_text(blocks, out),
            Block::List { items, .. } => {
                for item in items {
                    gap(out);
                    block_text(&item.blocks, out);
                }
            }
            Block::Code { code, .. } => out.push_str(&code.replace('\n', " ")),
            Block::Rule => {}
            Block::Table { head, rows, .. } => {
                for cell in head.iter().chain(rows.iter().flatten()) {
                    gap(out);
                    inline_text(cell, out);
                }
            }
        }
    }
}

/// The text of `source` without any Markdown syntax, on one line, cut to
/// `max_chars` characters (with `…` if cut). For places that cannot show
/// formatting, like the Terminal preview.
pub fn plain_text(source: &str, max_chars: usize) -> String {
    let mut out = String::new();
    block_text(&parse(source), &mut out);
    let flat = out.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= max_chars {
        return flat;
    }
    let mut cut: String = flat.chars().take(max_chars).collect();
    cut.push('…');
    cut
}

// ---------------------------------------------------------------------------
// Event stream -> tree
// ---------------------------------------------------------------------------

enum Container {
    Root,
    Quote(Option<Alert>),
    Item,
}

enum SpanKind {
    Emph,
    Strong,
    Strike,
    Link(Option<String>),
    Image(Option<String>),
    /// Beyond [`MAX_INLINE_DEPTH`]: the tag is dropped, its content kept.
    Flat,
}

#[derive(Default)]
struct TableState {
    align: Vec<Align>,
    head: Vec<Cell>,
    rows: Vec<Vec<Cell>>,
    row: Vec<Cell>,
}

enum Open {
    Blocks {
        kind: Container,
        blocks: Vec<Block>,
        /// Inline text that sits directly in a container (tight list items).
        pending: Vec<Inline>,
        checked: Option<bool>,
    },
    List {
        start: Option<u64>,
        items: Vec<Item>,
    },
    Paragraph(Vec<Inline>),
    Heading(u8, Vec<Inline>),
    Code {
        lang: Option<String>,
        code: String,
    },
    Table(TableState),
    Cell(Vec<Inline>),
    Span(SpanKind, Vec<Inline>),
}

struct Builder {
    stack: Vec<Open>,
    /// One entry per quote / list / item start: `true` if it was opened for
    /// real, `false` if it was flattened for being too deep.
    marks: Vec<bool>,
    /// Consecutive text events are joined before they are looked at, so a URL
    /// the parser split at an `_` is still one URL.
    text: String,
}

fn align(a: Alignment) -> Align {
    match a {
        Alignment::None => Align::Default,
        Alignment::Left => Align::Left,
        Alignment::Center => Align::Center,
        Alignment::Right => Align::Right,
    }
}

fn alert(kind: BlockQuoteKind) -> Alert {
    match kind {
        BlockQuoteKind::Note => Alert::Note,
        BlockQuoteKind::Tip => Alert::Tip,
        BlockQuoteKind::Important => Alert::Important,
        BlockQuoteKind::Warning => Alert::Warning,
        BlockQuoteKind::Caution => Alert::Caution,
    }
}

/// The language label of a fenced block: first word of the info string,
/// restricted to a short, harmless alphabet.
fn code_lang(info: &str) -> Option<String> {
    let word = info.split(|c: char| c.is_whitespace() || c == ',').next()?;
    let lang: String = word
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '#' | '_' | '.'))
        .take(16)
        .collect();
    (!lang.is_empty()).then_some(lang)
}

impl Builder {
    fn new() -> Self {
        Self {
            stack: vec![Open::Blocks {
                kind: Container::Root,
                blocks: Vec::new(),
                pending: Vec::new(),
                checked: None,
            }],
            marks: Vec::new(),
            text: String::new(),
        }
    }

    fn block_depth(&self) -> usize {
        self.stack
            .iter()
            .filter(|o| {
                matches!(
                    o,
                    Open::List { .. }
                        | Open::Blocks {
                            kind: Container::Quote(_),
                            ..
                        }
                )
            })
            .count()
    }

    fn span_depth(&self) -> usize {
        self.stack
            .iter()
            .filter(|o| matches!(o, Open::Span(..)))
            .count()
    }

    /// Inside a link or image: bare URLs must not become links again.
    fn in_link(&self) -> bool {
        self.stack
            .iter()
            .any(|o| matches!(o, Open::Span(SpanKind::Link(_) | SpanKind::Image(_), _)))
    }

    /// Where inline content goes right now.
    fn sink(&mut self) -> Option<&mut Vec<Inline>> {
        match self.stack.last_mut()? {
            Open::Paragraph(v) | Open::Heading(_, v) | Open::Cell(v) | Open::Span(_, v) => Some(v),
            Open::Blocks { pending, .. } => Some(pending),
            Open::List { .. } | Open::Code { .. } | Open::Table(_) => None,
        }
    }

    fn push_inline(&mut self, node: Inline) {
        if let Some(sink) = self.sink() {
            sink.push(node);
        }
    }

    /// Moves tight text that sits directly in the current container into a block.
    fn flush_pending(&mut self) {
        if let Some(Open::Blocks {
            blocks, pending, ..
        }) = self.stack.last_mut()
        {
            if !pending.is_empty() {
                blocks.push(Block::Tight(std::mem::take(pending)));
            }
        }
    }

    /// Adds a finished block to the nearest container.
    fn push_block(&mut self, block: Block) {
        self.flush_pending();
        for open in self.stack.iter_mut().rev() {
            if let Open::Blocks { blocks, .. } = open {
                blocks.push(block);
                return;
            }
        }
    }

    fn flush_text(&mut self) {
        if self.text.is_empty() {
            return;
        }
        let text = std::mem::take(&mut self.text);
        if let Some(Open::Code { code, .. }) = self.stack.last_mut() {
            code.push_str(&text);
        } else if self.in_link() {
            self.push_inline(Inline::Text(text));
        } else {
            for node in autolink(&text) {
                self.push_inline(node);
            }
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Paragraph => self.stack.push(Open::Paragraph(Vec::new())),
            Tag::Heading { level, .. } => {
                self.stack.push(Open::Heading(level as u8, Vec::new()));
            }
            Tag::BlockQuote(kind) => {
                let real = self.block_depth() < MAX_BLOCK_DEPTH;
                self.marks.push(real);
                // Text before a nested container must not run into the text
                // inside it, also when that container is flattened.
                self.flush_pending();
                if real {
                    self.stack.push(Open::Blocks {
                        kind: Container::Quote(kind.map(alert)),
                        blocks: Vec::new(),
                        pending: Vec::new(),
                        checked: None,
                    });
                }
            }
            Tag::CodeBlock(kind) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(info) => code_lang(&info),
                    CodeBlockKind::Indented => None,
                };
                self.stack.push(Open::Code {
                    lang,
                    code: String::new(),
                });
            }
            Tag::List(start) => {
                let real = self.block_depth() < MAX_BLOCK_DEPTH;
                self.marks.push(real);
                self.flush_pending();
                if real {
                    self.stack.push(Open::List {
                        start,
                        items: Vec::new(),
                    });
                }
            }
            Tag::Item => {
                let real = matches!(self.stack.last(), Some(Open::List { .. }));
                self.marks.push(real);
                self.flush_pending();
                if real {
                    self.stack.push(Open::Blocks {
                        kind: Container::Item,
                        blocks: Vec::new(),
                        pending: Vec::new(),
                        checked: None,
                    });
                }
            }
            Tag::Table(aligns) => self.stack.push(Open::Table(TableState {
                align: aligns.into_iter().map(align).collect(),
                ..TableState::default()
            })),
            Tag::TableCell => self.stack.push(Open::Cell(Vec::new())),
            Tag::Emphasis => self.span(SpanKind::Emph),
            Tag::Strong => self.span(SpanKind::Strong),
            Tag::Strikethrough => self.span(SpanKind::Strike),
            Tag::Link {
                link_type,
                dest_url,
                ..
            } => {
                // `<me@a.dev>`: the parser hands over the bare address.
                let href = if link_type == LinkType::Email {
                    safe_href(&format!("mailto:{dest_url}"))
                } else {
                    safe_href(&dest_url)
                };
                self.span(SpanKind::Link(href));
            }
            Tag::Image { dest_url, .. } => self.span(SpanKind::Image(safe_href(&dest_url))),
            // Raw HTML blocks only group `Html` events, which are shown as text;
            // table head/row only matter when they end. Anything else is off.
            _ => {}
        }
    }

    fn span(&mut self, kind: SpanKind) {
        let kind = if self.span_depth() >= MAX_INLINE_DEPTH {
            SpanKind::Flat
        } else {
            kind
        };
        self.stack.push(Open::Span(kind, Vec::new()));
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => {
                if let Some(Open::Paragraph(content)) =
                    self.pop_if(|o| matches!(o, Open::Paragraph(_)))
                {
                    if !content.is_empty() {
                        self.push_block(Block::Paragraph(content));
                    }
                }
            }
            TagEnd::Heading(_) => {
                if let Some(Open::Heading(level, content)) =
                    self.pop_if(|o| matches!(o, Open::Heading(..)))
                {
                    self.push_block(Block::Heading { level, content });
                }
            }
            TagEnd::BlockQuote(_) => {
                if self.marks.pop() == Some(true) {
                    self.flush_pending();
                    if let Some(Open::Blocks {
                        kind: Container::Quote(alert),
                        blocks,
                        ..
                    }) = self.pop_if(|o| {
                        matches!(
                            o,
                            Open::Blocks {
                                kind: Container::Quote(_),
                                ..
                            }
                        )
                    }) {
                        self.push_block(Block::Quote { alert, blocks });
                    }
                }
            }
            TagEnd::CodeBlock => {
                if let Some(Open::Code { lang, mut code }) =
                    self.pop_if(|o| matches!(o, Open::Code { .. }))
                {
                    if code.ends_with('\n') {
                        code.pop();
                    }
                    self.push_block(Block::Code { lang, code });
                }
            }
            TagEnd::List(_) => {
                if self.marks.pop() == Some(true) {
                    if let Some(Open::List { start, items }) =
                        self.pop_if(|o| matches!(o, Open::List { .. }))
                    {
                        self.push_block(Block::List { start, items });
                    }
                }
            }
            TagEnd::Item => {
                if self.marks.pop() == Some(true) {
                    self.flush_pending();
                    if let Some(Open::Blocks {
                        blocks, checked, ..
                    }) = self.pop_if(|o| {
                        matches!(
                            o,
                            Open::Blocks {
                                kind: Container::Item,
                                ..
                            }
                        )
                    }) {
                        if let Some(Open::List { items, .. }) = self.stack.last_mut() {
                            items.push(Item { checked, blocks });
                        }
                    }
                } else {
                    // Flattened item: keep its text apart from the next item's.
                    self.flush_pending();
                }
            }
            TagEnd::Table => {
                if let Some(Open::Table(t)) = self.pop_if(|o| matches!(o, Open::Table(_))) {
                    self.push_block(Block::Table {
                        align: t.align,
                        head: t.head,
                        rows: t.rows,
                    });
                }
            }
            TagEnd::TableHead => {
                if let Some(Open::Table(t)) = self.stack.last_mut() {
                    t.head = std::mem::take(&mut t.row);
                }
            }
            TagEnd::TableRow => {
                if let Some(Open::Table(t)) = self.stack.last_mut() {
                    let row = std::mem::take(&mut t.row);
                    t.rows.push(row);
                }
            }
            TagEnd::TableCell => {
                if let Some(Open::Cell(cell)) = self.pop_if(|o| matches!(o, Open::Cell(_))) {
                    if let Some(Open::Table(t)) = self.stack.last_mut() {
                        t.row.push(cell);
                    }
                }
            }
            TagEnd::Emphasis
            | TagEnd::Strong
            | TagEnd::Strikethrough
            | TagEnd::Link
            | TagEnd::Image => {
                if let Some(Open::Span(kind, children)) =
                    self.pop_if(|o| matches!(o, Open::Span(..)))
                {
                    self.close_span(kind, children);
                }
            }
            _ => {}
        }
    }

    /// Pops the top of the stack only if `is` says it is the expected kind, so
    /// an unbalanced event sequence cannot tear the wrong node off the stack.
    fn pop_if(&mut self, is: impl Fn(&Open) -> bool) -> Option<Open> {
        if self.stack.len() > 1 && self.stack.last().is_some_and(is) {
            self.stack.pop()
        } else {
            None
        }
    }

    fn close_span(&mut self, kind: SpanKind, children: Vec<Inline>) {
        match kind {
            SpanKind::Emph => self.push_inline(Inline::Emph(children)),
            SpanKind::Strong => self.push_inline(Inline::Strong(children)),
            SpanKind::Strike => self.push_inline(Inline::Strike(children)),
            SpanKind::Link(Some(href)) => self.push_inline(Inline::Link { href, children }),
            SpanKind::Link(None) | SpanKind::Flat => {
                for node in children {
                    self.push_inline(node);
                }
            }
            SpanKind::Image(href) => {
                let mut alt = String::new();
                inline_text(&children, &mut alt);
                self.push_inline(Inline::Image { alt, href });
            }
        }
    }

    fn event(&mut self, event: Event<'_>) {
        if let Event::Text(t) = &event {
            self.text.push_str(t);
            return;
        }
        self.flush_text();
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Code(c) => self.push_inline(Inline::Code(c.into_string())),
            // Raw HTML is text. It is shown, never interpreted.
            Event::Html(h) | Event::InlineHtml(h) => {
                self.push_inline(Inline::Text(h.into_string()))
            }
            Event::SoftBreak | Event::HardBreak => self.push_inline(Inline::Break),
            Event::Rule => self.push_block(Block::Rule),
            Event::TaskListMarker(done) => {
                // The marker may come before or inside the item's first paragraph.
                for open in self.stack.iter_mut().rev() {
                    if let Open::Blocks {
                        kind: Container::Item,
                        checked,
                        ..
                    } = open
                    {
                        *checked = Some(done);
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    fn finish(mut self) -> Vec<Block> {
        self.flush_text();
        // Balanced input leaves only the root. If it is ever not, whatever is
        // still open is dropped rather than guessed at.
        self.stack.truncate(1);
        self.flush_pending();
        match self.stack.pop() {
            Some(Open::Blocks { blocks, .. }) => blocks,
            _ => Vec::new(),
        }
    }
}

/// Folds parser events into blocks. Public so tests can feed unusual streams.
pub fn build<'a>(events: impl Iterator<Item = Event<'a>>) -> Vec<Block> {
    let mut builder = Builder::new();
    for event in events {
        builder.event(event);
    }
    builder.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::{CowStr, HeadingLevel};

    fn text(s: &str) -> Inline {
        Inline::Text(s.to_owned())
    }

    fn para(nodes: Vec<Inline>) -> Block {
        Block::Paragraph(nodes)
    }

    #[test]
    fn inline_formatting_becomes_nodes() {
        assert_eq!(
            parse("a **b** _c_ ~~d~~ `e`"),
            [para(vec![
                text("a "),
                Inline::Strong(vec![text("b")]),
                text(" "),
                Inline::Emph(vec![text("c")]),
                text(" "),
                Inline::Strike(vec![text("d")]),
                text(" "),
                Inline::Code("e".into()),
            ])]
        );
    }

    #[test]
    fn a_single_newline_is_a_line_break_like_in_a_comment() {
        assert_eq!(
            parse("one\ntwo\\\nthree"),
            [para(vec![
                text("one"),
                Inline::Break,
                text("two"),
                Inline::Break,
                text("three")
            ])]
        );
    }

    #[test]
    fn headings_keep_their_level() {
        let blocks = parse("# a\n\n### b");
        assert_eq!(
            blocks,
            [
                Block::Heading {
                    level: 1,
                    content: vec![text("a")]
                },
                Block::Heading {
                    level: 3,
                    content: vec![text("b")]
                },
            ]
        );
    }

    #[test]
    fn tight_lists_have_tight_text_and_loose_lists_have_paragraphs() {
        let tight = parse("- a\n- b");
        let Block::List { start, items } = &tight[0] else {
            panic!("{tight:?}")
        };
        assert_eq!(*start, None);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].blocks, [Block::Tight(vec![text("a")])]);

        let loose = parse("- a\n\n- b");
        let Block::List { items, .. } = &loose[0] else {
            panic!("{loose:?}")
        };
        assert_eq!(items[0].blocks, [para(vec![text("a")])]);
    }

    #[test]
    fn ordered_lists_remember_where_they_start_and_lists_nest() {
        let blocks = parse("3. a\n   - x\n   - y\n4. b");
        let Block::List { start, items } = &blocks[0] else {
            panic!("{blocks:?}")
        };
        assert_eq!(*start, Some(3));
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].blocks.len(), 2, "text, then the nested list");
        assert!(matches!(items[0].blocks[1], Block::List { .. }));
    }

    #[test]
    fn task_lists_are_checked_or_not_tight_or_loose() {
        for src in ["- [x] done\n- [ ] todo", "- [x] done\n\n- [ ] todo"] {
            let blocks = parse(src);
            let Block::List { items, .. } = &blocks[0] else {
                panic!("{blocks:?}")
            };
            assert_eq!(
                (items[0].checked, items[1].checked),
                (Some(true), Some(false)),
                "{src}"
            );
        }
        let plain = parse("- a");
        let Block::List { items, .. } = &plain[0] else {
            panic!()
        };
        assert_eq!(items[0].checked, None);
    }

    #[test]
    fn code_blocks_keep_their_text_and_a_safe_language_label() {
        let blocks = parse("```rust,ignore\nfn main() {}\n  x\n```");
        assert_eq!(
            blocks,
            [Block::Code {
                lang: Some("rust".into()),
                code: "fn main() {}\n  x".into()
            }]
        );
        let odd = parse("```<script>alert(1)</script>\nx\n```");
        let Block::Code { lang, .. } = &odd[0] else {
            panic!("{odd:?}")
        };
        assert_eq!(lang.as_deref(), Some("scriptalert1scri"));
        assert!(matches!(
            &parse("    indented")[0],
            Block::Code { lang: None, code } if code == "indented"
        ));
    }

    #[test]
    fn markdown_inside_code_is_not_interpreted_and_urls_are_not_linked() {
        let blocks = parse("```\n**x** https://a.dev\n```");
        assert_eq!(
            blocks,
            [Block::Code {
                lang: None,
                code: "**x** https://a.dev".into()
            }]
        );
        assert_eq!(
            parse("`https://a.dev`"),
            [para(vec![Inline::Code("https://a.dev".into())])]
        );
    }

    #[test]
    fn tables_have_alignment_a_head_and_rows() {
        let blocks = parse("| a | b |\n|:--|--:|\n| 1 | 2 |\n| 3 | 4 |");
        let Block::Table { align, head, rows } = &blocks[0] else {
            panic!("{blocks:?}")
        };
        assert_eq!(align, &[Align::Left, Align::Right]);
        assert_eq!(head, &[vec![text("a")], vec![text("b")]]);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1], [vec![text("3")], vec![text("4")]]);
    }

    #[test]
    fn quotes_and_github_alerts() {
        let blocks = parse("> hi\n\n> [!WARNING]\n> careful");
        assert!(matches!(&blocks[0], Block::Quote { alert: None, .. }));
        let Block::Quote { alert, blocks } = &blocks[1] else {
            panic!()
        };
        assert_eq!(*alert, Some(Alert::Warning));
        assert_eq!(blocks, &[para(vec![text("careful")])]);
    }

    #[test]
    fn rules_become_rules() {
        assert_eq!(parse("a\n\n---\n\nb").len(), 3);
        assert_eq!(parse("a\n\n---\n\nb")[1], Block::Rule);
    }

    #[test]
    fn raw_html_is_shown_as_text_and_never_becomes_structure() {
        for src in [
            "<script>alert(1)</script>",
            "<img src=x onerror=alert(1)>",
            "text <b>bold</b> <a href=\"javascript:alert(1)\">x</a>",
            "<div>\n<p>hi</p>\n</div>",
            "<!-- hidden -->",
        ] {
            let blocks = parse(src);
            let mut shown = String::new();
            block_text(&blocks, &mut shown);
            // Same characters, so the reader sees what was typed...
            assert!(shown.contains('<'), "{src}: {shown}");
            // ...and the tree has only text in it: no link, no image, no code.
            fn only_text(nodes: &[Inline]) -> bool {
                nodes
                    .iter()
                    .all(|n| matches!(n, Inline::Text(_) | Inline::Break))
            }
            for block in &blocks {
                match block {
                    Block::Paragraph(c) | Block::Tight(c) => assert!(only_text(c), "{src}"),
                    other => panic!("{src}: unexpected {other:?}"),
                }
            }
        }
    }

    #[test]
    fn only_web_and_mail_links_survive() {
        for ok in [
            "https://a.dev",
            "http://a.dev/x?y=1#z",
            "mailto:me@a.dev",
            "  https://a.dev  ",
        ] {
            assert_eq!(safe_href(ok).as_deref(), Some(ok.trim()), "{ok}");
        }
        for bad in [
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "data:text/html,<script>",
            "vbscript:x",
            "file:///etc/passwd",
            "//evil.example",
            "/relative",
            "relative.html",
            "#anchor",
            "HTTPS://A.DEV",
            "https://a b",
            "https://",
            "mailto:",
            "mailto:a@b@c",
            "mailto:me@a.dev?subject=x",
            "mailto:\u{202E}@a.dev",
            "",
        ] {
            assert_eq!(safe_href(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn unsafe_link_targets_keep_the_text_and_drop_the_link() {
        assert_eq!(
            parse("[click](javascript:alert(1))"),
            [para(vec![text("click")])]
        );
        assert_eq!(
            parse("[click](https://a.dev)"),
            [para(vec![Inline::Link {
                href: "https://a.dev".into(),
                children: vec![text("click")]
            }])]
        );
        assert_eq!(
            parse("[**bold** link](/local)"),
            [para(vec![
                Inline::Strong(vec![text("bold")]),
                text(" link")
            ])]
        );
    }

    #[test]
    fn images_are_links_never_pictures() {
        assert_eq!(
            parse("![a cat](https://a.dev/cat.png)"),
            [para(vec![Inline::Image {
                alt: "a cat".into(),
                href: Some("https://a.dev/cat.png".into())
            }])]
        );
        assert_eq!(
            parse("![x](javascript:alert(1))"),
            [para(vec![Inline::Image {
                alt: "x".into(),
                href: None
            }])]
        );
    }

    #[test]
    fn bare_urls_become_links_with_sensible_edges() {
        let url = |href: &str, shown: &str| Inline::Link {
            href: href.into(),
            children: vec![text(shown)],
        };
        assert_eq!(
            autolink("see https://a.dev/x."),
            [
                text("see "),
                url("https://a.dev/x", "https://a.dev/x"),
                text(".")
            ]
        );
        assert_eq!(
            autolink("(https://a.dev/x)"),
            [
                text("("),
                url("https://a.dev/x", "https://a.dev/x"),
                text(")")
            ]
        );
        assert_eq!(
            autolink("https://en.wikipedia.org/wiki/Rust_(language)"),
            [url(
                "https://en.wikipedia.org/wiki/Rust_(language)",
                "https://en.wikipedia.org/wiki/Rust_(language)"
            )]
        );
        assert_eq!(
            autolink("go to www.a.dev, ok"),
            [
                text("go to "),
                url("https://www.a.dev", "www.a.dev"),
                text(", ok")
            ]
        );
        assert_eq!(
            autolink("http://a.dev?q=1&r=2!"),
            [
                url("http://a.dev?q=1&r=2", "http://a.dev?q=1&r=2"),
                text("!")
            ]
        );
    }

    #[test]
    fn things_that_only_look_like_urls_stay_text() {
        for src in [
            "nohttps://a.dev",
            "xwww.a.dev",
            "https://",
            "http:// a",
            "www.",
            "HTTPS://A.DEV",
            "ftp://a.dev",
            "javascript:alert(1)",
            "https://a b",
        ] {
            let nodes = autolink(src);
            let links = nodes
                .iter()
                .filter(|n| matches!(n, Inline::Link { .. }))
                .count();
            let expected = usize::from(src == "https://a b");
            assert_eq!(links, expected, "{src}: {nodes:?}");
        }
    }

    #[test]
    fn a_url_the_parser_splits_at_underscores_is_still_one_link() {
        let blocks = parse("https://a.dev/a_b_c_d");
        let Block::Paragraph(nodes) = &blocks[0] else {
            panic!("{blocks:?}")
        };
        assert_eq!(nodes.len(), 1, "{nodes:?}");
        assert!(matches!(&nodes[0], Inline::Link { href, .. } if href == "https://a.dev/a_b_c_d"));
    }

    #[test]
    fn urls_inside_links_are_not_linked_twice() {
        let blocks = parse("[https://a.dev](https://b.dev)");
        let Block::Paragraph(nodes) = &blocks[0] else {
            panic!()
        };
        assert_eq!(nodes.len(), 1);
        let Inline::Link { href, children } = &nodes[0] else {
            panic!("{nodes:?}")
        };
        assert_eq!(href, "https://b.dev");
        assert_eq!(children, &[text("https://a.dev")]);
    }

    #[test]
    fn angle_bracket_autolinks_and_emails_work() {
        let blocks = parse("<https://a.dev> <me@a.dev>");
        let Block::Paragraph(nodes) = &blocks[0] else {
            panic!()
        };
        assert!(matches!(&nodes[0], Inline::Link { href, .. } if href == "https://a.dev"));
        assert!(matches!(&nodes[2], Inline::Link { href, .. } if href == "mailto:me@a.dev"));
    }

    fn depth(blocks: &[Block]) -> usize {
        blocks
            .iter()
            .map(|b| match b {
                Block::Quote { blocks, .. } => 1 + depth(blocks),
                Block::List { items, .. } => {
                    1 + items.iter().map(|i| depth(&i.blocks)).max().unwrap_or(0)
                }
                _ => 0,
            })
            .max()
            .unwrap_or(0)
    }

    fn flat_text(blocks: &[Block]) -> String {
        let mut out = String::new();
        block_text(blocks, &mut out);
        out
    }

    #[test]
    fn deeply_nested_quotes_are_flattened_but_keep_their_text() {
        let src = format!("{}deep", "> ".repeat(500));
        let blocks = parse(&src);
        assert!(depth(&blocks) <= MAX_BLOCK_DEPTH, "{}", depth(&blocks));
        assert!(flat_text(&blocks).contains("deep"));
    }

    #[test]
    fn deeply_nested_lists_are_flattened_but_keep_their_text() {
        let mut src = String::new();
        for level in 0..60 {
            src.push_str(&format!("{}- item{level}\n", "  ".repeat(level)));
        }
        let blocks = parse(&src);
        assert!(depth(&blocks) <= MAX_BLOCK_DEPTH, "{}", depth(&blocks));
        let shown = flat_text(&blocks);
        for level in [0, 5, 30, 59] {
            assert!(shown.contains(&format!("item{level}")), "item{level}");
        }
    }

    #[test]
    fn flattened_list_items_do_not_run_together() {
        let mut src = String::new();
        for level in 0..12 {
            src.push_str(&format!("{}- w{level}\n", "  ".repeat(level)));
        }
        let shown = flat_text(&parse(&src));
        assert!(shown.contains("w10 w11"), "{shown}");
        assert!(!shown.contains("w10w11"), "{shown}");
    }

    #[test]
    fn deeply_nested_inline_spans_are_capped() {
        let n = 10_000;
        let mut events: Vec<Event<'static>> = Vec::new();
        events.push(Event::Start(Tag::Paragraph));
        for _ in 0..n {
            events.push(Event::Start(Tag::Emphasis));
        }
        events.push(Event::Text(CowStr::from("core")));
        for _ in 0..n {
            events.push(Event::End(TagEnd::Emphasis));
        }
        events.push(Event::End(TagEnd::Paragraph));
        let blocks = build(events.into_iter());

        fn inline_depth(nodes: &[Inline]) -> usize {
            nodes
                .iter()
                .map(|n| match n {
                    Inline::Emph(c) | Inline::Strong(c) | Inline::Strike(c) => 1 + inline_depth(c),
                    Inline::Link { children, .. } => 1 + inline_depth(children),
                    _ => 0,
                })
                .max()
                .unwrap_or(0)
        }
        let Block::Paragraph(nodes) = &blocks[0] else {
            panic!()
        };
        assert!(inline_depth(nodes) <= MAX_INLINE_DEPTH);
        assert!(flat_text(&blocks).contains("core"));
    }

    #[test]
    fn unbalanced_event_streams_never_panic() {
        use Event::*;
        let streams: Vec<Vec<Event<'static>>> = vec![
            vec![
                End(TagEnd::Paragraph),
                End(TagEnd::List(true)),
                End(TagEnd::Item),
            ],
            vec![Start(Tag::List(None)), Start(Tag::Item), Text("x".into())],
            vec![Start(Tag::Table(vec![Alignment::None])), Text("x".into())],
            vec![
                End(TagEnd::TableCell),
                End(TagEnd::TableRow),
                End(TagEnd::TableHead),
            ],
            vec![TaskListMarker(true), Rule, Code("c".into()), SoftBreak],
            vec![
                Start(Tag::Heading {
                    level: HeadingLevel::H2,
                    id: None,
                    classes: vec![],
                    attrs: vec![],
                }),
                End(TagEnd::Emphasis),
                End(TagEnd::Heading(HeadingLevel::H2)),
            ],
            vec![Start(Tag::Emphasis), End(TagEnd::Strong), End(TagEnd::Link)],
            vec![],
        ];
        for events in streams {
            let _ = build(events.into_iter());
        }
    }

    #[test]
    fn arbitrary_markdown_like_noise_never_panics_and_stays_shallow() {
        // A small deterministic generator over the characters that matter.
        let alphabet: Vec<&str> = vec![
            "*",
            "_",
            "~",
            "`",
            "```",
            "#",
            ">",
            "-",
            "+",
            "1.",
            "[",
            "]",
            "(",
            ")",
            "!",
            "|",
            "\n",
            "\n\n",
            " ",
            "  ",
            "    ",
            "\t",
            "a",
            "b",
            "<",
            ">",
            "&",
            "\\",
            "http://x.y",
            "www.a.b",
            "[ ]",
            "[x]",
            ":",
            "---",
            "|---|",
        ];
        let mut seed: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut next = || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 33) as usize
        };
        for _ in 0..400 {
            let len = 1 + next() % 300;
            let src: String = (0..len)
                .map(|_| alphabet[next() % alphabet.len()])
                .collect();
            let blocks = parse(&src);
            assert!(depth(&blocks) <= MAX_BLOCK_DEPTH, "{src:?}");
            let _ = plain_text(&src, 80);
        }
    }

    #[test]
    fn code_labels_are_short_and_plain() {
        assert_eq!(code_lang("rust"), Some("rust".into()));
        assert_eq!(code_lang("c++ extra"), Some("c++".into()));
        assert_eq!(code_lang(""), None);
        assert_eq!(code_lang("   "), None);
        assert_eq!(code_lang(&"a".repeat(40)).map(|l| l.len()), Some(16));
    }

    #[test]
    fn plain_text_drops_syntax_and_cuts_long_entries() {
        assert_eq!(
            plain_text("# Title\n\nSome **bold** and [a link](https://a.dev).", 200),
            "Title Some bold and a link."
        );
        assert_eq!(plain_text("- a\n- b\n\n`c`", 200), "a b c");
        assert_eq!(
            plain_text("![alt text](https://a.dev/i.png)", 200),
            "alt text"
        );
        assert_eq!(
            plain_text("| a | b |\n|---|---|\n| 1 | 2 |", 200),
            "a b 1 2"
        );
        assert_eq!(plain_text("", 10), "");
        assert_eq!(plain_text("abcdefghij", 5), "abcde…");
        assert_eq!(plain_text("abcde", 5), "abcde");
        assert_eq!(plain_text("line one\nline two", 200), "line one line two");
    }

    #[test]
    fn every_block_kind_survives_a_realistic_readme() {
        let readme = "\
# Hello!

I'm **Moch**. Things I like:

- Rust
- [x] NixOS
  - flakes

> [!TIP]
> Try `nix develop`.

| tool | verdict |
|------|---------|
| dx   | fun     |

```rust
fn main() {}
```

---

Visit https://tquilla.is-a.dev ~~now~~ later.";
        let blocks = parse(readme);
        let kinds: Vec<&str> = blocks
            .iter()
            .map(|b| match b {
                Block::Heading { .. } => "h",
                Block::Paragraph(_) => "p",
                Block::List { .. } => "l",
                Block::Quote { .. } => "q",
                Block::Table { .. } => "t",
                Block::Code { .. } => "c",
                Block::Rule => "r",
                Block::Tight(_) => "tight",
            })
            .collect();
        assert_eq!(kinds, ["h", "p", "l", "q", "t", "c", "r", "p"]);
    }
}
