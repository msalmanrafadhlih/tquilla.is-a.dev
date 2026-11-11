//! Panel samping yang bisa dibuka-tutup di dalam sebuah jendela: dipakai oleh
//! riwayat chat (AI Assistant, kiri) dan "Sign the guestbook" (Guestbook, kanan).
//!
//! Panel **selalu dirender** dan hanya mengganti class `side-panel--open`.
//! Dengan begitu CSS bisa menganimasikan buka DAN tutup (geser + scrim memudar);
//! kalau panel dibuat/dibuang lewat `if open { .. }`, tutupnya tidak punya apa-apa
//! untuk dianimasikan. Saat tertutup panel `visibility: hidden` (lihat
//! `styles/global.css`), jadi isinya tidak bisa difokus dan tidak dibaca pembaca layar.
//!
//! Induknya harus `position: relative; overflow: hidden` (mis. `.ai`,
//! `.guestbook`): panel menempel di jendela, bukan di viewport.

use dioxus::prelude::*;

/// Sisi jendela tempat panel menempel dan dari mana ia bergeser masuk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

/// Class panel: dasar + sisi + (bila terbuka) modifier `--open` + class tambahan
/// milik UI pemakainya. `--open` ditulis setelah sisi di CSS supaya menang.
fn panel_class(side: Side, open: bool, extra: &str) -> String {
    let side = match side {
        Side::Left => "side-panel--left",
        Side::Right => "side-panel--right",
    };
    let state = if open { " side-panel--open" } else { "" };
    format!("side-panel {side}{state} {extra}")
        .trim_end()
        .to_string()
}

fn scrim_class(open: bool) -> &'static str {
    if open {
        "side-panel__scrim side-panel__scrim--open"
    } else {
        "side-panel__scrim"
    }
}

#[component]
pub fn SidePanel(
    /// Dipakai tombol pembukanya untuk `aria-controls`.
    id: &'static str,
    /// Nama panel untuk pembaca layar.
    label: &'static str,
    side: Side,
    open: bool,
    /// Dipanggil saat scrim diklik atau Esc ditekan di dalam panel.
    on_close: EventHandler<()>,
    /// Class UI pemakai (padding, lebar lewat `--side-panel-w`, ...).
    #[props(default = "")]
    class: &'static str,
    children: Element,
) -> Element {
    rsx! {
        // Klik di luar panel menutupnya. Scrim hanya dekorasi bagi pembaca layar.
        div {
            class: scrim_class(open),
            "aria-hidden": "true",
            onclick: move |_| on_close.call(()),
        }
        aside {
            id,
            "aria-label": label,
            class: panel_class(side, open, class),
            onkeydown: move |evt: KeyboardEvent| {
                if open && evt.key() == Key::Escape {
                    evt.stop_propagation();
                    on_close.call(());
                }
            },
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_panel_has_no_open_modifier() {
        assert_eq!(
            panel_class(Side::Left, false, "ai__history"),
            "side-panel side-panel--left ai__history"
        );
        assert_eq!(scrim_class(false), "side-panel__scrim");
    }

    #[test]
    fn open_panel_adds_the_open_modifier_after_the_side() {
        assert_eq!(
            panel_class(Side::Right, true, "guestbook__panel"),
            "side-panel side-panel--right side-panel--open guestbook__panel"
        );
        assert_eq!(
            scrim_class(true),
            "side-panel__scrim side-panel__scrim--open"
        );
    }

    #[test]
    fn empty_extra_class_leaves_no_trailing_space() {
        assert_eq!(
            panel_class(Side::Left, false, ""),
            "side-panel side-panel--left"
        );
    }
}
