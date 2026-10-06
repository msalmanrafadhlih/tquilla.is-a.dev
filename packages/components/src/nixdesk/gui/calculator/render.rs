//! Ubah ekspresi linear menjadi MathML untuk ditampilkan (pangkat, pecahan, akar, ∑, ∫, ...).
//!
//! Renderer ini toleran: ekspresi yang belum lengkap (kurung terbuka, argumen belum diisi)
//! tetap dirender, dengan kotak □ redup sebagai tempat kosong.

use super::engine::parser::match_name;

const EMPTY: &str = "<mi style=\"opacity:.4\">□</mi>";

pub fn to_mathml(expr: &str) -> String {
    let mut cur = Cursor {
        chars: expr.chars().collect(),
        i: 0,
    };
    let body = cur.row(true);
    if body.is_empty() {
        return "<math><mn>0</mn></math>".to_string();
    }
    format!("<math>{body}</math>")
}

/// Escape teks biasa agar aman dipakai sebagai `inner_html`.
pub fn html_escape(s: &str) -> String {
    s.chars().map(esc).collect()
}

struct Cursor {
    chars: Vec<char>,
    i: usize,
}

fn esc(c: char) -> String {
    match c {
        '<' => "&lt;".into(),
        '>' => "&gt;".into(),
        '&' => "&amp;".into(),
        _ => c.to_string(),
    }
}

fn mo(s: &str) -> String {
    format!("<mo>{s}</mo>")
}

fn or_empty(s: &str) -> String {
    if s.is_empty() {
        EMPTY.to_string()
    } else {
        s.to_string()
    }
}

fn mrow(s: &str) -> String {
    format!("<mrow>{}</mrow>", or_empty(s))
}

