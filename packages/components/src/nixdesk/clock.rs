use crate::languages::{use_language, Calendar, Strings};
use crate::platform::{Date, TimeoutFuture};
use dioxus::prelude::*;

/// 24-hour ("19:52") or 12-hour ("07:52 PM") display. One value shared by
/// the whole desktop simulation (navbar clock, terminal clock panel and the
/// shell's `date` command) through a context signal, so switching it in one
/// place changes it everywhere, and it survives the Desktop <-> Terminal
/// switch because the provider lives in `MainPage`.
///
/// Later, the Settings window can read/write the same signal
/// (`use_clock_format()`) to offer this as a preference.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ClockFormat {
    #[default]
    H24,
    H12,
}

impl ClockFormat {
    pub fn toggled(self) -> Self {
        match self {
            ClockFormat::H24 => ClockFormat::H12,
            ClockFormat::H12 => ClockFormat::H24,
        }
    }

    /// What clicking the clock will switch *to* (for tooltip / aria-label).
    pub fn switch_label(self, s: &Strings) -> &'static str {
        match self {
            ClockFormat::H24 => s.clock.switch_to_12h,
            ClockFormat::H12 => s.clock.switch_to_24h,
        }
    }
}

/// Call once, high in the tree (`MainPage`).
pub fn use_clock_format_provider() -> Signal<ClockFormat> {
    use_context_provider(|| Signal::new(ClockFormat::default()))
}

/// Read/write the shared clock format from any descendant of `MainPage`.
pub fn use_clock_format() -> Signal<ClockFormat> {
    use_context::<Signal<ClockFormat>>()
}

/// Pure formatter: `(19, 7, H24)` -> "19:07", `(19, 7, H12)` -> "07:07 PM".
/// Sufiks AM/PM diambil dari `calendar` (bahasa aktif).
pub fn format_hm(hours24: u32, minutes: u32, format: ClockFormat, calendar: &Calendar) -> String {
    match format {
        ClockFormat::H24 => format!("{hours24:02}:{minutes:02}"),
        ClockFormat::H12 => {
            let (hours12, suffix) = match hours24 {
                0 => (12, calendar.am),
                1..=11 => (hours24, calendar.am),
                12 => (12, calendar.pm),
                _ => (hours24 - 12, calendar.pm),
            };
            format!("{hours12:02}:{minutes:02} {suffix}")
        }
    }
}

pub fn format_time(date: &Date, format: ClockFormat, s: &Strings) -> String {
    format_hm(date.get_hours(), date.get_minutes(), format, &s.calendar)
}

/// "Monday, September 14, 2026"
pub fn format_date(date: &Date, s: &Strings) -> String {
    let weekday = s.calendar.weekday(date.get_day() as usize);
    let month = s.calendar.month(date.get_month() as usize);
    let day = date.get_date();
    let year = date.get_full_year();
    format!("{weekday}, {month} {day}, {year}")
}

/// Ticks a `(time, date)` signal pair every second off the real system
/// clock. Shared by DesktopMode's navbar and TerminalMode's clock panel
/// so each caller starts exactly one timer and every consumer reads the
/// same live signals.
pub fn use_live_clock() -> (Signal<String>, Signal<String>) {
    let format = use_clock_format();
    let language = use_language();
    let mut time = use_signal(|| format_time(&Date::new_0(), format(), language().strings()));
    let mut date = use_signal(|| format_date(&Date::new_0(), language().strings()));

    // Re-render the text right away when the format or the language changes,
    // instead of waiting up to a second for the next tick. Only `format` and
    // `language` are tracked here (`time.set` is a write).
    use_effect(move || {
        let f = format();
        let s = language().strings();
        let now = Date::new_0();
        time.set(format_time(&now, f, s));
        date.set(format_date(&now, s));
    });

    use_effect(move || {
        spawn(async move {
            loop {
                TimeoutFuture::new(1000).await;
                let now = Date::new_0();
                let s = language.peek().strings();
                time.set(format_time(&now, *format.peek(), s));
                date.set(format_date(&now, s));
            }
        });
    });

    (time, date)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::languages::{Language, EN};

    #[test]
    fn twenty_four_hour_is_zero_padded() {
        assert_eq!(format_hm(0, 5, ClockFormat::H24, &EN.calendar), "00:05");
        assert_eq!(format_hm(9, 30, ClockFormat::H24, &EN.calendar), "09:30");
        assert_eq!(format_hm(19, 52, ClockFormat::H24, &EN.calendar), "19:52");
        assert_eq!(format_hm(23, 59, ClockFormat::H24, &EN.calendar), "23:59");
    }

    #[test]
    fn twelve_hour_handles_midnight_noon_and_afternoon() {
        let c = &EN.calendar;
        assert_eq!(format_hm(0, 5, ClockFormat::H12, c), "12:05 AM");
        assert_eq!(format_hm(1, 0, ClockFormat::H12, c), "01:00 AM");
        assert_eq!(format_hm(11, 59, ClockFormat::H12, c), "11:59 AM");
        assert_eq!(format_hm(12, 0, ClockFormat::H12, c), "12:00 PM");
        assert_eq!(format_hm(13, 7, ClockFormat::H12, c), "01:07 PM");
        assert_eq!(format_hm(23, 59, ClockFormat::H12, c), "11:59 PM");
    }

    #[test]
    fn toggle_flips_and_defaults_to_24_hour() {
        assert_eq!(ClockFormat::default(), ClockFormat::H24);
        assert_eq!(ClockFormat::H24.toggled(), ClockFormat::H12);
        assert_eq!(ClockFormat::H12.toggled().toggled(), ClockFormat::H12);
        let s = Language::En.strings();
        assert_ne!(
            ClockFormat::H24.switch_label(s),
            ClockFormat::H12.switch_label(s)
        );
    }
}
