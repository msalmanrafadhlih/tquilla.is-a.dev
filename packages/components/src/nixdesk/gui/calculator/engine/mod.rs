//! Mesin kalkulator: tidak bergantung pada Dioxus, jadi bisa diuji dengan `cargo test`.
//!
//! Sintaks ekspresi (string linear):
//! - angka desimal memakai titik (`3.14`); koma tetap diterima sebagai masukan
//! - operator: `+ - × ÷ * / ^`, postfix: `! % °`
//! - pemisah argumen fungsi: `;`
//! - fungsi: sin cos tan csc sec cot arcsin arccos arctan sqrt/√ root abs ln log exp frac nCr nPr
//! - kalkulus numerik: `diff(f; v; a)`, `lim(f; v; a)`, `liml`, `limr`,
//!   `sum(f; v; dari; sampai)`, `int(f; v; a; b)` (batas boleh ∞)
//! - perbandingan di level atas: `= < ≤ > ≥ ≠` → hasil True/False
//!
//! Engine tidak membawa teks UI: error berupa enum ([`CalcError`]) dan
//! diterjemahkan di UI lewat `CalcError::message`; hasil `True/False` lewat
//! `Outcome::display`. `format_number` memakai bentuk kanonik (titik desimal).

mod calculus;
mod eval;
mod format;
pub mod parser;

#[cfg(test)]
mod tests;

use std::fmt;

pub use format::format_number;

use crate::languages::{Language, Strings};

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
    /// Teks yang ditampilkan ke pengguna dalam bahasa `s` (True/False dan
    /// pemisah desimal mengikuti bahasa).
    pub fn display(self, s: &Strings) -> String {
        match self {
            Outcome::Number(v) => localize_number(&format_number(v), s),
            Outcome::Bool(true) => s.calc.true_text.to_string(),
            Outcome::Bool(false) => s.calc.false_text.to_string(),
        }
    }
}

/// Ganti titik desimal kanonik dengan pemisah desimal bahasa `s`.
/// Hanya untuk tampilan; jangan dimasukkan kembali ke ekspresi.
pub fn localize_number(canonical: &str, s: &Strings) -> String {
    canonical.replace('.', s.calc.decimal_separator)
}

/// Semua kegagalan perhitungan. Tidak ada teks di sini: pesan ditampilkan lewat
/// [`CalcError::message`] supaya mengikuti bahasa UI.
#[derive(Clone, PartialEq, Debug)]
pub enum CalcError {
    Empty,
    MisplacedDecimal,
    InvalidNumber,
    UnexpectedChar(char),
    ExtraCloseParen,
    InvalidExpression,
    Incomplete,
    NeedsParens(&'static str),
    ArgCount {
        name: &'static str,
        lo: usize,
        hi: usize,
    },
    DivZero,
    Overflow,
    Undefined,
    UnknownVar(char),
    UnknownFunction(String),
    ComplexUnsupported,
    FractionalPowerOfNegative,
    ComparisonNotTopLevel,
    SqrtNegative,
    InvalidLogBase,
    FnUndefinedHere(&'static str),
    ArcDomain,
    LogNotPositive,
    FactorialDomain,
    RootIndex,
    EvenRootOfNegative,
    CombinatoricsDomain,
    SecondArgNotVariable,
    InfiniteOrUndefined,
    DerivativeAtInfinity,
    InfiniteSeries,
    SigmaBoundsNotInteger,
    SigmaRangeTooLarge,
    LimitUndefined,
    LimitNoConverge,
    LimitPointInvalid,
    LimitSidesDiffer,
    IntegralDiverges,
    IntegralBoundsInvalid,
}

impl CalcError {
    /// Pesan dalam bahasa `s`.
    pub fn message(&self, s: &Strings) -> String {
        let e = &s.calc_err;
        match self {
            CalcError::Empty => e.empty.to_string(),
            CalcError::MisplacedDecimal => e.misplaced_decimal.to_string(),
            CalcError::InvalidNumber => e.invalid_number.to_string(),
            CalcError::UnexpectedChar(c) => e.unexpected_char(*c),
            CalcError::ExtraCloseParen => e.extra_close_paren.to_string(),
            CalcError::InvalidExpression => e.invalid_expression.to_string(),
            CalcError::Incomplete => e.incomplete.to_string(),
            CalcError::NeedsParens(name) => e.needs_parens(name),
            CalcError::ArgCount { name, lo, hi } => e.arg_count(name, *lo, *hi),
            CalcError::DivZero => e.div_zero.to_string(),
            CalcError::Overflow => e.overflow.to_string(),
            CalcError::Undefined => e.undefined.to_string(),
            CalcError::UnknownVar(c) => e.unknown_var(*c),
            CalcError::UnknownFunction(name) => e.unknown_function(name),
            CalcError::ComplexUnsupported => e.complex_unsupported.to_string(),
            CalcError::FractionalPowerOfNegative => e.fractional_power_negative.to_string(),
            CalcError::ComparisonNotTopLevel => e.comparison_top_level.to_string(),
            CalcError::SqrtNegative => e.sqrt_negative.to_string(),
            CalcError::InvalidLogBase => e.invalid_log_base.to_string(),
            CalcError::FnUndefinedHere(name) => e.fn_undefined_here(name),
            CalcError::ArcDomain => e.arc_domain.to_string(),
            CalcError::LogNotPositive => e.log_positive.to_string(),
            CalcError::FactorialDomain => e.factorial_domain.to_string(),
            CalcError::RootIndex => e.root_index.to_string(),
            CalcError::EvenRootOfNegative => e.even_root_negative.to_string(),
            CalcError::CombinatoricsDomain => e.comb_domain.to_string(),
            CalcError::SecondArgNotVariable => e.second_arg_variable.to_string(),
            CalcError::InfiniteOrUndefined => e.infinite_or_undefined.to_string(),
            CalcError::DerivativeAtInfinity => e.derivative_infinity.to_string(),
            CalcError::InfiniteSeries => e.infinite_series.to_string(),
            CalcError::SigmaBoundsNotInteger => e.sigma_bounds_integer.to_string(),
            CalcError::SigmaRangeTooLarge => e.sigma_range_large.to_string(),
            CalcError::LimitUndefined => e.limit_undefined.to_string(),
            CalcError::LimitNoConverge => e.limit_no_converge.to_string(),
            CalcError::LimitPointInvalid => e.limit_point_invalid.to_string(),
            CalcError::LimitSidesDiffer => e.limit_sides_differ.to_string(),
            CalcError::IntegralDiverges => e.integral_diverges.to_string(),
            CalcError::IntegralBoundsInvalid => e.integral_bounds_invalid.to_string(),
        }
    }
}

/// Bahasa bawaan (Inggris), untuk log dan test; UI memakai `message`.
impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message(Language::default().strings()))
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
                return Err(CalcError::Undefined);
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
