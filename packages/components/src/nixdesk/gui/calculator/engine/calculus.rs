//! Kalkulus numerik: turunan, limit, deret (Σ), dan integral tentu.

use super::eval::Ctx;
use super::parser::Expr;
use super::CalcError;

/// Bulatkan ke `sig` digit signifikan dan buang noise di sekitar nol.
fn clean(v: f64, sig: usize, zero_tol: f64) -> f64 {
    if !v.is_finite() {
        return v;
    }
    if v.abs() < zero_tol {
        return 0.0;
    }
    format!("{:.*e}", sig - 1, v).parse().unwrap_or(v)
}

fn var_arg(e: &Expr) -> Result<char, CalcError> {
    match e {
        Expr::Var(c) => Ok(*c),
        _ => Err(CalcError::Syntax(
            "Argumen kedua harus berupa variabel (mis. x)".into(),
        )),
    }
}

fn finite(v: f64) -> Result<f64, CalcError> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(CalcError::Domain(
            "Fungsi menghasilkan nilai tak hingga atau tak terdefinisi".into(),
        ))
    }
}

// ---------------------------------------------------------------- turunan

/// d/dv f di titik a, memakai stensil 5 titik (galat O(h⁴)).
pub fn diff(ctx: &mut Ctx, args: &[Expr]) -> Result<f64, CalcError> {
    let v = var_arg(&args[1])?;
    let a = ctx.eval(&args[2])?;
    if !a.is_finite() {
        return Err(CalcError::Unsupported(
            "Turunan di titik tak hingga tidak didukung".into(),
        ));
    }
    let h = 1e-3 * a.abs().max(1.0);
    let mut f = |x: f64| -> Result<f64, CalcError> { finite(ctx.eval_at(&args[0], v, x)?) };
    let d = (-f(a + 2.0 * h)? + 8.0 * f(a + h)? - 8.0 * f(a - h)? + f(a - 2.0 * h)?) / (12.0 * h);
    Ok(clean(d, 9, 1e-9))
}

// ------------------------------------------------------------------ deret

pub fn sum(ctx: &mut Ctx, args: &[Expr]) -> Result<f64, CalcError> {
    let v = var_arg(&args[1])?;
    let a = ctx.eval(&args[2])?;
    let b = ctx.eval(&args[3])?;
    if !a.is_finite() || !b.is_finite() {
        return Err(CalcError::Unsupported(
            "Deret tak hingga belum didukung".into(),
        ));
    }
    if a.fract() != 0.0 || b.fract() != 0.0 {
        return Err(CalcError::Domain("Batas Σ harus bilangan bulat".into()));
    }
    if b < a {
        return Ok(0.0);
    }
    if b - a >= 1_000_000.0 {
        return Err(CalcError::Unsupported(
            "Rentang Σ terlalu besar (maks. 1.000.000 suku)".into(),
        ));
    }
    let mut total = 0.0;
    let mut k = a;
    while k <= b {
        total += finite(ctx.eval_at(&args[0], v, k)?)?;
        k += 1.0;
    }
    Ok(total)
}

// ------------------------------------------------------------------ limit

