//! Definisi tombol dan komponen keypad (utama + tiga mode).
//!
//! Tombol ber-label MathML (`mathml: true`) meniru template □ milik Google. Karena input
//! berbentuk linear, tombol hanya menyisipkan teks; renderer menampilkannya sebagai
//! pangkat/pecahan/akar. Argumen fungsi dipisah dengan `;` (koma dipakai untuk desimal).

use dioxus::prelude::*;

use super::state::{Action, CalcState, Mode};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Key {
    pub label: &'static str,
    /// `true` bila `label` berisi MathML.
    pub mathml: bool,
    pub italic: bool,
    pub action: Action,
    /// Tooltip / petunjuk penggunaan.
    pub title: &'static str,
}

const fn txt(label: &'static str, insert: &'static str) -> Key {
    Key { label, mathml: false, italic: false, action: Action::Insert(insert), title: "" }
}

const fn var(label: &'static str) -> Key {
    Key { label, mathml: false, italic: true, action: Action::Insert(label), title: "" }
}

const fn ml(label: &'static str, insert: &'static str, title: &'static str) -> Key {
    Key { label, mathml: true, italic: false, action: Action::Insert(insert), title }
}

const fn act(label: &'static str, action: Action, title: &'static str) -> Key {
    Key { label, mathml: false, italic: false, action, title }
}

// ------------------------------------------------------------- keypad utama (4 kolom)

pub const MAIN: &[Key] = &[
    txt("(", "("),
    txt(")", ")"),
    act("⌫", Action::Backspace, "Hapus"),
    act("c", Action::Clear, "Bersihkan semua"),
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
    txt(",", ","),
    act("=", Action::Equals, "Hitung"),
    txt("+", "+"),
];

// ------------------------------------------------------------------- Aljabar (3 kolom)