impl Cursor {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.i).copied()
    }

    fn rest(&self) -> String {
        self.chars[self.i..].iter().collect()
    }

    /// Baca deretan atom sampai ujung ekspresi, `)` atau `;`.
    /// Di level teratas (`top`), `)` dan `;` yang nyasar ditampilkan apa adanya.
    fn row(&mut self, top: bool) -> String {
        let mut atoms: Vec<String> = Vec::new();
        while let Some(c) = self.peek() {
            if !top && (c == ')' || c == ';') {
                break;
            }
            match c {
                '^' | '²' | '³' => {
                    self.i += 1;
                    let exp = match c {
                        '²' => "<mn>2</mn>".to_string(),
                        '³' => "<mn>3</mn>".to_string(),
                        _ => self.exponent(),
                    };
                    let base = atoms.pop().unwrap_or_else(|| EMPTY.to_string());
                    atoms.push(format!("<msup>{}{}</msup>", mrow_one(&base), mrow(&exp)));
                }
                _ => {
                    if let Some(a) = self.atom() {
                        atoms.push(a);
                    }
                }
            }
        }
        atoms.concat()
    }

    /// Pangkat: `(…)` ditampilkan tanpa kurung; selain itu satu angka/huruf (boleh bertanda -).
    fn exponent(&mut self) -> String {
        let mut out = String::new();
        if self.peek() == Some('(') {
            self.i += 1;
            out.push_str(&self.row(false));
            if self.peek() == Some(')') {
                self.i += 1;
            }
            return out;
        }
        if matches!(self.peek(), Some('-') | Some('−') | Some('+')) {
            out.push_str(&mo(if self.peek() == Some('+') { "+" } else { "−" }));
            self.i += 1;
        }
        match self.peek() {
            Some(c) if c.is_ascii_digit() || c == ',' || c == '.' => out.push_str(&self.number()),
            Some(c) if c.is_alphabetic() => {
                if let Some(a) = self.atom() {
                    out.push_str(&a);
                }
            }
            _ => {}
        }
        out
    }

    fn number(&mut self) -> String {
        let mut s = String::new();
        let mut seen_sep = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                s.push(c);
            } else if (c == ',' || c == '.') && !seen_sep {
                seen_sep = true;
                s.push(',');
            } else {
                break;
            }
            self.i += 1;
        }
        format!("<mn>{s}</mn>")
    }

    /// Argumen fungsi setelah `(`: daftar baris, dipisah `;`. Mengembalikan (argumen, sudah ditutup).
    fn args(&mut self) -> (Vec<String>, bool) {
        let mut out = Vec::new();
        loop {
            out.push(self.row(false));
            match self.peek() {
                Some(';') => self.i += 1,
                Some(')') => {
                    self.i += 1;
                    return (out, true);
                }
                _ => return (out, false),
            }
        }
    }

    fn atom(&mut self) -> Option<String> {
        let c = self.peek()?;
        if c.is_ascii_digit() || ((c == ',' || c == '.') && self.chars.get(self.i + 1).is_some_and(|d| d.is_ascii_digit())) {
            return Some(self.number());
        }
        match c {
            '(' => {
                self.i += 1;
                let inner = self.row(false);
                let closed = self.peek() == Some(')');
                if closed {
                    self.i += 1;
                }
                let close = if closed {
                    mo(")")
                } else {
                    "<mo style=\"opacity:.35\">)</mo>".to_string()
                };
                Some(format!("<mrow>{}{}{}</mrow>", mo("("), inner, close))
            }
            '+' => self.take(mo("+")),
            '-' | '−' => self.take(mo("−")),
            '*' | '×' | '·' => self.take(mo("×")),
            '/' | '÷' => self.take(mo("÷")),
            '!' | '%' | '°' | '=' | '<' | '>' | '≤' | '≥' | '≠' | ')' | ';' | ',' | '.' => {
                self.i += 1;
                Some(mo(&esc(c)))
            }
            'π' => self.take("<mi>π</mi>".into()),
            '∞' => self.take("<mi>∞</mi>".into()),
            '√' => {
                self.i += 1;
                if self.peek() == Some('(') {
                    self.i += 1;
                    let (args, _) = self.args();
                    Some(format!("<msqrt>{}</msqrt>", or_empty(&args[0])))
                } else {
                    Some(mo("√"))
                }
            }
            c if c.is_alphabetic() => {
                let rest = self.rest();
                match match_name(&rest) {
                    Some(name) if rest[name.len()..].starts_with('(') => {
                        self.i += name.chars().count() + 1;
                        let (args, closed) = self.args();
                        Some(self.call(name, &args, closed))
                    }
                    Some("pi") => {
                        self.i += 2;
                        Some("<mi>π</mi>".into())
                    }
                    Some("inf") => {
                        self.i += 3;
                        Some("<mi>∞</mi>".into())
                    }
                    Some(name) => {
                        self.i += name.chars().count();
                        Some(format!("<mi>{name}</mi>"))
                    }
                    None => {
                        self.i += 1;
                        Some(format!("<mi>{c}</mi>"))
                    }
                }
            }
            other => {
                self.i += 1;
                Some(format!("<mo>{}</mo>", esc(other)))
            }
        }
    }

    fn take(&mut self, s: String) -> Option<String> {
        self.i += 1;
        Some(s)
    }

    fn call(&self, name: &str, a: &[String], closed: bool) -> String {
        let arg = |k: usize| a.get(k).map(String::as_str).unwrap_or("");
        let paren_close = if closed {
            mo(")")
        } else {
            "<mo style=\"opacity:.35\">)</mo>".to_string()
        };
        match name {
            "sqrt" => format!("<msqrt>{}</msqrt>", or_empty(arg(0))),
            "abs" => format!("<mrow>{}{}{}</mrow>", mo("|"), or_empty(arg(0)), mo("|")),
            "frac" => format!("<mfrac>{}{}</mfrac>", mrow(arg(0)), mrow(arg(1))),
            "root" => format!("<mroot>{}{}</mroot>", mrow(arg(1)), mrow(arg(0))),
            "log" if a.len() >= 2 => format!(
                "<msub><mi>log</mi>{}</msub>{}{}{}",
                mrow(arg(0)),
                mo("("),
                or_empty(arg(1)),
                paren_close
            ),
            "nCr" | "nPr" => format!(
                "<mi>{}</mi>{}{}{}{}{}",
                if name == "nCr" { "C" } else { "P" },
                mo("("),
                or_empty(arg(0)),
                mo(","),
                or_empty(arg(1)),
                paren_close
            ),
            "diff" => {
                let v = or_empty(arg(1));
                let mut s = format!(
                    "<mfrac><mi>d</mi><mrow><mi>d</mi>{v}</mrow></mfrac>{}{}{}",
                    mo("("),
                    or_empty(arg(0)),
                    mo(")")
                );
                if a.len() >= 3 {
                    s.push_str(&format!(
                        "<msub>{}<mrow>{v}{}{}</mrow></msub>",
                        mo("|"),
                        mo("="),
                        or_empty(arg(2))
                    ));
                }
                s
            }
            "lim" | "liml" | "limr" => {
                let target = match name {
                    "limr" => format!("<msup>{}<mo>+</mo></msup>", mrow(arg(2))),
                    "liml" => format!("<msup>{}<mo>−</mo></msup>", mrow(arg(2))),
                    _ => or_empty(arg(2)),
                };
                format!(
                    "<munder><mi>lim</mi><mrow>{}{}{}</mrow></munder>{}",
                    or_empty(arg(1)),
                    mo("→"),
                    target,
                    mrow(arg(0))
                )
            }
            "sum" => format!(
                "<munderover><mo>∑</mo><mrow>{}{}{}</mrow>{}</munderover>{}",
                or_empty(arg(1)),
                mo("="),
                or_empty(arg(2)),
                mrow(arg(3)),
                mrow(arg(0))
            ),
            "int" => format!(
                "<msubsup><mo>∫</mo>{}{}</msubsup>{}<mi>d</mi>{}",
                mrow(arg(2)),
                mrow(arg(3)),
                mrow(arg(0)),
                or_empty(arg(1))
            ),
            // sin, cos, ln, log (satu argumen), exp, ...
            _ => format!(
                "<mi>{name}</mi>{}{}{}",
                mo("("),
                or_empty(arg(0)),
                paren_close
            ),
        }
    }
}

