//! Evaluator AST → f64.

use std::f64::consts::{E, PI};

use super::calculus;
use super::parser::{BinOp, CmpOp, Expr};
use super::{Angle, CalcError};

pub struct Ctx {
    pub angle: Angle,
    env: Vec<(char, f64)>,
}

pub fn compare(op: CmpOp, a: f64, b: f64) -> bool {
    let eq = a == b || (a - b).abs() <= 1e-9 * 1.0_f64.max(a.abs()).max(b.abs());
    match op {
        CmpOp::Eq => eq,
        CmpOp::Ne => !eq,
        CmpOp::Lt => a < b && !eq,
        CmpOp::Le => a < b || eq,
        CmpOp::Gt => a > b && !eq,
        CmpOp::Ge => a > b || eq,
    }
}

/// Hasil operasi hingga bila operandnya hingga; selain itu error.
fn check(a: f64, b: f64, r: f64) -> Result<f64, CalcError> {
    if r.is_nan() {
        return Err(CalcError::Domain("Hasil tidak terdefinisi".into()));
    }
    if r.is_infinite() && a.is_finite() && b.is_finite() {
        return Err(CalcError::Overflow);
    }
    Ok(r)
}

/// Bulatkan hasil trigonometri yang seharusnya nol.
fn snap(x: f64) -> f64 {
    if x.abs() < 1e-14 {
        0.0
    } else {
        x
    }
}

impl Ctx {
    pub fn new(angle: Angle) -> Self {
        Ctx {
            angle,
            env: Vec::new(),
        }
    }

    /// Evaluasi `e` dengan variabel `v` terikat ke `x`.
    pub fn eval_at(&mut self, e: &Expr, v: char, x: f64) -> Result<f64, CalcError> {
        self.env.push((v, x));
        let r = self.eval(e);
        self.env.pop();
        r
    }

    fn to_rad(&self, x: f64) -> f64 {
        match self.angle {
            Angle::Rad => x,
            Angle::Deg => x * PI / 180.0,
        }
    }

    fn from_rad(&self, x: f64) -> f64 {
        match self.angle {
            Angle::Rad => x,
            Angle::Deg => x * 180.0 / PI,
        }
    }

    pub fn eval(&mut self, e: &Expr) -> Result<f64, CalcError> {
        match e {
            Expr::Num(v) => Ok(*v),
            Expr::Const("pi") => Ok(PI),
            Expr::Const(_) => Ok(f64::INFINITY),
            Expr::Var(c) => {
                if let Some((_, v)) = self.env.iter().rev().find(|(n, _)| n == c) {
                    return Ok(*v);
                }
                match c {
                    'e' => Ok(E),
                    'i' => Err(CalcError::Unsupported(
                        "Bilangan kompleks (i) belum didukung".into(),
                    )),
                    _ => Err(CalcError::UnknownVar(*c)),
                }
            }
            Expr::Neg(x) => Ok(-self.eval(x)?),
            Expr::Bin(op, l, r) => {
                let a = self.eval(l)?;
                let b = self.eval(r)?;
                let v = match op {
                    BinOp::Add => a + b,
                    BinOp::Sub => a - b,
                    BinOp::Mul => a * b,
                    BinOp::Div => {
                        if b == 0.0 {
                            return Err(CalcError::DivZero);
                        }
                        a / b
                    }
                    BinOp::Pow => {
                        if a == 0.0 && b < 0.0 {
                            return Err(CalcError::DivZero);
                        }
                        if a < 0.0 && b.is_finite() && b.fract() != 0.0 {
                            return Err(CalcError::Domain(
                                "Pangkat pecahan dari bilangan negatif belum didukung".into(),
                            ));
                        }
                        a.powf(b)
                    }
                };
                check(a, b, v)
            }
            Expr::Fact(x) => factorial(self.eval(x)?),
            Expr::Percent(x) => Ok(self.eval(x)? / 100.0),
            Expr::Degree(x) => {
                let v = self.eval(x)?;
                // Di mode DEG, `30°` sudah dalam satuan sudut aktif.
                Ok(match self.angle {
                    Angle::Deg => v,
                    Angle::Rad => v * PI / 180.0,
                })
            }
            Expr::Cmp(..) => Err(CalcError::Unsupported(
                "Perbandingan (=, <, ≤, …) hanya boleh di tingkat teratas".into(),
            )),
            Expr::Call(name, args) => self.call(name, args),
        }
    }

    fn arg(&mut self, args: &[Expr], i: usize) -> Result<f64, CalcError> {
        self.eval(&args[i])
    }

