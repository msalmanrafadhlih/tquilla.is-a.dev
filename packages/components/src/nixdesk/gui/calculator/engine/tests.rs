use super::*;

fn num(s: &str) -> f64 {
    num_a(s, Angle::Rad)
}

fn num_a(s: &str, a: Angle) -> f64 {
    match evaluate(s, a) {
        Ok(Outcome::Number(v)) => v,
        other => panic!("{s:?} → {other:?}"),
    }
}

fn text(s: &str, a: Angle) -> String {
    match evaluate(s, a) {
        Ok(o) => o.to_text(),
        Err(e) => format!("ERR: {e}"),
    }
}

fn close(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() <= tol * (1.0 + b.abs())
}

macro_rules! assert_close {
    ($expr:expr, $want:expr, $tol:expr) => {{
        let got = num($expr);
        assert!(close(got, $want, $tol), "{} = {} (harusnya {})", $expr, got, $want);
    }};
}

#[test]
fn aritmetika_dasar() {
    assert_eq!(num("2+98"), 100.0);
    assert_eq!(num("2+3×4"), 14.0);
    assert_eq!(num("(2+3)×4"), 20.0);
    assert_eq!(num("10÷4"), 2.5);
    assert_eq!(num("7-10"), -3.0);
    assert_eq!(num("-2^2"), -4.0);
    assert_eq!(num("2^3^2"), 512.0);
    assert_eq!(num("2^-1"), 0.5);
    assert_eq!(num("5!"), 120.0);
    assert_eq!(num("50%"), 0.5);
    assert_eq!(num("0,1+0,2"), 0.1 + 0.2);
}

#[test]
fn perkalian_implisit_dan_kurung_terbuka() {
    assert_eq!(num("2(3+4)"), 14.0);
    assert_eq!(num("(2+3)(4)"), 20.0);
    assert_close!("2π", 2.0 * std::f64::consts::PI, 1e-12);
    assert_eq!(num("3×(2+1"), 9.0); // kurung belum ditutup
    assert_eq!(num("sqrt(16"), 4.0);
}

#[test]
fn error_aritmetika() {
    assert_eq!(text("1÷0", Angle::Rad), "ERR: Tidak bisa membagi dengan nol");
    assert!(text("(-1)!", Angle::Rad).starts_with("ERR"));
    assert!(text("ln(0)", Angle::Rad).starts_with("ERR"));
    assert!(text("√(-4)", Angle::Rad).starts_with("ERR"));
    assert!(text("x+1", Angle::Rad).contains("Variabel x"));
    assert!(text("2+", Angle::Rad).starts_with("ERR"));
    assert!(text("2+)", Angle::Rad).starts_with("ERR"));
    assert!(text("171!", Angle::Rad).starts_with("ERR"));
    assert!(text("10^400", Angle::Rad).starts_with("ERR"));
    assert_eq!(evaluate("", Angle::Rad), Err(CalcError::Empty));
}

#[test]
fn fungsi_aljabar() {
    assert_eq!(num("abs(-5)"), 5.0);
    assert_eq!(num("√(81)"), 9.0);
    assert_eq!(num("root(3;27)"), 3.0);
    assert_eq!(num("root(3;-8)"), -2.0);
    assert_eq!(num("log(1000)"), 3.0);
    assert_eq!(num("log(2;8)"), 3.0);
    assert_close!("ln(e)", 1.0, 1e-12);
    assert_eq!(num("frac(1;4)"), 0.25);
    assert_eq!(num("nCr(5;2)"), 10.0);
    assert_eq!(num("nPr(5;2)"), 20.0);
    assert_eq!(num("nCr(52;5)"), 2_598_960.0);
}

#[test]
fn perbandingan() {
    let t = |s: &str| text(s, Angle::Rad);
    assert_eq!(t("2+2=4"), "Benar");
    assert_eq!(t("2<1"), "Salah");
    assert_eq!(t("3≤3"), "Benar");
    assert_eq!(t("5≥6"), "Salah");
    assert_eq!(t("0,1+0,2=0,3"), "Benar");
}

#[test]
fn trigonometri_radian_dan_derajat() {
    assert_close!("sin(π÷2)", 1.0, 1e-12);
    assert_eq!(num("sin(π)"), 0.0);
    assert!(close(num_a("sin(30)", Angle::Deg), 0.5, 1e-12));
    assert!(close(num_a("cos(60)", Angle::Deg), 0.5, 1e-12));
    assert_eq!(num_a("cos(90)", Angle::Deg), 0.0);
    assert_eq!(num_a("sin(180)", Angle::Deg), 0.0);
    assert!(close(num_a("tan(45)", Angle::Deg), 1.0, 1e-12));
    assert!(text("tan(90)", Angle::Deg).starts_with("ERR"));
    assert!(close(num_a("arcsin(0,5)", Angle::Deg), 30.0, 1e-12));
    assert!(close(num_a("arctan(1)", Angle::Deg), 45.0, 1e-12));
    assert!(close(num("arctan(1)"), std::f64::consts::FRAC_PI_4, 1e-12));
    assert!(close(num_a("sec(60)", Angle::Deg), 2.0, 1e-12));
    // satuan derajat eksplisit
    assert!(close(num("sin(30°)"), 0.5, 1e-12));
    assert!(close(num_a("sin(30°)", Angle::Deg), 0.5, 1e-12));
    assert_close!("sin(2)^2+cos(2)^2", 1.0, 1e-12);
    assert!(text("arcsin(2)", Angle::Rad).starts_with("ERR"));
}