/// Limit sepihak dengan ekstrapolasi polinomial (Neville) pada h → 0.
/// `dir` = +1 (dari kanan) atau -1 (dari kiri). Untuk a = ±∞ dipakai x = ±1/t, t → 0⁺.
fn one_sided(ctx: &mut Ctx, f: &Expr, v: char, a: f64, dir: f64) -> Result<f64, CalcError> {
    const N: usize = 7;
    let h0 = if a.is_infinite() {
        0.1
    } else {
        0.1 * a.abs().max(1.0)
    };
    let mut g = [0.0_f64; N];
    for (k, slot) in g.iter_mut().enumerate() {
        let h = h0 / f64::powi(2.0, k as i32);
        let x = if a.is_infinite() {
            a.signum() / h
        } else {
            a + dir * h
        };
        *slot = ctx.eval_at(f, v, x)?;
        if slot.is_nan() {
            return Err(CalcError::Domain("Limit tidak terdefinisi".into()));
        }
    }

    // Divergen: besarnya terus membesar dengan laju konstan (seperti 1/x di 0) dan sudah besar.
    let last = g[N - 1];
    let grows = |hi: f64, lo: f64| hi.abs() > 1.3 * lo.abs();
    if last.abs() > 20.0 && grows(g[N - 1], g[N - 2]) && grows(g[N - 2], g[N - 3]) {
        return Ok(last.signum() * f64::INFINITY);
    }
    if g.iter().any(|x| !x.is_finite()) {
        return Err(CalcError::Domain("Limit tidak terdefinisi".into()));
    }

    // Tabel Neville (rasio langkah 2).
    let mut t = g;
    let mut prev_best = g[N - 1];
    for j in 1..N {
        let factor = f64::powi(2.0, j as i32) - 1.0;
        for k in (j..N).rev() {
            t[k] = t[k] + (t[k] - t[k - 1]) / factor;
        }
        if j == N - 2 {
            prev_best = t[N - 1];
        }
    }
    let best = t[N - 1];
    let tol = 1e-6 * (1.0 + best.abs());
    if (best - prev_best).abs() <= tol {
        Ok(best)
    } else {
        non_smooth(ctx, f, v, a, dir)
    }
}

/// Cadangan untuk fungsi tidak mulus (mis. √x di 0): sampel pada h yang sangat kecil
/// dan terima hanya bila selisih antar-sampel menyusut.
fn non_smooth(ctx: &mut Ctx, f: &Expr, v: char, a: f64, dir: f64) -> Result<f64, CalcError> {
    let scale = if a.is_infinite() {
        1.0
    } else {
        a.abs().max(1.0)
    };
    let mut g = [0.0_f64; 3];
    for (k, slot) in g.iter_mut().enumerate() {
        let h = scale * 10f64.powi(-9 - 2 * k as i32);
        let x = if a.is_infinite() {
            a.signum() / h
        } else {
            a + dir * h
        };
        *slot = ctx.eval_at(f, v, x)?;
    }
    let (d1, d2) = ((g[1] - g[0]).abs(), (g[2] - g[1]).abs());
    if g.iter().all(|x| x.is_finite()) && d2 <= d1 && d2 <= 1e-4 * (1.0 + g[2].abs()) {
        Ok(if g[2].abs() < 1e-6 { 0.0 } else { g[2] })
    } else {
        Err(CalcError::Domain("Limit tidak konvergen".into()))
    }
}

pub fn limit(ctx: &mut Ctx, name: &str, args: &[Expr]) -> Result<f64, CalcError> {
    let v = var_arg(&args[1])?;
    let a = ctx.eval(&args[2])?;
    if a.is_nan() {
        return Err(CalcError::Domain("Titik limit tidak valid".into()));
    }
    let f = &args[0];
    if a.is_infinite() {
        return Ok(clean(one_sided(ctx, f, v, a, 1.0)?, 8, 1e-9));
    }
    let r = match name {
        "limr" => one_sided(ctx, f, v, a, 1.0)?,
        "liml" => one_sided(ctx, f, v, a, -1.0)?,
        _ => {
            let l = one_sided(ctx, f, v, a, -1.0)?;
            let r = one_sided(ctx, f, v, a, 1.0)?;
            match (l.is_infinite(), r.is_infinite()) {
                (true, true) if l == r => l,
                (false, false) if (l - r).abs() <= 1e-5 * (1.0 + l.abs().max(r.abs())) => {
                    (l + r) / 2.0
                }
                _ => {
                    return Err(CalcError::Domain(
                        "Limit tidak ada (sisi kiri ≠ sisi kanan)".into(),
                    ))
                }
            }
        }
    };
    Ok(clean(r, 8, 1e-9))
}

// ---------------------------------------------------------------- integral

// Kuadratur Gauss–Kronrod G7K15 (QUADPACK qk15), setengah kanan simpul.
const XGK: [f64; 8] = [
    0.991_455_371_120_812_6,
    0.949_107_912_342_758_5,
    0.864_864_423_359_769_1,
    0.741_531_185_599_394_4,
    0.586_087_235_467_691_1,
    0.405_845_151_377_397_2,
    0.207_784_955_007_898_47,
    0.0,
];
const WGK: [f64; 8] = [
    0.022_935_322_010_529_225,
    0.063_092_092_629_978_55,
    0.104_790_010_322_250_18,
    0.140_653_259_715_525_92,
    0.169_004_726_639_267_9,
    0.190_350_578_064_785_4,
    0.204_432_940_075_298_9,
    0.209_482_141_084_727_83,
];
const WG: [f64; 4] = [
    0.129_484_966_168_869_7,
    0.279_705_391_489_276_7,
    0.381_830_050_505_118_9,
    0.417_959_183_673_469_4,
];

