//! A tiny, forgiving tokenizer for the HTML an entry may contain.
//!
//! It never builds markup. It only splits a chunk of raw HTML into text, open
//! tags (with their attributes) and close tags; the builder in the parent
//! module then decides, from a fixed allow-list, which of those mean anything.
//! Everything it does not understand stays text, so it cannot fail.

/// Most attributes kept per tag. The rest are ignored (the input is tiny
/// anyway; this only bounds the work for a hostile tag).
const MAX_ATTRS: usize = 8;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Text(String),
    /// Lower-case tag name and its attributes (names lower-case, values with
    /// entities decoded).
    Open {
        name: String,
        attrs: Vec<(String, String)>,
    },
    Close(String),
}

/// The value of attribute `name`, if present (an attribute without a value,
/// like `open`, is `Some("")`).
pub fn attr<'a>(attrs: &'a [(String, String)], name: &str) -> Option<&'a str> {
    attrs
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, v)| v.as_str())
}

/// Splits `src` into tokens. Comments are dropped; a `<` that does not start
/// a well-formed tag is ordinary text.
pub fn tokenize(src: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let bytes = src.as_bytes();
    let mut text_from = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            i += 1;
            continue;
        }
        let rest = &src[i..];
        let found = if let Some(inside) = rest.strip_prefix("<!--") {
            let end = inside.find("-->").map_or(rest.len(), |e| 4 + e + 3);
            Some((end, None))
        } else {
            parse_tag(rest).map(|(len, token)| (len, Some(token)))
        };
        match found {
            Some((len, token)) => {
                if text_from < i {
                    out.push(Token::Text(src[text_from..i].to_owned()));
                }
                out.extend(token);
                i += len;
                text_from = i;
            }
            None => i += 1,
        }
    }
    if text_from < src.len() {
        out.push(Token::Text(src[text_from..].to_owned()));
    }
    out
}

fn is_space(b: u8) -> bool {
    b.is_ascii_whitespace()
}

/// Parses one tag at the start of `rest` (which begins with `<`). Returns its
/// length in bytes, or `None` if it is not a well-formed tag.
fn parse_tag(rest: &str) -> Option<(usize, Token)> {
    let after = &rest[1..];
    let (closing, body) = match after.strip_prefix('/') {
        Some(b) => (true, b),
        None => (false, after),
    };
    let offset = rest.len() - body.len();
    let b = body.as_bytes();
    if !b.first().is_some_and(u8::is_ascii_alphabetic) {
        return None;
    }
    let mut pos = b
        .iter()
        .take_while(|c| c.is_ascii_alphanumeric() || **c == b'-')
        .count();
    let name = body[..pos].to_ascii_lowercase();

    let mut attrs: Vec<(String, String)> = Vec::new();
    loop {
        while pos < b.len() && is_space(b[pos]) {
            pos += 1;
        }
        match b.get(pos)? {
            b'>' => {
                pos += 1;
                break;
            }
            b'/' => {
                pos += 1;
                continue;
            }
            _ => {}
        }
        let start = pos;
        while pos < b.len() && !is_space(b[pos]) && !matches!(b[pos], b'=' | b'>' | b'/') {
            pos += 1;
        }
        if start == pos {
            // A stray `=`.
            pos += 1;
            continue;
        }
        let attr_name = body[start..pos].to_ascii_lowercase();
        while pos < b.len() && is_space(b[pos]) {
            pos += 1;
        }
        let mut value = String::new();
        if b.get(pos) == Some(&b'=') {
            pos += 1;
            while pos < b.len() && is_space(b[pos]) {
                pos += 1;
            }
            match b.get(pos)? {
                quote @ (b'"' | b'\'') => {
                    pos += 1;
                    let from = pos;
                    while pos < b.len() && b[pos] != *quote {
                        pos += 1;
                    }
                    if pos >= b.len() {
                        return None;
                    }
                    value = body[from..pos].to_owned();
                    pos += 1;
                }
                _ => {
                    let from = pos;
                    while pos < b.len() && !is_space(b[pos]) && b[pos] != b'>' {
                        pos += 1;
                    }
                    value = body[from..pos].to_owned();
                }
            }
        }
        if attrs.len() < MAX_ATTRS {
            attrs.push((attr_name, decode_entities(&value)));
        }
    }
    let token = if closing {
        Token::Close(name)
    } else {
        Token::Open { name, attrs }
    };
    Some((offset + pos, token))
}

/// Characters that must never come out of an entity: controls, zero-width and
/// direction overrides (the server refuses the same ones in a message).
pub fn printable(c: char) -> bool {
    !c.is_control()
        && !matches!(
            c,
            '\u{200B}'..='\u{200F}'
                | '\u{202A}'..='\u{202E}'
                | '\u{2060}'..='\u{2064}'
                | '\u{2066}'..='\u{2069}'
                | '\u{FEFF}'
        )
}

