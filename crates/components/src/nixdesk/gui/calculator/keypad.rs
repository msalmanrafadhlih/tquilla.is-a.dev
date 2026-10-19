//! Definisi tombol dan komponen keypad (utama + tiga mode).
//!
//! Tombol ber-label MathML (`mathml: true`) meniru template □ milik Google. Karena input
//! berbentuk linear, tombol hanya menyisipkan teks; renderer menampilkannya sebagai
//! pangkat/pecahan/akar. Argumen fungsi dipisah dengan `;`.

use dioxus::prelude::*;

use super::state::{Action, CalcState, Mode};
use crate::languages::{use_strings, Strings};

/// Petunjuk (tooltip) sebuah tombol. Enum, bukan teks, supaya tabel tombol
/// tetap `const` dan teksnya mengikuti bahasa aktif (`Tip::text`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tip {
    None,
    Delete,
    ClearAll,
    Equals,
    Power,
    NthRoot,
    Fraction,
    Abs,
    Log,
    Factorial,
    Square,
    Degree,
    Diff,
    Lim,
    LimR,
    LimL,
    Ncr,
    Npr,
    Sum,
    Int,
}

impl Tip {
    pub fn text(self, s: &Strings) -> &'static str {
        let c = &s.calc;
        match self {
            Tip::None => "",
            Tip::Delete => c.tip_delete,
            Tip::ClearAll => c.tip_clear_all,
            Tip::Equals => c.tip_equals,
            Tip::Power => c.tip_power,
            Tip::NthRoot => c.tip_nth_root,
            Tip::Fraction => c.tip_fraction,
            Tip::Abs => c.tip_abs,
            Tip::Log => c.tip_log,
            Tip::Factorial => c.tip_factorial,
            Tip::Square => c.tip_square,
            Tip::Degree => c.tip_degree,
            Tip::Diff => c.tip_diff,
            Tip::Lim => c.tip_lim,
            Tip::LimR => c.tip_limr,
            Tip::LimL => c.tip_liml,
            Tip::Ncr => c.tip_ncr,
            Tip::Npr => c.tip_npr,
            Tip::Sum => c.tip_sum,
            Tip::Int => c.tip_int,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Key {
    pub label: &'static str,
    /// `true` bila `label` berisi MathML.
    pub mathml: bool,
    pub italic: bool,
    pub action: Action,
    /// Tooltip / petunjuk penggunaan.
    pub tip: Tip,
}

const fn txt(label: &'static str, insert: &'static str) -> Key {
    Key {
        label,
        mathml: false,
        italic: false,
        action: Action::Insert(insert),
        tip: Tip::None,
    }
}

const fn var(label: &'static str) -> Key {
    Key {
        label,
        mathml: false,
        italic: true,
        action: Action::Insert(label),
        tip: Tip::None,
    }
}

const fn ml(label: &'static str, insert: &'static str, tip: Tip) -> Key {
    Key {
        label,
        mathml: true,
        italic: false,
        action: Action::Insert(insert),
        tip,
    }
}

const fn act(label: &'static str, action: Action, tip: Tip) -> Key {
    Key {
        label,
        mathml: false,
        italic: false,
        action,
        tip,
    }
}

// ------------------------------------------------------------- keypad utama (4 kolom)

pub const MAIN: &[Key] = &[
    txt("(", "("),
    txt(")", ")"),
    act("⌫", Action::Backspace, Tip::Delete),
    act("c", Action::Clear, Tip::ClearAll),
    txt("7", "7"),
    txt("8", "8"),
    txt("9", "9"),
    txt("÷", "÷"),
    txt("4", "4"),
    txt("5", "5"),
    txt("6", "6"),
    txt("×", "×"),
    txt("1", "1"),
    txt("2", "2"),
    txt("3", "3"),
    txt("-", "-"),
    txt("0", "0"),
    txt(".", "."),
    act("=", Action::Equals, Tip::Equals),
    txt("+", "+"),
];

// ------------------------------------------------------------------- Algebra (3 kolom)

pub const ALGEBRA: &[Key] = &[
    ml(
        "<math><msup><mi>□</mi><mi>□</mi></msup></math>",
        "^",
        Tip::Power,
    ),
    ml(
        "<math><mroot><mi>□</mi><mi>□</mi></mroot></math>",
        "root(",
        Tip::NthRoot,
    ),
    txt("<", "<"),
    ml(
        "<math><mfrac><mi>□</mi><mi>□</mi></mfrac></math>",
        "frac(",
        Tip::Fraction,
    ),
    ml(
        "<math><mo>|</mo><mi>□</mi><mo>|</mo></math>",
        "abs(",
        Tip::Abs,
    ),
    txt("≤", "≤"),
    ml(
        "<math><msub><mi>log</mi><mi>□</mi></msub></math>",
        "log(",
        Tip::Log,
    ),
    ml("<math><mi>□</mi><mo>!</mo></math>", "!", Tip::Factorial),
    txt(">", ">"),
    var("i"),
    txt("%", "%"),
    txt("≥", "≥"),
    var("x"),
    var("y"),
    txt("=", "="),
    txt(";", ";"),
    txt("√", "√("),
    txt("ln", "ln("),
];

// -------------------------------------------------------------- Trigonometry (3 kolom)

pub const TRIGONOMETRY: &[Key] = &[
    txt("sin", "sin("),
    txt("cos", "cos("),
    txt("tan", "tan("),
    txt("csc", "csc("),
    txt("sec", "sec("),
    txt("cot", "cot("),
    txt("arcsin", "arcsin("),
    txt("arccos", "arccos("),
    txt("arctan", "arctan("),
    ml(
        "<math><msup><mi>□</mi><mn>2</mn></msup></math>",
        "^2",
        Tip::Square,
    ),
    ml(
        "<math><msup><mi>□</mi><mo>°</mo></msup></math>",
        "°",
        Tip::Degree,
    ),
    txt("π", "π"),
    var("x"),
    var("y"),
    txt("=", "="),
];

// ----------------------------------------------------------------- Calculus (3 kolom)

pub const CALCULUS: &[Key] = &[
    ml(
        "<math><mfrac><mi>d</mi><mrow><mi>d</mi><mi>□</mi></mrow></mfrac></math>",
        "diff(",
        Tip::Diff,
    ),
    txt("∞", "∞"),
    ml("<math><mroot><mi>□</mi><mi>□</mi></mroot></math>", "root(", Tip::NthRoot),
    ml(
        "<math><munder><mi>lim</mi><mrow><mi>□</mi><mo>→</mo><mi>□</mi></mrow></munder></math>",
        "lim(",
        Tip::Lim,
    ),
    ml(
        "<math><munder><mi>lim</mi><mrow><mi>□</mi><mo>→</mo><msup><mi>□</mi><mo>+</mo></msup></mrow></munder></math>",
        "limr(",
        Tip::LimR,
    ),
    ml(
        "<math><munder><mi>lim</mi><mrow><mi>□</mi><mo>→</mo><msup><mi>□</mi><mo>−</mo></msup></mrow></munder></math>",
        "liml(",
        Tip::LimL,
    ),
    ml("<math><msub><mi>log</mi><mi>□</mi></msub></math>", "log(", Tip::Log),
    ml(
        "<math><mi>C</mi><mo>(</mo><mi>n</mi><mo>,</mo><mi>k</mi><mo>)</mo></math>",
        "nCr(",
        Tip::Ncr,
    ),
    ml(
        "<math><mi>P</mi><mo>(</mo><mi>n</mi><mo>,</mo><mi>k</mi><mo>)</mo></math>",
        "nPr(",
        Tip::Npr,
    ),
    ml("<math><mo>∑</mo></math>", "sum(", Tip::Sum),
    txt(";", ";"),
    ml(
        "<math><msubsup><mo>∫</mo><mi>□</mi><mi>□</mi></msubsup><mi>□</mi></math>",
        "int(",
        Tip::Int,
    ),
    var("x"),
    var("y"),
    var("e"),
];

// ---------------------------------------------------------------------- komponen

#[component]
pub fn CalcKey(
    label: &'static str,
    #[props(default)] mathml: bool,
    #[props(default)] italic: bool,
    #[props(default)] title: &'static str,
    #[props(default = "rounded-[5px]")] radius: &'static str,
    #[props(default)] tone: &'static str,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let font = if italic { "italic font-serif" } else { "" };
    rsx! {
        button {
            r#type: "button",
            title: "{title}",
            class: "relative w-full h-max py-2 border-[0.5px] border-solid border-[var(--fg-secondary)] appearance-none bg-[var(--bg-secondary)] text-inherit {radius} {tone}",
            class: "active:scale-95 transition-all duration-150 ease-in-out hover:opacity-60",
            onclick: move |evt| onclick.call(evt),
            if mathml {
                span {
                    class: "h-6 flex items-center justify-center text-lg leading-[normal]",
                    dangerous_inner_html: "{label}",
                }
            } else {
                span {
                    class: "h-6 flex items-center justify-center [font:'JetBrains_Mono-Regular',Helvetica] font-normal text-fg-main text-lg text-center tracking-[0] leading-[normal] {font}",
                    "{label}"
                }
            }
        }
    }
}

/// Grid tombol generik; membaca `CalcState` dari context.
#[component]
pub fn KeyPad(
    keys: &'static [Key],
    cols: &'static str,
    #[props(default = "rounded-[5px]")] radius: &'static str,
    #[props(default)] tone: &'static str,
) -> Element {
    let state = use_context::<CalcState>();
    let s = use_strings();
    rsx! {
        section {
            title: s.calc.keypad_title,
            class: "w-full grid gap-2 bg-[var(--bg-secondary)] text-xs self-stretch overflow-scroll {cols}",
            class: "[-webkit-touch-callout:none] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden",
            for (i , key) in keys.iter().enumerate() {
                CalcKey {
                    key: "{i}",
                    label: key.label,
                    mathml: key.mathml,
                    italic: key.italic,
                    title: key.tip.text(s),
                    radius,
                    tone,
                    onclick: {
                        let action = key.action;
                        move |_| state.press(action)
                    },
                }
            }
        }
    }
}

#[component]
pub fn Algebra() -> Element {
    rsx! {
        KeyPad { keys: ALGEBRA, cols: "grid-cols-3", radius: "rounded-full", tone: "bg-white/10" }
    }
}

#[component]
pub fn Trigonometry() -> Element {
    rsx! {
        KeyPad { keys: TRIGONOMETRY, cols: "grid-cols-3", radius: "rounded-full", tone: "bg-white/10" }
    }
}

#[component]
pub fn Calculus() -> Element {
    rsx! {
        KeyPad { keys: CALCULUS, cols: "grid-cols-3", radius: "rounded-full", tone: "bg-white/10" }
    }
}

/// Tab pemilih mode (Algebra / Trigonometry / Calculus).
#[component]
pub fn ModeTab(mode: Mode, label: &'static str) -> Element {
    let state = use_context::<CalcState>();
    let active = *state.mode.read() == mode;
    let class = if active {
        "w-full p-1 appearance-none text-xs text-[var(--bg-secondary)] bg-[var(--fg-main)]"
    } else {
        "w-full p-1 appearance-none text-xs hover:opacity-100 opacity-50"
    };
    rsx! {
        button {
            r#type: "button",
            class: "{class}",
            onclick: move |_| {
                let mut m = state.mode;
                m.set(mode);
            },
            "{label}"
        }
    }
}
