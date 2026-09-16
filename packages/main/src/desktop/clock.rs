use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use js_sys::Date;

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

/// "07:52 PM"
pub fn format_time(date: &Date) -> String {
    let hours24 = date.get_hours();
    let minutes = date.get_minutes();
    let (hours12, suffix) = match hours24 {
        0 => (12, "AM"),
        1..=11 => (hours24, "AM"),
        12 => (12, "PM"),
        _ => (hours24 - 12, "PM"),
    };
    format!("{hours12:02}:{minutes:02} {suffix}")
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
    let mut time = use_signal(|| format_time(&Date::new_0()));
    let mut date = use_signal(|| format_date(&Date::new_0()));

    use_effect(move || {
        spawn(async move {
            loop {
                TimeoutFuture::new(1000).await;
                let now = Date::new_0();
                time.set(format_time(&now));
                date.set(format_date(&now));
            }
        });
    });

    (time, date)
}
