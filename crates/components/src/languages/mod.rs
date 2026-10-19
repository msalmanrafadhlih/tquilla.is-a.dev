//! Bahasa antarmuka.
//!
//! - [`Language`]: bahasa yang tersedia (sekarang hanya Inggris).
//! - [`Strings`]: semua teks UI untuk satu bahasa (`language.rs`).
//! - Pilihan bahasa disimpan di context `Signal<Language>` (pola yang sama
//!   dengan `ClockFormat`): provider di `App`, lalu komponen mana pun cukup
//!   memanggil [`use_strings`]. Settings nanti tinggal menulis ke signal yang
//!   sama lewat [`use_language`]; semua teks ikut berganti.
//!
//! Di luar komponen (handler async, fungsi murni) ambil `&'static Strings`
//! sekali saat render lalu teruskan; jangan membaca signal dari dalam `spawn`.

mod language;

#[cfg(test)]
mod tests;

use dioxus::prelude::*;

pub use language::*;

/// Bahasa UI. Tambah varian baru di sini, lalu isi `Language::strings`,
/// `code` dan `native_name`; compiler menunjukkan semua `match` yang perlu diperbarui.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Language {
    #[default]
    En,
}

impl Language {
    /// Semua bahasa, urutan tampil di pemilih bahasa nanti.
    pub const ALL: &'static [Language] = &[Language::En];

    /// Kode BCP 47 (untuk `<html lang>` dan penyimpanan preferensi nanti).
    pub fn code(self) -> &'static str {
        match self {
            Language::En => "en",
        }
    }

    /// Nama bahasa dalam bahasa itu sendiri (untuk pemilih bahasa).
    pub fn native_name(self) -> &'static str {
        match self {
            Language::En => "English",
        }
    }

    pub fn strings(self) -> &'static Strings {
        match self {
            Language::En => &EN,
        }
    }
}

/// Panggil sekali, tinggi di pohon komponen (`App`).
pub fn use_language_provider() -> Signal<Language> {
    use_context_provider(|| Signal::new(Language::default()))
}

/// Baca/tulis bahasa aktif dari komponen turunan `App`.
pub fn use_language() -> Signal<Language> {
    use_context::<Signal<Language>>()
}

/// Teks UI untuk bahasa aktif. Membaca signal, jadi komponen yang
/// memanggilnya dirender ulang saat bahasa berganti.
pub fn use_strings() -> &'static Strings {
    let language = use_language();
    let current = language();
    current.strings()
}