type Integrand<'a> = dyn FnMut(f64) -> Result<f64, CalcError> + 'a;

/// Satu aturan G7K15 pada [a, b] → (nilai, galat, ∫|f|).
fn gk15(f: &mut Integrand, a: f64, b: f64) -> Result<(f64, f64, f64), CalcError> {
    let c = 0.5 * (a + b);
    let h = 0.5 * (b - a);
    let fc = finite(f(c)?)?;
    let mut kron = fc * WGK[7];
    let mut gauss = fc * WG[3];
    let mut abs = fc.abs() * WGK[7];
    for j in 0..7 {
        let dx = h * XGK[j];
        let f1 = finite(f(c - dx)?)?;
        let f2 = finite(f(c + dx)?)?;
        kron += WGK[j] * (f1 + f2);
        abs += WGK[j] * (f1.abs() + f2.abs());
        if j % 2 == 1 {
            gauss += WG[j / 2] * (f1 + f2);
        }
    }
    Ok((kron * h, ((kron - gauss) * h).abs(), abs * h.abs()))
}

/// Integrasi adaptif pada interval berhingga.
fn adaptive(f: &mut Integrand, a: f64, b: f64) -> Result<f64, CalcError> {
    let (first, _, _) = gk15(f, a, b)?;
    let tol = 1e-11 * 1.0_f64.max(first.abs());
    let mut stack = vec![(a, b)];
    let mut total = 0.0;
    let mut pieces = 0;
    while let Some((lo, hi)) = stack.pop() {
        let (r, err, abs) = gk15(f, lo, hi)?;
        pieces += 1;
        if err <= tol * ((hi - lo) / (b - a)).abs().max(1e-3) || err <= 1e-14 * abs {
            total += r;
        } else if pieces > 3000 || (hi - lo).abs() < 1e-12 * (b - a).abs() {
            return Err(CalcError::Domain("Integral tidak konvergen".into()));
        } else {
            let m = 0.5 * (lo + hi);
            stack.push((lo, m));
            stack.push((m, hi));
        }
    }
    Ok(total)
}

pub fn integral(ctx: &mut Ctx, args: &[Expr]) -> Result<f64, CalcError> {
    let v = var_arg(&args[1])?;
    let a = ctx.eval(&args[2])?;
    let b = ctx.eval(&args[3])?;
    if a.is_nan() || b.is_nan() {
        return Err(CalcError::Domain("Batas integral tidak valid".into()));
    }
    if a == b {
        return Ok(0.0);
    }
    if a > b {
        let swapped = [
            args[0].clone(),
            args[1].clone(),
            args[3].clone(),
            args[2].clone(),
        ];
        return integral(ctx, &swapped).map(|r| -r);
    }

    let body = &args[0];
    let mut g = |x: f64| ctx.eval_at(body, v, x);
    let r = match (a.is_infinite(), b.is_infinite()) {
        (false, false) => adaptive(&mut g, a, b)?,
        (false, true) => adaptive(
            &mut |t: f64| {
                let x = a + t / (1.0 - t);
                Ok(g(x)? / ((1.0 - t) * (1.0 - t)))
            },
            0.0,
            1.0,
        )?,
        (true, false) => adaptive(
            &mut |t: f64| {
                let x = b - t / (1.0 - t);
                Ok(g(x)? / ((1.0 - t) * (1.0 - t)))
            },
            0.0,
            1.0,
        )?,
        (true, true) => adaptive(
            &mut |t: f64| {
                let d = 1.0 - t * t;
                let x = t / d;
                Ok(g(x)? * (1.0 + t * t) / (d * d))
            },
            -1.0,
            1.0,
        )?,
    };
    Ok(clean(r, 10, 1e-12))
}
