//! Kalkulator multi-mode (Aljabar, Trigonometri, Kalkulus) bergaya kalkulator Google.
//!
//! Struktur:
//! - `engine/`  : tokenizer, parser, evaluator, kalkulus numerik (tanpa Dioxus, ada tes unit)
//! - `input.rs` : aturan penyuntingan ekspresi (desimal, operator ganda, backspace)
//! - `render.rs`: ekspresi → MathML untuk tampilan
//! - `state.rs` : `CalcState` (signal) dan aksi tombol
//! - `keypad.rs`: definisi tombol dan komponen keypad

use dioxus::prelude::*;

mod engine;
mod input;
mod keypad;
mod render;
mod state;

use engine::Angle;
use keypad::{Aljabar, Kalkulus, KeyPad, ModeTab, Trigonometri, MAIN};
use state::{CalcState, Mode};

use crate::shared::PLAYFAIR_FONT_URL;

#[component]
pub fn Calculator() -> Element {
    let expr = use_signal(String::new);
    let history = use_signal(String::new);
    let fresh = use_signal(|| false);
    let notice = use_signal(|| Option::<String>::None);
    let mode = use_signal(|| Mode::Aljabar);
    let angle = use_signal(|| Angle::Rad);
    let state = use_context_provider(|| CalcState {
        expr,
        history,
        fresh,
        notice,
        mode,
        angle,
    });

    // Baris utama: ekspresi yang sedang diketik, dirender sebagai MathML.
    let main_html = render::to_mathml(&state.expr.read());

    // Baris kedua: pesan (error / Benar / Salah), atau "ekspresi =" setelah menekan "=",
    // atau pratinjau hasil selagi mengetik.
    let secondary_html = if let Some(msg) = state.notice.read().clone() {
        format!("<span>{}</span>", render::html_escape(&msg))
    } else if *state.fresh.read() && !state.history.read().is_empty() {
        render::to_mathml(&format!("{}=", state.history.read()))
    } else {
        let e = state.expr.read().clone();
        if e.is_empty() || engine::is_plain_number(&e) {
            String::new()
        } else {
            match engine::evaluate(&e, *state.angle.read()) {
                Ok(outcome) => format!("<span>{}</span>", render::html_escape(&outcome.to_text())),
                Err(_) => String::new(), // ekspresi belum lengkap: jangan tampilkan error
            }
        }
    };

    let angle_label = match *state.angle.read() {
        Angle::Rad => "RAD",
        Angle::Deg => "DEG",
    };

    rsx! {
        // Tombol italik (`italic font-serif` di keypad.rs) memakai Playfair
        // Display; font ini tidak lagi dimuat global di `app.rs`.
        document::Stylesheet { href: PLAYFAIR_FONT_URL }
        section {
            class: "flex flex-col items-center h-full w-full gap-2.5 relative",
            title: "Calculator",
            header {
                class: "flex flex-col px-2.5 py-3 relative self-stretch w-full flex-[0_0_auto] rounded-[5px] border-[0.5px] border-solid border-[var(--fg-secondary)] bg-[image:var(--linear-lr)]",
                button {
                    r#type: "button",
                    class: "self-start appearance-none bg-[var(--bg-secondary)] text-xs opacity-50 hover:opacity-100",
                    title: "Ganti satuan sudut",
                    onclick: move |_| state.toggle_angle(),
                    "{angle_label}"
                }
                output {
                    class: "relative w-full min-h-8 text-2xl flex items-center justify-end overflow-x-auto [font:'JetBrains_Mono-Regular',Helvetica] font-normal text-fg-main text-center tracking-[0] leading-[normal]",
                    title: "output",
                    dangerous_inner_html: "{main_html}",
                }
                output {
                    class: "relative flex items-center justify-end w-full min-h-5 opacity-50 [font:'JetBrains_Mono-Regular',Helvetica] font-normal text-fg-main text-sm text-center tracking-[0] leading-[normal]",
                    title: "Runtime preview: result",
                    dangerous_inner_html: "{secondary_html}",
                }
            }
            KeyPad { keys: MAIN, cols: "grid-cols-4" }
            div { class: "flex flex-row w-full h-max py-2 gap-2 border-b border-solid border-[var(--fg-secondary)]",
                ModeTab { mode: Mode::Aljabar, label: "Aljabar" }
                ModeTab { mode: Mode::Trigonometri, label: "Trigonometri" }
                ModeTab { mode: Mode::Kalkulus, label: "Kalkulus" }
            }
            match *state.mode.read() {
                Mode::Aljabar => rsx! {
                    Aljabar {}
                },
                Mode::Trigonometri => rsx! {
                    Trigonometri {}
                },
                Mode::Kalkulus => rsx! {
                    Kalkulus {}
                },
            }
        }
    }
}
