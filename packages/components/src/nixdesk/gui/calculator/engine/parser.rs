//! Tokenizer dan parser rekursif-turun.

use super::CalcError;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Tok {
    Num(f64),
    /// Nama fungsi atau konstanta yang dikenal (lihat `NAMES`).
    Name(&'static str),
    /// Variabel satu huruf.
    Var(char),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
    Semi,
    Bang,
    Percent,
    Degree,
    Cmp(CmpOp),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Expr {
    Num(f64),
    Var(char),
    Const(&'static str),
    Neg(Box<Expr>),
    Bin(BinOp, Box<Expr>, Box<Expr>),
    Fact(Box<Expr>),
    Percent(Box<Expr>),
    Degree(Box<Expr>),
    Call(&'static str, Vec<Expr>),
    Cmp(CmpOp, Box<Expr>, Box<Expr>),
}

/// Nama yang dikenal, urut dari yang terpanjang agar pencocokan awalan benar.
/// Dipakai juga oleh renderer MathML.
pub const NAMES: &[&str] = &[
    "arcsin", "arccos", "arctan", "asin", "acos", "atan", "sqrt", "root", "frac", "diff", "liml",
    "limr", "lim", "sum", "int", "abs", "sin", "cos", "tan", "csc", "sec", "cot", "log", "exp",
    "nCr", "nPr", "inf", "ln", "pi",
];

/// Cari nama yang dikenal di awal `s`.
pub fn match_name(s: &str) -> Option<&'static str> {
    NAMES.iter().copied().find(|n| s.starts_with(n))
}

/// Jumlah argumen yang diizinkan (min, maks) untuk sebuah fungsi.
fn arity(name: &str) -> (usize, usize) {
    match name {
        "frac" | "root" | "nCr" | "nPr" => (2, 2),
        "log" => (1, 2),
        "diff" | "lim" | "liml" | "limr" => (3, 3),
        "sum" | "int" => (4, 4),
        _ => (1, 1),
    }
}

pub fn lex(input: &str) -> Result<Vec<Tok>, CalcError> {
    let chars: Vec<char> = input.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' | '\n' => i += 1,
            '0'..='9' | '.' | ',' => {
                // ',' atau '.' hanya valid bila diikuti/didahului angka
                let starts_with_sep = c == '.' || c == ',';
                if starts_with_sep && !chars.get(i + 1).is_some_and(|d| d.is_ascii_digit()) {
                    return Err(CalcError::Syntax(
                        "Tanda desimal tidak pada tempatnya".into(),
                    ));
                }
                let mut s = String::new();
                while i < chars.len() && chars[i].is_ascii_digit() {
                    s.push(chars[i]);
                    i += 1;
                }
                if i < chars.len() && (chars[i] == '.' || chars[i] == ',') {
                    s.push('.');
                    i += 1;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        s.push(chars[i]);
                        i += 1;
                    }
                }
                if s.starts_with('.') {
                    s.insert(0, '0');
                }
                if s.ends_with('.') {
                    s.push('0');
                }
                let v: f64 = s
                    .parse()
                    .map_err(|_| CalcError::Syntax("Angka tidak valid".into()))?;
                out.push(Tok::Num(v));
            }
            '+' => {
                out.push(Tok::Plus);
                i += 1;
            }
            '-' | '−' => {
                out.push(Tok::Minus);
                i += 1;
            }
            '*' | '×' | '·' => {
                out.push(Tok::Star);
                i += 1;
            }
            '/' | '÷' => {
                out.push(Tok::Slash);
                i += 1;
            }
            '^' => {
                out.push(Tok::Caret);
                i += 1;
            }
            '²' => {
                out.push(Tok::Caret);
                out.push(Tok::Num(2.0));
                i += 1;
            }
            '³' => {
                out.push(Tok::Caret);
                out.push(Tok::Num(3.0));
                i += 1;
            }
            '(' => {
                out.push(Tok::LParen);
                i += 1;
            }
            ')' => {
                out.push(Tok::RParen);
                i += 1;
            }
            ';' => {
                out.push(Tok::Semi);
                i += 1;
            }
            '!' => {
                out.push(Tok::Bang);
                i += 1;
            }
            '%' => {
                out.push(Tok::Percent);
                i += 1;
            }
            '°' => {
                out.push(Tok::Degree);
                i += 1;
            }
            '√' => {
                out.push(Tok::Name("sqrt"));
                i += 1;
            }
            'π' => {
                out.push(Tok::Name("pi"));
                i += 1;
            }
            '∞' => {
                out.push(Tok::Name("inf"));
                i += 1;
            }
            '=' => {
                out.push(Tok::Cmp(CmpOp::Eq));
                i += 1;
            }
            '≠' => {
                out.push(Tok::Cmp(CmpOp::Ne));
                i += 1;
            }
            '<' => {
                out.push(Tok::Cmp(CmpOp::Lt));
                i += 1;
            }
            '≤' => {
                out.push(Tok::Cmp(CmpOp::Le));
                i += 1;
            }
            '>' => {
                out.push(Tok::Cmp(CmpOp::Gt));
                i += 1;
            }
            '≥' => {
                out.push(Tok::Cmp(CmpOp::Ge));
                i += 1;
            }
            c if c.is_alphabetic() => {
                let rest: String = chars[i..].iter().collect();
                if let Some(name) = match_name(&rest) {
                    out.push(Tok::Name(name));
                    i += name.chars().count();
                } else {
                    out.push(Tok::Var(c));
                    i += 1;
                }
            }
            other => {
                return Err(CalcError::Syntax(format!(
                    "Karakter tidak dikenali: {other}"
                )))
            }
        }
    }
    Ok(out)
}

