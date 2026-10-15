//! Waktu relatif ("3 minutes ago") untuk Live Chat dan preview chat di mode
//! Terminal. Sebelumnya fungsi ini disalin di dua file; sekarang satu
//! implementasi murni (`relative_time_between`) yang bisa diuji tanpa jam.

use crate::languages::{Strings, TimeUnit};
use crate::platform::Date;

/// Selisih `then_ms` -> `now_ms` (milidetik epoch) dalam kata-kata.
/// Waktu di masa depan atau tak valid (NaN) dianggap "just now".
pub fn relative_time_between(then_ms: f64, now_ms: f64, s: &Strings) -> String {
    let diff_secs = ((now_ms - then_ms) / 1000.0).max(0.0);

    let minutes = diff_secs / 60.0;
    let hours = minutes / 60.0;
    let days = hours / 24.0;
    let weeks = days / 7.0;
    let months = days / 30.0;
    let years = days / 365.0;

    if minutes < 1.0 {
        s.time.just_now.to_string()
    } else if minutes < 60.0 {
        s.time.ago(minutes as u32, TimeUnit::Minute)
    } else if hours < 24.0 {
        s.time.ago(hours as u32, TimeUnit::Hour)
    } else if days < 7.0 {
        s.time.ago(days as u32, TimeUnit::Day)
    } else if weeks < 5.0 {
        s.time.ago(weeks as u32, TimeUnit::Week)
    } else if months < 12.0 {
        s.time.ago(months as u32, TimeUnit::Month)
    } else {
        s.time.ago(years as u32, TimeUnit::Year)
    }
}

/// `iso` (RFC 3339) terhadap waktu sekarang.
pub fn relative_time(iso: &str, s: &Strings) -> String {
    relative_time_between(Date::from_iso(iso).get_time(), Date::now(), s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::languages::Language;

    const MIN: f64 = 60.0 * 1000.0;
    const HOUR: f64 = 60.0 * MIN;
    const DAY: f64 = 24.0 * HOUR;
    const NOW: f64 = 2_000_000_000_000.0;

    fn ago(ms: f64) -> String {
        relative_time_between(NOW - ms, NOW, Language::En.strings())
    }

    #[test]
    fn just_now_covers_under_a_minute_future_and_invalid() {
        assert_eq!(ago(0.0), "just now");
        assert_eq!(ago(59_000.0), "just now");
        let s = Language::En.strings();
        assert_eq!(relative_time_between(NOW + HOUR, NOW, s), "just now");
        assert_eq!(relative_time_between(f64::NAN, NOW, s), "just now");
    }

    #[test]
    fn minutes_and_hours_pluralise() {
        assert_eq!(ago(MIN), "1 minute ago");
        assert_eq!(ago(2.0 * MIN), "2 minutes ago");
        assert_eq!(ago(59.0 * MIN), "59 minutes ago");
        assert_eq!(ago(HOUR), "1 hour ago");
        assert_eq!(ago(23.0 * HOUR), "23 hours ago");
    }

    #[test]
    fn days_weeks_months_years() {
        assert_eq!(ago(DAY), "1 day ago");
        assert_eq!(ago(6.0 * DAY), "6 days ago");
        assert_eq!(ago(7.0 * DAY), "1 week ago");
        assert_eq!(ago(28.0 * DAY), "4 weeks ago");
        // 5 weeks (35 days) is past the week bucket, ~1 month
        assert_eq!(ago(35.0 * DAY), "1 month ago");
        assert_eq!(ago(90.0 * DAY), "3 months ago");
        assert_eq!(ago(365.0 * DAY), "1 year ago");
        assert_eq!(ago(800.0 * DAY), "2 years ago");
    }

    #[test]
    fn iso_input_goes_through_the_same_buckets() {
        // an ancient timestamp must read as years, an invalid one as "just now"
        let s = Language::En.strings();
        assert!(relative_time("2001-01-01T00:00:00Z", s).ends_with("years ago"));
        assert_eq!(relative_time("not a date", s), "just now");
    }
}
