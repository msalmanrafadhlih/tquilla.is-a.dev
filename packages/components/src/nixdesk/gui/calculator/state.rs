//! State kalkulator yang dibagikan ke semua keypad lewat context.

use dioxus::prelude::*;

use super::engine::{self, Angle, Outcome};
use super::input;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Aljabar,
    Trigonometri,
    Kalkulus,
}

/// Apa yang dilakukan sebuah tombol.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Action {
    Insert(&'static str),
    Backspace,
    Clear,
    Equals,
}

#[derive(Clone, Copy)]
pub struct CalcState {
    /// Ekspresi yang sedang diketik (string linear, lihat engine/mod.rs).
    pub expr: Signal<String>,
    /// Ekspresi sebelum "=" ditekan (ditampilkan kecil di bawah hasil).
    pub history: Signal<String>,
    /// `true` bila `expr` adalah hasil dari "=" (ketikan berikut memulai ekspresi baru).
    pub fresh: Signal<bool>,
    /// Pesan error atau hasil Benar/Salah setelah "=".
    pub notice: Signal<Option<String>>,
    pub mode: Signal<Mode>,
    pub angle: Signal<Angle>,
}

impl CalcState {
    fn reset_feedback(mut self) {
        self.notice.set(None);
        self.history.set(String::new());
    }

    pub fn press(mut self, action: Action) {
        match action {
            Action::Insert(text) => {
                let mut e = self.expr.peek().clone();
                input::insert(&mut e, *self.fresh.peek(), text);
                self.expr.set(e);
                self.fresh.set(false);
                self.reset_feedback();
            }
            Action::Backspace => {
                let mut e = self.expr.peek().clone();
                input::backspace(&mut e);
                self.expr.set(e);
                self.fresh.set(false);
                self.reset_feedback();
            }
            Action::Clear => {
                self.expr.set(String::new());
                self.fresh.set(false);
                self.reset_feedback();
            }
            Action::Equals => self.equals(),
        }
    }

    fn equals(mut self) {
        let e = self.expr.peek().clone();
        if e.is_empty() || engine::is_plain_number(&e) {
            return;
        }
        match engine::evaluate(&e, *self.angle.peek()) {
            Ok(Outcome::Number(v)) => {
                self.history.set(e);
                self.expr.set(engine::format_number(v));
                self.fresh.set(true);
                self.notice.set(None);
            }
            Ok(Outcome::Bool(b)) => {
                self.history.set(String::new());
                self.notice.set(Some(Outcome::Bool(b).to_text()));
            }
            Err(err) => {
                self.history.set(String::new());
                self.notice.set(Some(err.to_string()));
            }
        }
    }

    pub fn toggle_angle(mut self) {
        let next = match *self.angle.peek() {
            Angle::Rad => Angle::Deg,
            Angle::Deg => Angle::Rad,
        };
        self.angle.set(next);
        self.notice.set(None);
    }
}