pub const ALJABAR: &[Key] = &[
    ml("<math><msup><mi>□</mi><mi>□</mi></msup></math>", "^", "Pangkat: 2^3 (gunakan kurung untuk pangkat panjang)"),
    ml("<math><mroot><mi>□</mi><mi>□</mi></mroot></math>", "root(", "Akar ke-n: root(n; x)"),
    txt("<", "<"),
    ml("<math><mfrac><mi>□</mi><mi>□</mi></mfrac></math>", "frac(", "Pecahan: frac(atas; bawah)"),
    ml("<math><mo>|</mo><mi>□</mi><mo>|</mo></math>", "abs(", "Nilai mutlak"),
    txt("≤", "≤"),
    ml("<math><msub><mi>log</mi><mi>□</mi></msub></math>", "log(", "log(x) basis 10, atau log(basis; x)"),
    ml("<math><mi>□</mi><mo>!</mo></math>", "!", "Faktorial"),
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

// -------------------------------------------------------------- Trigonometri (3 kolom)

pub const TRIGONOMETRI: &[Key] = &[
    txt("sin", "sin("),
    txt("cos", "cos("),
    txt("tan", "tan("),
    txt("csc", "csc("),
    txt("sec", "sec("),
    txt("cot", "cot("),
    txt("arcsin", "arcsin("),
    txt("arccos", "arccos("),
    txt("arctan", "arctan("),
    ml("<math><msup><mi>□</mi><mn>2</mn></msup></math>", "^2", "Kuadrat"),
    ml("<math><msup><mi>□</mi><mo>°</mo></msup></math>", "°", "Derajat"),
    txt("π", "π"),
    var("x"),
    var("y"),
    txt("=", "="),
];

// ----------------------------------------------------------------- Kalkulus (3 kolom)

pub const KALKULUS: &[Key] = &[
    ml(
        "<math><mfrac><mi>d</mi><mrow><mi>d</mi><mi>□</mi></mrow></mfrac></math>",
        "diff(",
        "Turunan di satu titik: diff(f; x; a)",
    ),
    txt("∞", "∞"),
    ml("<math><mroot><mi>□</mi><mi>□</mi></mroot></math>", "root(", "Akar ke-n: root(n; x)"),
    ml(
        "<math><munder><mi>lim</mi><mrow><mi>□</mi><mo>→</mo><mi>□</mi></mrow></munder></math>",
        "lim(",
        "Limit dua sisi: lim(f; x; a)",
    ),
    ml(
        "<math><munder><mi>lim</mi><mrow><mi>□</mi><mo>→</mo><msup><mi>□</mi><mo>+</mo></msup></mrow></munder></math>",
        "limr(",
        "Limit dari kanan: limr(f; x; a)",
    ),
    ml(
        "<math><munder><mi>lim</mi><mrow><mi>□</mi><mo>→</mo><msup><mi>□</mi><mo>−</mo></msup></mrow></munder></math>",
        "liml(",
        "Limit dari kiri: liml(f; x; a)",
    ),
    ml("<math><msub><mi>log</mi><mi>□</mi></msub></math>", "log(", "log(x) basis 10, atau log(basis; x)"),
    ml(
        "<math><mi>C</mi><mo>(</mo><mi>n</mi><mo>,</mo><mi>k</mi><mo>)</mo></math>",
        "nCr(",
        "Kombinasi: nCr(n; k)",
    ),
    ml(
        "<math><mi>P</mi><mo>(</mo><mi>n</mi><mo>,</mo><mi>k</mi><mo>)</mo></math>",
        "nPr(",
        "Permutasi: nPr(n; k)",
    ),
    ml("<math><mo>∑</mo></math>", "sum(", "Jumlah: sum(f; k; dari; sampai)"),
    txt(";", ";"),
    ml(
        "<math><msubsup><mo>∫</mo><mi>□</mi><mi>□</mi></msubsup><mi>□</mi></math>",
        "int(",
        "Integral tentu: int(f; x; a; b)",
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
            class: "relative w-full h-max py-2 border-[0.5px] border-solid border-[var(--variable-collection-fg-secondary)] appearance-none bg-transparent text-inherit {radius} {tone}",
            class: "active:scale-95 transition-all duration-150 ease-in-out hover:opacity-60",
            onclick: move |evt| onclick.call(evt),
            if mathml {
                span {
                    class: "h-6 flex items-center justify-center text-lg leading-[normal]",
                    dangerous_inner_html: "{label}",
                }
            } else {
                span {
                    class: "h-6 flex items-center justify-center [font:'JetBrains_Mono-Regular',Helvetica] font-normal text-variable-collection-fg-main text-lg text-center tracking-[0] leading-[normal] {font}",
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
    rsx! {
        section {
            title: "Calculator Keypad",
            class: "w-full grid gap-2 bg-transparent text-xs self-stretch overflow-scroll {cols}",
            class: "[-webkit-touch-callout:none] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden",
            for (i , key) in keys.iter().enumerate() {
                CalcKey {
                    key: "{i}",
                    label: key.label,
                    mathml: key.mathml,
                    italic: key.italic,
                    title: key.title,
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
pub fn Aljabar() -> Element {
    rsx! {
        KeyPad { keys: ALJABAR, cols: "grid-cols-3", radius: "rounded-full", tone: "bg-white/10" }
    }
}

#[component]
pub fn Trigonometri() -> Element {
    rsx! {
        KeyPad { keys: TRIGONOMETRI, cols: "grid-cols-3", radius: "rounded-full", tone: "bg-white/10" }
    }
}

#[component]
pub fn Kalkulus() -> Element {
    rsx! {
        KeyPad { keys: KALKULUS, cols: "grid-cols-3", radius: "rounded-full", tone: "bg-white/10" }
    }
}

/// Tab pemilih mode (Aljabar / Trigonometri / Kalkulus).
#[component]
pub fn ModeTab(mode: Mode, label: &'static str) -> Element {
    let state = use_context::<CalcState>();
    let active = *state.mode.read() == mode;
    let class = if active {
        "w-full p-1 appearance-none text-xs text-[var(--variable-collection-bg-main)] bg-[var(--variable-collection-fg-main)]"
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
