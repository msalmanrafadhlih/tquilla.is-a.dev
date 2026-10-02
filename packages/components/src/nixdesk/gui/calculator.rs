use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

/// All the calculator's state, bundled so each key can be wired with a
/// trivial inline closure (`move |_| input_digit(state, "7")`) instead of
/// sharing one mutable closure across many buttons.
#[derive(Clone, Copy)]
struct CalcState {
    display: Signal<String>,
    stored: Signal<Option<f64>>,
    pending_op: Signal<Option<Op>>,
    fresh_entry: Signal<bool>,
}

fn compute(a: f64, b: f64, op: Op) -> f64 {
    match op {
        Op::Add => a + b,
        Op::Sub => a - b,
        Op::Mul => a * b,
        Op::Div => {
            if b == 0.0 {
                0.0
            } else {
                a / b
            }
        }
    }
}

fn format_result(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{value:.0}")
    } else {
        let s = format!("{value:.6}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn input_digit(mut state: CalcState, d: &str) {
    let mut current = (state.display)();
    if (state.fresh_entry)() || current == "0" {
        current = if d == "." {
            "0.".to_string()
        } else {
            d.to_string()
        };
        state.fresh_entry.set(false);
    } else if d == "." && current.contains('.') {
        // ignore a second decimal point
    } else {
        current.push_str(d);
    }
    state.display.set(current);
}

fn apply_op(mut state: CalcState, op: Op) {
    let current: f64 = (state.display)().parse().unwrap_or(0.0);
    if let (Some(prev), Some(prev_op)) = ((state.stored)(), (state.pending_op)()) {
        let result = compute(prev, current, prev_op);
        state.display.set(format_result(result));
        state.stored.set(Some(result));
    } else {
        state.stored.set(Some(current));
    }
    state.pending_op.set(Some(op));
    state.fresh_entry.set(true);
}

fn equals(mut state: CalcState) {
    let current: f64 = (state.display)().parse().unwrap_or(0.0);
    if let (Some(prev), Some(op)) = ((state.stored)(), (state.pending_op)()) {
        state.display.set(format_result(compute(prev, current, op)));
    }
    state.stored.set(None);
    state.pending_op.set(None);
    state.fresh_entry.set(true);
}

fn clear_all(mut state: CalcState) {
    state.display.set("0".to_string());
    state.stored.set(None);
    state.pending_op.set(None);
    state.fresh_entry.set(true);
}

fn toggle_sign(mut state: CalcState) {
    let current: f64 = (state.display)().parse().unwrap_or(0.0);
    state.display.set(format_result(-current));
}

fn percent(mut state: CalcState) {
    let current: f64 = (state.display)().parse().unwrap_or(0.0);
    state.display.set(format_result(current / 100.0));
}

#[component]
pub fn Calculator() -> Element {
    let display = use_signal(|| "0".to_string());
    let stored = use_signal(|| Option::<f64>::None);
    let pending_op = use_signal(|| Option::<Op>::None);
    let fresh_entry = use_signal(|| true);
    let state = CalcState {
        display,
        stored,
        pending_op,
        fresh_entry,
    };

    rsx! {
        div { class: "flex flex-col h-fit bg-black gap-2 text-white font-mono",
            div { class: "flex-1 flex border border-white/15 items-end justify-end px-4 py-3 text-3xl truncate", "{display}" }
            div { class: "grid grid-cols-4 gap-2 bg-white/10 text-sm shrink-0",
                CalcKey { label: "C", onclick: move |_| clear_all(state) }
                CalcKey { label: "±", onclick: move |_| toggle_sign(state) }
                CalcKey { label: "%", onclick: move |_| percent(state) }
                CalcKey { label: "÷", onclick: move |_| apply_op(state, Op::Div) }

                CalcKey { label: "7", onclick: move |_| input_digit(state, "7") }
                CalcKey { label: "8", onclick: move |_| input_digit(state, "8") }
                CalcKey { label: "9", onclick: move |_| input_digit(state, "9") }
                CalcKey { label: "×", onclick: move |_| apply_op(state, Op::Mul) }

                CalcKey { label: "4", onclick: move |_| input_digit(state, "4") }
                CalcKey { label: "5", onclick: move |_| input_digit(state, "5") }
                CalcKey { label: "6", onclick: move |_| input_digit(state, "6") }
                CalcKey { label: "-", onclick: move |_| apply_op(state, Op::Sub) }

                CalcKey { label: "1", onclick: move |_| input_digit(state, "1") }
                CalcKey { label: "2", onclick: move |_| input_digit(state, "2") }
                CalcKey { label: "3", onclick: move |_| input_digit(state, "3") }
                CalcKey { label: "+", onclick: move |_| apply_op(state, Op::Add) }

                CalcKey { label: ".", onclick: move |_| input_digit(state, ".") }
                CalcKey { label: "0", onclick: move |_| input_digit(state, "0") }
                CalcKey { label: "000", onclick: move |_| input_digit(state, "000") }
                CalcKey { label: "=", onclick: move |_| equals(state) }
            }
        }
    }
}

#[component]
fn CalcKey(label: &'static str, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            r#type: "button",
            class: "bg-black hover:bg-white hover:text-black transition-colors duration-100 py-3.5",
            onclick: move |evt| onclick.call(evt),
            "{label}"
        }
    }
}