    fn call(&mut self, name: &str, args: &[Expr]) -> Result<f64, CalcError> {
        match name {
            "diff" => return calculus::diff(self, args),
            "lim" | "liml" | "limr" => return calculus::limit(self, name, args),
            "sum" => return calculus::sum(self, args),
            "int" => return calculus::integral(self, args),
            _ => {}
        }
        let x = self.arg(args, 0)?;
        match name {
            "sqrt" => {
                if x < 0.0 {
                    return Err(CalcError::Domain(
                        "Akar kuadrat bilangan negatif belum didukung".into(),
                    ));
                }
                Ok(x.sqrt())
            }
            "abs" => Ok(x.abs()),
            "exp" => check(x, x, x.exp()),
            "ln" => ln(x),
            "log" => {
                if args.len() == 1 {
                    ln(x).map(|l| snap_int(l / std::f64::consts::LN_10))
                } else {
                    // log(basis; x)
                    let b = x;
                    let v = self.arg(args, 1)?;
                    if b <= 0.0 || b == 1.0 {
                        return Err(CalcError::Domain("Basis logaritma tidak valid".into()));
                    }
                    Ok(snap_int(ln(v)? / ln(b)?))
                }
            }
            "frac" => {
                let d = self.arg(args, 1)?;
                if d == 0.0 {
                    return Err(CalcError::DivZero);
                }
                check(x, d, x / d)
            }
            "root" => {
                let v = self.arg(args, 1)?;
                root(x, v)
            }
            "nCr" | "nPr" => {
                let k = self.arg(args, 1)?;
                combinatorics(name == "nCr", x, k)
            }
            "sin" => Ok(snap(self.to_rad(x).sin())),
            "cos" => Ok(snap(self.to_rad(x).cos())),
            "tan" => {
                let r = self.to_rad(x);
                let c = snap(r.cos());
                if c == 0.0 {
                    return Err(CalcError::Domain("tan tidak terdefinisi di sini".into()));
                }
                Ok(snap(r.sin()) / c)
            }
            "csc" => {
                let s = snap(self.to_rad(x).sin());
                if s == 0.0 {
                    return Err(CalcError::Domain("csc tidak terdefinisi di sini".into()));
                }
                Ok(1.0 / s)
            }
            "sec" => {
                let c = snap(self.to_rad(x).cos());
                if c == 0.0 {
                    return Err(CalcError::Domain("sec tidak terdefinisi di sini".into()));
                }
                Ok(1.0 / c)
            }
            "cot" => {
                let r = self.to_rad(x);
                let s = snap(r.sin());
                if s == 0.0 {
                    return Err(CalcError::Domain("cot tidak terdefinisi di sini".into()));
                }
                Ok(snap(r.cos()) / s)
            }
            "arcsin" | "asin" | "arccos" | "acos" => {
                if !(-1.0..=1.0).contains(&x) {
                    return Err(CalcError::Domain(
                        "arcsin/arccos hanya untuk nilai antara -1 dan 1".into(),
                    ));
                }
                let r = if name.ends_with("sin") { x.asin() } else { x.acos() };
                Ok(snap(self.from_rad(r)))
            }
            "arctan" | "atan" => Ok(snap(self.from_rad(x.atan()))),
            _ => Err(CalcError::Syntax(format!("Fungsi {name} tidak dikenal"))),
        }
    }
}

/// Buang noise seperti 2.9999999999999996 → 3 untuk hasil logaritma.
fn snap_int(x: f64) -> f64 {
    let r = x.round();
    if (x - r).abs() < 1e-12 {
        r
    } else {
        x
    }
}

fn ln(x: f64) -> Result<f64, CalcError> {
    if x <= 0.0 {
        return Err(CalcError::Domain(
            "Logaritma hanya untuk bilangan positif".into(),
        ));
    }
    Ok(x.ln())
}

fn factorial(x: f64) -> Result<f64, CalcError> {
    if x < 0.0 || x.fract() != 0.0 {
        return Err(CalcError::Domain(
            "Faktorial hanya untuk bilangan bulat ≥ 0".into(),
        ));
    }
    if x > 170.0 {
        return Err(CalcError::Overflow);
    }
    Ok((1..=x as u32).fold(1.0, |acc, k| acc * k as f64))
}

fn root(n: f64, x: f64) -> Result<f64, CalcError> {
    if n == 0.0 || n.fract() != 0.0 {
        return Err(CalcError::Domain(
            "Indeks akar harus bilangan bulat bukan nol".into(),
        ));
    }
    if x < 0.0 {
        if (n as i64) % 2 == 0 {
            return Err(CalcError::Domain(
                "Akar genap bilangan negatif belum didukung".into(),
            ));
        }
        return Ok(snap_int(-(-x).powf(1.0 / n)));
    }
    if x == 0.0 && n < 0.0 {
        return Err(CalcError::DivZero);
    }
    Ok(snap_int(x.powf(1.0 / n)))
}

fn combinatorics(comb: bool, n: f64, k: f64) -> Result<f64, CalcError> {
    if n < 0.0 || k < 0.0 || n.fract() != 0.0 || k.fract() != 0.0 || k > n {
        return Err(CalcError::Domain(
            "n dan k harus bilangan bulat dengan 0 ≤ k ≤ n".into(),
        ));
    }
    let (n, k) = (n as u64, k as u64);
    let k_eff = if comb { k.min(n - k) } else { k };
    let mut r = 1.0_f64;
    for i in 0..k_eff {
        r *= (n - i) as f64;
        if comb {
            r /= (i + 1) as f64;
        }
        if r.is_infinite() {
            return Err(CalcError::Overflow);
        }
    }
    Ok(r.round())
}
