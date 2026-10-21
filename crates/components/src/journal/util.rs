use api_types::journal::Day;

use crate::languages::Strings;

/// Nama bulan pendek untuk tanggal pendek di grafik (`"Sep 14"`, format `en-US`).
const MONTHS_SHORT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Mirrors the JS `formatStars` helper: 1234 -> "1.2K", 42 -> "42".
pub fn format_stars(count: u32) -> String {
    if count < 1000 {
        count.to_string()
    } else {
        format!("{:.1}K", count as f64 / 1000.0)
    }
}

/// Mirrors `c.total_contribution_volume.toLocaleString()` (en-US thousands grouping).
pub fn format_volume(n: u32) -> String {
    let digits = n.to_string();
    let mut grouped = String::new();
    for (i, ch) in digits.chars().rev().enumerate() {
        if i != 0 && i % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    grouped.chars().rev().collect()
}

/// Formats an ISO `YYYY-MM-DD` date the way `toLocaleDateString("en-US", { month: "short", day: "numeric" })` does.
pub fn format_short_date(iso_date: &str) -> String {
    let parts: Vec<&str> = iso_date.split('-').collect();
    if parts.len() < 3 {
        return iso_date.to_string();
    }
    let month_idx = parts[1]
        .parse::<usize>()
        .unwrap_or(1)
        .saturating_sub(1)
        .min(11);
    let day: u32 = parts[2].parse().unwrap_or(1);
    format!("{} {}", MONTHS_SHORT[month_idx], day)
}

/// Rolling 12-month label row starting from the first timeline entry's month,
/// same as the JS `months` generation. Labels follow the UI language.
pub fn build_month_labels(timeline: &[Day], s: &Strings) -> Vec<String> {
    let Some(first) = timeline.first() else {
        return vec![];
    };
    let parts: Vec<&str> = first.date.split('-').collect();
    let start_month = parts
        .get(1)
        .and_then(|m| m.parse::<usize>().ok())
        .unwrap_or(1)
        .saturating_sub(1)
        % 12;
    (0..12)
        .map(|i| s.calendar.month_short((start_month + i) % 12).to_string())
        .collect()
}

/// Today's date, computed from the local clock (`crate::platform::Date`, equivalent to JS `new Date()`).
pub struct Today {
    pub month_upper: String,
    pub day: u32,
    pub year: i32,
    pub day_of_year: u32,
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub fn today(s: &Strings) -> Today {
    let now = crate::platform::Date::new_0();
    let month0 = now.get_month() as usize; // 0-indexed, like JS
    let day = now.get_date();
    let year = now.get_full_year() as i32;

    const DAYS_IN_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut day_of_year = day;
    for (m, days) in DAYS_IN_MONTH.iter().enumerate().take(month0) {
        day_of_year += days;
        if m == 1 && is_leap_year(year) {
            day_of_year += 1;
        }
    }

    Today {
        month_upper: s.calendar.month_short(month0).to_uppercase(),
        day,
        year,
        day_of_year,
    }
}

/// `+12.5%` / `-3.0%` / `+0.0%` (satu desimal, tanda selalu ada).
pub fn format_growth(percent: f64) -> String {
    let p = if percent.is_finite() { percent } else { 0.0 };
    format!("{p:+.1}%")
}

/// Indeks bulan 0..=11 dari `YYYY-MM-...`; `None` jika bentuknya salah.
pub fn month_index(iso: &str) -> Option<usize> {
    let month: usize = iso.get(5..7)?.parse().ok()?;
    (1..=12).contains(&month).then(|| month - 1)
}

/// Sepotong teks kalimat; `highlight` = ditonjolkan (warna aksen).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub text: String,
    pub highlight: bool,
}

/// Mengisi `{nama}` pada templat teks UI menjadi potongan-potongan, supaya
/// bagian tertentu bisa ditonjolkan **tanpa membuat string HTML**: tiap
/// potongan nanti menjadi teks atau `span` biasa di RSX (Dioxus yang meng-escape).
/// Satu putaran: isi yang berisi `{x}` tidak diproses lagi; placeholder yang
/// tidak dikenal dibiarkan apa adanya.
pub fn fill_segments(template: &str, fills: &[(&str, &str, bool)]) -> Vec<Segment> {
    let mut out: Vec<Segment> = Vec::new();
    let mut plain = String::new();
    let mut rest = template;
    let flush = |plain: &mut String, out: &mut Vec<Segment>| {
        if !plain.is_empty() {
            out.push(Segment {
                text: std::mem::take(plain),
                highlight: false,
            });
        }
    };
    while let Some(start) = rest.find('{') {
        plain.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else {
            plain.push_str(&rest[start..]);
            rest = "";
            break;
        };
        let name = &after[..end];
        match fills.iter().find(|(k, _, _)| *k == name) {
            Some((_, value, true)) => {
                flush(&mut plain, &mut out);
                out.push(Segment {
                    text: (*value).to_owned(),
                    highlight: true,
                });
            }
            Some((_, value, false)) => plain.push_str(value),
            None => {
                plain.push('{');
                plain.push_str(name);
                plain.push('}');
            }
        }
        rest = &after[end + 1..];
    }
    plain.push_str(rest);
    flush(&mut plain, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::languages::Language;

    fn seg(text: &str, highlight: bool) -> Segment {
        Segment {
            text: text.into(),
            highlight,
        }
    }

    #[test]
    fn growth_always_has_a_sign_and_one_decimal() {
        assert_eq!(format_growth(12.5), "+12.5%");
        assert_eq!(format_growth(-3.0), "-3.0%");
        assert_eq!(format_growth(0.0), "+0.0%");
        assert_eq!(format_growth(18.04), "+18.0%");
        assert_eq!(format_growth(f64::NAN), "+0.0%");
    }

    #[test]
    fn month_index_reads_only_well_formed_dates() {
        assert_eq!(month_index("2026-10-20T12:00:00Z"), Some(9));
        assert_eq!(month_index("2026-01-01"), Some(0));
        assert_eq!(month_index("2026-12-31"), Some(11));
        for bad in ["", "2026", "2026-13-01", "2026-00-01", "2026-xx-01"] {
            assert_eq!(month_index(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn segments_highlight_only_marked_values_and_keep_the_rest_as_text() {
        let got = fill_segments(
            "{m} saw {a}, in {l} for {r} setup.",
            &[
                ("m", "October", false),
                ("a", "tooling", false),
                ("l", "Nix", true),
                ("r", "flexinix", true),
            ],
        );
        assert_eq!(
            got,
            [
                seg("October saw tooling, in ", false),
                seg("Nix", true),
                seg(" for ", false),
                seg("flexinix", true),
                seg(" setup.", false),
            ]
        );
    }

    #[test]
    fn values_are_never_interpreted_again_so_markup_stays_inert_text() {
        let got = fill_segments(
            "a {x} b {y}",
            &[("x", "{y}", true), ("y", "<b>hi</b>", false)],
        );
        assert_eq!(
            got,
            [
                seg("a ", false),
                seg("{y}", true),
                seg(" b <b>hi</b>", false)
            ]
        );
    }

    #[test]
    fn unknown_unclosed_and_empty_templates_are_left_alone() {
        assert_eq!(fill_segments("a {zzz} b", &[]), [seg("a {zzz} b", false)]);
        assert_eq!(fill_segments("open {", &[]), [seg("open {", false)]);
        assert_eq!(fill_segments("{x}", &[("x", "v", true)]), [seg("v", true)]);
        assert!(fill_segments("", &[]).is_empty());
    }

    #[test]
    fn the_real_focus_templates_produce_the_expected_sentence() {
        let s = Language::En.strings();
        let got = fill_segments(
            s.journal.focus_sentence,
            &[
                ("month", s.calendar.month(9), false),
                ("activity", s.journal.activity_for(Some("Nix")), false),
                ("language", "Nix", true),
                ("repo", "flexinix", true),
            ],
        );
        let text: String = got.iter().map(|g| g.text.as_str()).collect();
        assert_eq!(
            text,
            "October saw a significant shift towards reproducible infrastructure, with heavy activity in Nix configurations for the flexinix setup."
        );
        assert_eq!(got.iter().filter(|g| g.highlight).count(), 2);

        let empty = fill_segments(
            s.journal.focus_empty,
            &[("ideas", s.journal.focus_ideas, true)],
        );
        let text: String = empty.iter().map(|g| g.text.as_str()).collect();
        assert_eq!(text, "Structuring ideas into reality.");
    }
}
