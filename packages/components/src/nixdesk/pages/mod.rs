use dioxus::prelude::*;
use crate::platform::TimeoutFuture;

mod interface;
mod booting;
mod login;
mod tty;

pub use login::Login;
pub use booting::Booting;
pub use tty::TerminalMode;
pub use interface::DesktopMode;

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Desktop,
    Terminal,
}

/// Crossfade duration in ms. Keep this in sync with the `duration-[…ms]`
/// class on the wrapper below — one drives the JS timing, the other the
/// CSS transition, and they need to agree or the swap will flash.
const SWITCH_MS: u32 = 250;

/// Disables the browser context menu (right click) everywhere except inside
/// inputs/textareas and elements marked with `data-allow-contextmenu`.
/// Guarded by a window flag so remounting `MainPage` never stacks listeners.
const DISABLE_CONTEXT_MENU_JS: &str = r#"
    if (!window.__noCtxMenu) {
        window.__noCtxMenu = true;
        document.addEventListener('contextmenu', (e) => {
            if (e.target.closest('input, textarea, [data-allow-contextmenu]')) return;
            e.preventDefault();
        });
    }
"#;

#[component]
pub fn MainPage() -> Element {
    let mut mode = use_signal(|| Mode::Desktop);
    let mut switching = use_signal(|| false);

    use_effect(|| {
        document::eval(DISABLE_CONTEXT_MENU_JS);
    });

    let toggle_mode = move || {
        spawn(async move {
            switching.set(true);
            TimeoutFuture::new(SWITCH_MS).await;

            mode.set(if mode() == Mode::Desktop { Mode::Terminal } else { Mode::Desktop });

            document::eval(
                "await new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));"
            ).await.ok();
            switching.set(false);
        });
    };

    rsx! {
        div {
            class: "transition-opacity duration-[250ms] ease-out select-none [-webkit-touch-callout:none] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden",
            style: if switching() {
                "opacity: 0; pointer-events: none;"
            } else {
                "opacity: 1; pointer-events: auto;"
            },

            // Selalu ter-mount supaya window yang terbuka (dan prosesnya,
            // misal radio/embience) tidak mati saat pindah ke Terminal.
            // Cukup disembunyikan dengan display:none.
            div {
                style: if mode() == Mode::Desktop { "display: block;" } else { "display: none;" },
                DesktopMode { on_toggle: move |_| toggle_mode() }
            }

            if mode() == Mode::Terminal {
                TerminalMode { on_toggle: move |_| toggle_mode() }
            }
        }
    }
}
 
