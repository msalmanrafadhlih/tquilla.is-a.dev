use crate::platform::{Date, TimeoutFuture};
use dioxus::prelude::*;

const WEEKDAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

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
    pub fn switch_label(self) -> &'static str {
        match self {
            ClockFormat::H24 => "Switch to 12-hour clock",
            ClockFormat::H12 => "Switch to 24-hour clock",
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
pub fn format_hm(hours24: u32, minutes: u32, format: ClockFormat) -> String {
    match format {
        ClockFormat::H24 => format!("{hours24:02}:{minutes:02}"),
        ClockFormat::H12 => {
            let (hours12, suffix) = match hours24 {
                0 => (12, "AM"),
                1..=11 => (hours24, "AM"),
                12 => (12, "PM"),
                _ => (hours24 - 12, "PM"),
            };
            format!("{hours12:02}:{minutes:02} {suffix}")
        }
    }
}

pub fn format_time(date: &Date, format: ClockFormat) -> String {
    format_hm(date.get_hours(), date.get_minutes(), format)
}

/// "Monday, September 14, 2026"
pub fn format_date(date: &Date) -> String {
    let weekday = WEEKDAYS[date.get_day() as usize];
    let month = MONTHS[date.get_month() as usize];
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
    let mut time = use_signal(|| format_time(&Date::new_0(), format()));
    let mut date = use_signal(|| format_date(&Date::new_0()));

    // Re-render the text right away when the format changes, instead of
    // waiting up to a second for the next tick. Only `format` is tracked
    // here (`time.set` is a write).
    use_effect(move || {
        let f = format();
        time.set(format_time(&Date::new_0(), f));
    });

    use_effect(move || {
        spawn(async move {
            loop {
                TimeoutFuture::new(1000).await;
                let now = Date::new_0();
                time.set(format_time(&now, *format.peek()));
                date.set(format_date(&now));
            }
        });
    });

    (time, date)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twenty_four_hour_is_zero_padded() {
        assert_eq!(format_hm(0, 5, ClockFormat::H24), "00:05");
        assert_eq!(format_hm(9, 30, ClockFormat::H24), "09:30");
        assert_eq!(format_hm(19, 52, ClockFormat::H24), "19:52");
        assert_eq!(format_hm(23, 59, ClockFormat::H24), "23:59");
    }

    #[test]
    fn twelve_hour_handles_midnight_noon_and_afternoon() {
        assert_eq!(format_hm(0, 5, ClockFormat::H12), "12:05 AM");
        assert_eq!(format_hm(1, 0, ClockFormat::H12), "01:00 AM");
        assert_eq!(format_hm(11, 59, ClockFormat::H12), "11:59 AM");
        assert_eq!(format_hm(12, 0, ClockFormat::H12), "12:00 PM");
        assert_eq!(format_hm(13, 7, ClockFormat::H12), "01:07 PM");
        assert_eq!(format_hm(23, 59, ClockFormat::H12), "11:59 PM");
    }

    #[test]
    fn toggle_flips_and_defaults_to_24_hour() {
        assert_eq!(ClockFormat::default(), ClockFormat::H24);
        assert_eq!(ClockFormat::H24.toggled(), ClockFormat::H12);
        assert_eq!(ClockFormat::H12.toggled().toggled(), ClockFormat::H12);
        assert_ne!(
            ClockFormat::H24.switch_label(),
            ClockFormat::H12.switch_label()
        );
    }
}
