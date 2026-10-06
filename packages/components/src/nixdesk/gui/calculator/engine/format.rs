//! Pemformatan angka: 10 digit signifikan, koma desimal, notasi `m×10^e` untuk angka ekstrem.
//! Hasilnya sengaja berupa ekspresi yang valid, sehingga bisa dipakai lagi sebagai masukan.

pub fn format_number(v: f64) -> String {
    if v.is_nan() {
        return "Error".to_string();
    }
    if v.is_infinite() {
        return if v > 0.0 { "∞" } else { "-∞" }.to_string();
    }
    if v == 0.0 {
        return "0".to_string();
    }

    // Bilangan bulat tampil utuh (mis. 123456789012), bukan dibulatkan ke 10 digit.
    if v.fract() == 0.0 && v.abs() < 1e15 {
        return format!("{v:.0}");
    }

    // "d.ddddddddde±x": eksponen diambil dari string agar tidak bergantung pada log10.
    let sci = format!("{:.9e}", v);
    let (mant, exp) = sci.split_once('e').unwrap_or((&sci, "0"));
    let exp: i32 = exp.parse().unwrap_or(0);

    if (-7..=14).contains(&exp) {
        let rounded: f64 = sci.parse().unwrap_or(v);
        let decimals = (9 - exp).max(0) as usize;
        let s = format!("{:.*}", decimals, rounded);
        trim_zeros(&s).replace('.', ",")
    } else {
        let m = trim_zeros(mant).replace('.', ",");
        format!("{m}×10^{exp}")
    }
}

fn trim_zeros(s: &str) -> String {
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s.to_string()
    }
}