pub fn parse(toks: Vec<Tok>) -> Result<Expr, CalcError> {
    let mut p = Parser { toks, pos: 0 };
    let e = p.parse_cmp()?;
    match p.peek() {
        None => Ok(e),
        Some(Tok::RParen) => Err(CalcError::Syntax("Kurung tutup berlebih".into())),
        Some(_) => Err(CalcError::Syntax("Ekspresi tidak valid".into())),
    }
}

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn bump(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn incomplete() -> CalcError {
        CalcError::Syntax("Ekspresi belum lengkap".into())
    }

    fn parse_cmp(&mut self) -> Result<Expr, CalcError> {
        let l = self.parse_add()?;
        if let Some(Tok::Cmp(op)) = self.peek().cloned() {
            self.bump();
            let r = self.parse_add()?;
            return Ok(Expr::Cmp(op, Box::new(l), Box::new(r)));
        }
        Ok(l)
    }

    fn parse_add(&mut self) -> Result<Expr, CalcError> {
        let mut l = self.parse_mul()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Plus) => BinOp::Add,
                Some(Tok::Minus) => BinOp::Sub,
                _ => break,
            };
            self.bump();
            let r = self.parse_mul()?;
            l = Expr::Bin(op, Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn starts_primary(&self) -> bool {
        matches!(
            self.peek(),
            Some(Tok::Num(_) | Tok::Name(_) | Tok::Var(_) | Tok::LParen)
        )
    }

    fn parse_mul(&mut self) -> Result<Expr, CalcError> {
        let mut l = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Star) => {
                    self.bump();
                    BinOp::Mul
                }
                Some(Tok::Slash) => {
                    self.bump();
                    BinOp::Div
                }
                _ if self.starts_primary() => BinOp::Mul, // perkalian implisit: 2π, 3(4+1), 2x
                _ => break,
            };
            let r = self.parse_unary()?;
            l = Expr::Bin(op, Box::new(l), Box::new(r));
        }
        Ok(l)
    }

    fn parse_unary(&mut self) -> Result<Expr, CalcError> {
        match self.peek() {
            Some(Tok::Minus) => {
                self.bump();
                Ok(Expr::Neg(Box::new(self.parse_unary()?)))
            }
            Some(Tok::Plus) => {
                self.bump();
                self.parse_unary()
            }
            _ => self.parse_power(),
        }
    }

    fn parse_power(&mut self) -> Result<Expr, CalcError> {
        let base = self.parse_postfix()?;
        if let Some(Tok::Caret) = self.peek() {
            self.bump();
            let exp = self.parse_unary()?; // asosiatif kanan, mengizinkan 2^-3
            return Ok(Expr::Bin(BinOp::Pow, Box::new(base), Box::new(exp)));
        }
        Ok(base)
    }

    fn parse_postfix(&mut self) -> Result<Expr, CalcError> {
        let mut e = self.parse_primary()?;
        loop {
            e = match self.peek() {
                Some(Tok::Bang) => Expr::Fact(Box::new(e)),
                Some(Tok::Percent) => Expr::Percent(Box::new(e)),
                Some(Tok::Degree) => Expr::Degree(Box::new(e)),
                _ => break,
            };
            self.bump();
        }
        Ok(e)
    }

    /// Tutup kurung; kurung yang belum ditutup di ujung ekspresi diterima.
    fn close_paren(&mut self) -> Result<(), CalcError> {
        match self.peek() {
            Some(Tok::RParen) => {
                self.bump();
                Ok(())
            }
            None => Ok(()),
            Some(_) => Err(CalcError::Syntax("Ekspresi tidak valid".into())),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, CalcError> {
        match self.bump() {
            Some(Tok::Num(v)) => Ok(Expr::Num(v)),
            Some(Tok::Var(c)) => Ok(Expr::Var(c)),
            Some(Tok::LParen) => {
                let e = self.parse_add()?;
                self.close_paren()?;
                Ok(e)
            }
            Some(Tok::Name(n)) => {
                if n == "pi" || n == "inf" {
                    return Ok(Expr::Const(n));
                }
                if !matches!(self.peek(), Some(Tok::LParen)) {
                    return Err(CalcError::Syntax(format!(
                        "Fungsi {n} membutuhkan tanda kurung"
                    )));
                }
                self.bump();
                let mut args = vec![self.parse_add()?];
                while let Some(Tok::Semi) = self.peek() {
                    self.bump();
                    args.push(self.parse_add()?);
                }
                self.close_paren()?;
                let (lo, hi) = arity(n);
                if args.len() < lo || args.len() > hi {
                    return Err(CalcError::Syntax(if lo == hi {
                        format!("{n} membutuhkan {lo} argumen (pisahkan dengan ;)")
                    } else {
                        format!("{n} membutuhkan {lo}–{hi} argumen (pisahkan dengan ;)")
                    }));
                }
                Ok(Expr::Call(n, args))
            }
            None => Err(Self::incomplete()),
            Some(Tok::RParen) => Err(CalcError::Syntax("Kurung tutup berlebih".into())),
            Some(_) => Err(CalcError::Syntax("Ekspresi tidak valid".into())),
        }
    }
}