#[test]
fn turunan_numerik() {
    assert_close!("diff(x^2;x;3)", 6.0, 1e-8);
    assert_close!("diff(sin(x);x;0)", 1.0, 1e-8);
    assert_close!("diff(e^x;x;1)", std::f64::consts::E, 1e-8);
    assert_close!("diff(x^3-2x;x;2)", 10.0, 1e-8);
    assert_eq!(num("diff(x^2;x;0)"), 0.0);
    assert_eq!(num("diff(x^2-2x;x;1)"), 0.0);
    assert_close!("diff(ln(x);x;2)", 0.5, 1e-8);
}

#[test]
fn deret_sigma() {
    assert_eq!(num("sum(k;k;1;100)"), 5050.0);
    assert_eq!(num("sum(k^2;k;1;10)"), 385.0);
    assert_eq!(num("sum(k;k;5;1)"), 0.0);
    assert!(text("sum(k;k;1;∞)", Angle::Rad).starts_with("ERR"));
}

#[test]
fn integral_tentu() {
    assert_close!("int(x^2;x;0;1)", 1.0 / 3.0, 1e-9);
    assert_close!("int(sin(x);x;0;π)", 2.0, 1e-9);
    assert_eq!(num("int(sin(x);x;0;2π)"), 0.0);
    assert_close!("int(e^x;x;0;1)", std::f64::consts::E - 1.0, 1e-9);
    assert_close!("int(1÷x;x;1;e)", 1.0, 1e-9);
    assert_close!("int(x;x;3;1)", -4.0, 1e-9); // batas terbalik
    assert_close!("int(e^-x;x;0;∞)", 1.0, 1e-8);
    assert_close!("int(1÷x^2;x;1;∞)", 1.0, 1e-8);
    assert_close!("int(e^(-x^2);x;-∞;∞)", std::f64::consts::PI.sqrt(), 1e-8);
    assert_close!("int(sqrt(x);x;0;4)", 16.0 / 3.0, 1e-8);
    assert!(text("int(1÷x;x;0;1)", Angle::Rad).starts_with("ERR"));
}

#[test]
fn limit_numerik() {
    assert_close!("lim(sin(x)÷x;x;0)", 1.0, 1e-6);
    assert_close!("lim((1-cos(x))÷x^2;x;0)", 0.5, 1e-5);
    assert_close!("lim((x^2-1)÷(x-1);x;1)", 2.0, 1e-6);
    assert_close!("lim((1+1÷x)^x;x;∞)", std::f64::consts::E, 1e-5);
    assert_close!("lim(1÷x;x;∞)", 0.0, 1e-6);
    assert_close!("limr(sqrt(x);x;0)", 0.0, 1e-3);
    assert_eq!(text("limr(1÷x;x;0)", Angle::Rad), "∞");
    assert_eq!(text("liml(1÷x;x;0)", Angle::Rad), "-∞");
    assert_eq!(text("lim(1÷x^2;x;0)", Angle::Rad), "∞");
    assert!(text("lim(1÷x;x;0)", Angle::Rad).starts_with("ERR"));
    assert!(text("lim(abs(x)÷x;x;0)", Angle::Rad).starts_with("ERR"));
}

#[test]
fn format_angka() {
    assert_eq!(format_number(100.0), "100");
    assert_eq!(format_number(0.1 + 0.2), "0,3");
    assert_eq!(format_number(-2.5), "-2,5");
    assert_eq!(format_number(1.0 / 3.0), "0,3333333333");
    assert_eq!(format_number(1e20), "1×10^20");
    assert_eq!(format_number(1.5e-9), "1,5×10^-9");
    assert_eq!(format_number(123456789012.0), "123456789012");
    assert_eq!(format_number(2.0_f64.sqrt()), "1,414213562");
    // hasil yang diformat bisa dipakai lagi sebagai masukan
    for v in [1e20, 1.5e-9, -2.5, 1.0 / 3.0] {
        let s = format_number(v);
        assert!(close(num(&s), v, 1e-9), "{s}");
    }
}

#[test]
fn angka_polos() {
    assert!(is_plain_number("42"));
    assert!(is_plain_number("-3,5"));
    assert!(!is_plain_number("2+3"));
    assert!(!is_plain_number("π"));
}
