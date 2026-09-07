use dioxus::prelude::*;

/// Format a float the way a pocket calculator would: round away tiny
/// floating point noise, then drop a trailing ".0" for whole numbers.
fn format_num(n: f64) -> String {
    if !n.is_finite() {
        return "Error".to_string();
    }
    let rounded = (n * 1e9).round() / 1e9;
    if rounded == rounded.trunc() && rounded.abs() < 1e15 {
        format!("{}", rounded.trunc() as i64)
    } else {
        format!("{rounded}")
    }
}

fn apply(a: f64, b: f64, op: char) -> f64 {
    match op {
        '+' => a + b,
        '-' => a - b,
        '×' => a * b,
        '÷' => {
            if b == 0.0 {
                f64::NAN
            } else {
                a / b
            }
        }
        _ => b,
    }
}

/// A small, fully working calculator — the one dock app that isn't just a
/// placeholder. Basic four-function arithmetic with chained operators
/// (e.g. `2 + 3 + 4 =` reads left to right, like a real pocket calculator).
#[component]
pub fn CalculatorApp() -> Element {
    let mut display = use_signal(|| "0".to_string());
    let mut stored = use_signal(|| Option::<f64>::None);
    let mut pending_op = use_signal(|| Option::<char>::None);
    let mut fresh = use_signal(|| true);

    let mut press_digit = move |d: &'static str| {
        if fresh() || display() == "0" {
            display.set(d.to_string());
            fresh.set(false);
        } else {
            display.with_mut(|v| v.push_str(d));
        }
    };

    let press_dot = move |_| {
        if fresh() {
            display.set("0.".to_string());
            fresh.set(false);
        } else if !display().contains('.') {
            display.with_mut(|v| v.push('.'));
        }
    };

    let mut press_op = move |op: char| {
        let current: f64 = display().parse().unwrap_or(0.0);
        if let (Some(a), Some(prev_op)) = (stored(), pending_op()) {
            if !fresh() {
                let result = apply(a, current, prev_op);
                stored.set(Some(result));
                display.set(format_num(result));
            }
        } else {
            stored.set(Some(current));
        }
        pending_op.set(Some(op));
        fresh.set(true);
    };

    let press_equals = move |_| {
        let current: f64 = display().parse().unwrap_or(0.0);
        if let (Some(a), Some(op)) = (stored(), pending_op()) {
            let result = apply(a, current, op);
            display.set(format_num(result));
        }
        stored.set(None);
        pending_op.set(None);
        fresh.set(true);
    };

    let press_clear = move |_| {
        display.set("0".to_string());
        stored.set(None);
        pending_op.set(None);
        fresh.set(true);
    };

    let press_sign = move |_| {
        let current: f64 = display().parse().unwrap_or(0.0);
        display.set(format_num(current * -1.0));
    };

    let press_percent = move |_| {
        let current: f64 = display().parse().unwrap_or(0.0);
        display.set(format_num(current / 100.0));
    };

    let key_base = "select-none rounded-md border border-white/25 py-3 text-sm transition-all duration-150 ease-out hover:border-white/70 hover:bg-white/10 active:scale-90 active:bg-white/20 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-white/70";

    let pending_hint = match (stored(), pending_op()) {
        (Some(a), Some(op)) => format!("{} {}", format_num(a), op),
        _ => String::new(),
    };
    let current_display = display();

    rsx! {
        div { class: "mx-auto flex w-full max-w-[280px] flex-col gap-3",
            div {
                class: "min-h-14 w-full overflow-hidden rounded-md border border-white/25 bg-white/5 px-3 py-2 text-right",
                p { class: "truncate text-3xl font-light tabular-nums", "{current_display}" }
                p { class: "h-4 truncate text-xs text-white/40", "{pending_hint}" }
            }

            div { class: "grid grid-cols-4 gap-2",
                button { r#type: "button", class: "{key_base} text-white/70", onclick: press_clear, "C" }
                button { r#type: "button", class: "{key_base} text-white/70", onclick: press_sign, "±" }
                button { r#type: "button", class: "{key_base} text-white/70", onclick: press_percent, "%" }
                button { r#type: "button", class: "{key_base} text-white/70", onclick: move |_| press_op('÷'), "÷" }

                button { r#type: "button", class: "{key_base}", onclick: move |_| press_digit("7"), "7" }
                button { r#type: "button", class: "{key_base}", onclick: move |_| press_digit("8"), "8" }
                button { r#type: "button", class: "{key_base}", onclick: move |_| press_digit("9"), "9" }
                button { r#type: "button", class: "{key_base} text-white/70", onclick: move |_| press_op('×'), "×" }

                button { r#type: "button", class: "{key_base}", onclick: move |_| press_digit("4"), "4" }
                button { r#type: "button", class: "{key_base}", onclick: move |_| press_digit("5"), "5" }
                button { r#type: "button", class: "{key_base}", onclick: move |_| press_digit("6"), "6" }
                button { r#type: "button", class: "{key_base} text-white/70", onclick: move |_| press_op('-'), "−" }

                button { r#type: "button", class: "{key_base}", onclick: move |_| press_digit("1"), "1" }
                button { r#type: "button", class: "{key_base}", onclick: move |_| press_digit("2"), "2" }
                button { r#type: "button", class: "{key_base}", onclick: move |_| press_digit("3"), "3" }
                button { r#type: "button", class: "{key_base} text-white/70", onclick: move |_| press_op('+'), "+" }

                button { r#type: "button", class: "{key_base} col-span-2", onclick: move |_| press_digit("0"), "0" }
                button { r#type: "button", class: "{key_base}", onclick: press_dot, "." }
                button {
                    r#type: "button",
                    class: "select-none rounded-md border border-white/70 bg-white/90 py-3 text-sm font-medium text-black transition-all duration-150 ease-out hover:bg-white active:scale-90",
                    onclick: press_equals,
                    "=",
                }
            }
        }
    }
}