/// Bungkus satu atom agar bisa jadi basis `<msup>` (atom sudah berupa satu elemen).
fn mrow_one(atom: &str) -> String {
    format!("<mrow>{atom}</mrow>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kosong_dan_angka() {
        assert_eq!(to_mathml(""), "<math><mn>0</mn></math>");
        assert_eq!(to_mathml("12,5"), "<math><mn>12,5</mn></math>");
    }

    #[test]
    fn pangkat_dan_pecahan() {
        let m = to_mathml("2^3+1");
        assert!(m.contains("<msup><mrow><mn>2</mn></mrow><mrow><mn>3</mn></mrow></msup>"), "{m}");
        assert!(m.ends_with("<mo>+</mo><mn>1</mn></math>"), "{m}");
        let m = to_mathml("frac(1;2)");
        assert!(m.contains("<mfrac><mrow><mn>1</mn></mrow><mrow><mn>2</mn></mrow></mfrac>"), "{m}");
        let m = to_mathml("1,5×10^20");
        assert!(m.contains("<msup><mrow><mn>10</mn></mrow><mrow><mn>20</mn></mrow></msup>"), "{m}");
        let m = to_mathml("2^-3");
        assert!(m.contains("<mo>−</mo><mn>3</mn>"), "{m}");
    }

    #[test]
    fn belum_lengkap_tetap_dirender() {
        let m = to_mathml("sin(");
        assert!(m.contains("<mi>sin</mi>") && m.contains("□"), "{m}");
        let m = to_mathml("frac(1;");
        assert!(m.contains("<mfrac>") && m.contains("□"), "{m}");
        let m = to_mathml("(2+3");
        assert!(m.contains("opacity:.35"), "{m}");
        let m = to_mathml("3^");
        assert!(m.contains("<msup>") && m.contains("□"), "{m}");
        let m = to_mathml("^");
        assert!(m.contains("<msup>"), "{m}");
    }

    #[test]
    fn kalkulus() {
        let m = to_mathml("int(x^2;x;0;1)");
        assert!(m.contains("<msubsup><mo>∫</mo>") && m.contains("<mi>d</mi><mi>x</mi>"), "{m}");
        let m = to_mathml("sum(k;k;1;10)");
        assert!(m.contains("<munderover><mo>∑</mo>"), "{m}");
        let m = to_mathml("limr(1÷x;x;0)");
        assert!(m.contains("<munder><mi>lim</mi>") && m.contains("<mo>+</mo>"), "{m}");
        let m = to_mathml("diff(x^2;x;3)");
        assert!(m.contains("<mfrac><mi>d</mi>"), "{m}");
        let m = to_mathml("root(3;x)");
        assert!(m.contains("<mroot>"), "{m}");
        let m = to_mathml("log(2;8)");
        assert!(m.contains("<msub><mi>log</mi>"), "{m}");
    }

    #[test]
    fn karakter_khusus_di_escape() {
        let m = to_mathml("2<3");
        assert!(m.contains("&lt;"), "{m}");
        assert!(!m.contains("<mo><</mo>"));
    }
}
