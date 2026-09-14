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

/// "07:52 PM" — same style as the navbar clock on the Desktop side.
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
/// clock. Called once from `TerminalMode` so every panel that shows the
/// clock (desktop column + mobile tab) reads the same live signals
/// instead of each starting its own timer.
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

/// "shell 3" — the digital clock panel.
#[component]
pub fn ClockPanel(time: Signal<String>, date: Signal<String>) -> Element {
    rsx! {
        div { class: "flex flex-col items-center justify-center h-full gap-2 border border-white/15 px-6 py-8 text-center",
            span { class: "text-3xl sm:text-4xl tracking-widest font-semibold", "{time}" }
            span { class: "text-white/50 text-xs sm:text-sm tracking-wide", "{date}" }
        }
    }
}