/// One entity at the start of `s` (`s` begins with `&`): the character (or
/// `None` for a numeric entity that is not allowed) and the length consumed.
fn entity(s: &str) -> Option<(Option<char>, usize)> {
    let end = s[1..].find(';').filter(|e| *e <= 10)?;
    let name = &s[1..1 + end];
    let len = end + 2;
    let c = match name {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => ' ',
        _ => {
            let digits = name.strip_prefix('#')?;
            let code = match digits.strip_prefix(['x', 'X']) {
                Some(hex) => u32::from_str_radix(hex, 16).ok()?,
                None => digits.parse::<u32>().ok()?,
            };
            return Some((char::from_u32(code).filter(|c| printable(*c)), len));
        }
    };
    Some((Some(c), len))
}

/// Decodes the few entities that matter in real READMEs (`&amp;`, `&lt;`,
/// `&gt;`, `&quot;`, `&apos;`, `&nbsp;` and numeric ones). Anything else is
/// left as typed.
pub fn decode_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        rest = &rest[at..];
        match entity(rest) {
            Some((c, len)) => {
                out.extend(c);
                rest = &rest[len..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Collapses every run of whitespace to one space, as HTML does.
pub fn collapse_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut space = false;
    for c in s.chars() {
        if c.is_whitespace() {
            space = true;
        } else {
            if space {
                out.push(' ');
                space = false;
            }
            out.push(c);
        }
    }
    if space {
        out.push(' ');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open(name: &str, attrs: &[(&str, &str)]) -> Token {
        Token::Open {
            name: name.into(),
            attrs: attrs
                .iter()
                .map(|(n, v)| ((*n).into(), (*v).into()))
                .collect(),
        }
    }

    #[test]
    fn tags_text_and_attributes_are_split() {
        assert_eq!(
            tokenize("a <b>bold</b> <IMG SRC=\"x.png\" alt='y z' width=10 hidden/> c"),
            [
                Token::Text("a ".into()),
                open("b", &[]),
                Token::Text("bold".into()),
                Token::Close("b".into()),
                Token::Text(" ".into()),
                open(
                    "img",
                    &[
                        ("src", "x.png"),
                        ("alt", "y z"),
                        ("width", "10"),
                        ("hidden", "")
                    ]
                ),
                Token::Text(" c".into()),
            ]
        );
    }

    #[test]
    fn a_greater_than_sign_inside_quotes_does_not_end_the_tag() {
        assert_eq!(
            tokenize("<a title=\"1 > 0\" href=x>"),
            [open("a", &[("title", "1 > 0"), ("href", "x")])]
        );
    }

    #[test]
    fn comments_are_dropped_and_odd_angle_brackets_stay_text() {
        assert_eq!(
            tokenize("a<!-- hi -->b"),
            [Token::Text("a".into()), Token::Text("b".into())]
        );
        assert_eq!(tokenize("<!-- never closed"), []);
        for odd in [
            "1 < 2",
            "<3",
            "<>",
            "</>",
            "<a",
            "<a href=\"x",
            "<?php ?>",
            "<",
        ] {
            let tokens = tokenize(odd);
            assert!(
                tokens.iter().all(|t| matches!(t, Token::Text(_))),
                "{odd}: {tokens:?}"
            );
        }
    }

    #[test]
    fn entities_decode_but_never_into_control_or_direction_characters() {
        assert_eq!(
            decode_entities("a &amp; b &lt;c&gt; &quot;d&quot;"),
            "a & b <c> \"d\""
        );
        assert_eq!(decode_entities("&#65;&#x42;"), "AB");
        assert_eq!(decode_entities("&#8238;x"), "x"); // U+202E
        assert_eq!(decode_entities("&#0;&#10;&#x200B;y"), "y");
        assert_eq!(decode_entities("&#99999999999;"), "&#99999999999;");
        assert_eq!(decode_entities("&unknown; & &"), "&unknown; & &");
    }

    #[test]
    fn attribute_values_are_decoded() {
        assert_eq!(
            tokenize("<a href=\"https://a.dev/?x=1&amp;y=2\">"),
            [open("a", &[("href", "https://a.dev/?x=1&y=2")])]
        );
    }

    #[test]
    fn whitespace_collapses_like_html() {
        assert_eq!(collapse_whitespace(" a \n\t b  "), " a b ");
        assert_eq!(collapse_whitespace("\n\n"), " ");
        assert_eq!(collapse_whitespace(""), "");
    }

    #[test]
    fn many_attributes_are_capped_and_garbage_never_panics() {
        let many = format!("<a {}>", "x=1 ".repeat(100));
        match &tokenize(&many)[0] {
            Token::Open { attrs, .. } => assert!(attrs.len() <= MAX_ATTRS),
            other => panic!("{other:?}"),
        }
        for s in [
            "<a =>",
            "<a = = =>",
            "<a b=>",
            "<a b= >",
            "</a b c>",
            "<é>",
            "<a é=é>",
            "<a/////>",
            "\u{1F600}<b>\u{1F600}</b>",
        ] {
            let _ = tokenize(s);
        }
    }
}
