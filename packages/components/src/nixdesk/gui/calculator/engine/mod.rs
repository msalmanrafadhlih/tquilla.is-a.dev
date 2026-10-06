//! Mesin kalkulator: tidak bergantung pada Dioxus, jadi bisa diuji dengan `cargo test`.
//!
//! Sintaks ekspresi (string linear):
//! - angka desimal memakai koma atau titik: `3,14`
//! - operator: `+ - × ÷ * / ^`, postfix: `! % °`
//! - pemisah argumen fungsi: `;` (karena koma dipakai sebagai desimal)
//! - fungsi: sin cos tan csc sec cot arcsin arccos arctan sqrt/√ root abs ln log exp frac nCr nPr
//! - kalkulus numerik: `diff(f; v; a)`, `lim(f; v; a)`, `liml`, `limr`,
//!   `sum(f; v; dari; sampai)`, `int(f; v; a; b)` (batas boleh ∞)
//! - perbandingan di level atas: `= < ≤ > ≥ ≠` → hasil Benar/Salah

mod calculus;
mod eval;
mod format;
pub mod parser;

#[cfg(test)]
mod tests;

use std::fmt;

pub use format::format_number;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Angle {
    Deg,
    Rad,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Outcome {
    Number(f64),
    Bool(bool),
}

impl Outcome {
    /// Teks yang ditampilkan ke pengguna.
    pub fn to_text(&self) -> String {
        match self {
            Outcome::Number(v) => format_number(*v),
            Outcome::Bool(true) => "Benar".to_string(),
            Outcome::Bool(false) => "Salah".to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum CalcError {
    Empty,
    Syntax(String),
    DivZero,
    Overflow,
    Domain(String),
    UnknownVar(char),
    Unsupported(String),
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalcError::Empty => write!(f, "Ekspresi kosong"),
            CalcError::Syntax(m) => write!(f, "{m}"),
            CalcError::DivZero => write!(f, "Tidak bisa membagi dengan nol"),
            CalcError::Overflow => write!(f, "Hasil terlalu besar"),
            CalcError::Domain(m) => write!(f, "{m}"),
            CalcError::UnknownVar(c) => write!(f, "Variabel {c} belum diketahui"),
            CalcError::Unsupported(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for CalcError {}

/// Hitung sebuah ekspresi. Kurung yang belum ditutup di ujung ekspresi diterima.
pub fn evaluate(input: &str, angle: Angle) -> Result<Outcome, CalcError> {
    let toks = parser::lex(input)?;
    if toks.is_empty() {
        return Err(CalcError::Empty);
    }
    let expr = parser::parse(toks)?;
    let mut ctx = eval::Ctx::new(angle);
    match expr {
        parser::Expr::Cmp(op, l, r) => {
            let a = ctx.eval(&l)?;
            let b = ctx.eval(&r)?;
            Ok(Outcome::Bool(eval::compare(op, a, b)))
        }
        other => {
            let v = ctx.eval(&other)?;
            if v.is_nan() {
                return Err(CalcError::Domain("Hasil tidak terdefinisi".into()));
            }
            Ok(Outcome::Number(v))
        }
    }
}

/// `true` bila ekspresi hanya sebuah angka (dengan tanda minus opsional).
/// Dipakai UI untuk menyembunyikan pratinjau yang sama dengan masukan.
pub fn is_plain_number(input: &str) -> bool {
    use parser::Tok;
    match parser::lex(input) {
        Ok(t) => matches!(t.as_slice(), [Tok::Num(_)] | [Tok::Minus, Tok::Num(_)]),
        Err(_) => false,
    }
}
