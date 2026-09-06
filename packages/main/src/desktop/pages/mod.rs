use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;

mod interface;
mod booting;
mod login;
mod tty;

pub use login::Login;
pub use booting::Booting;
pub use tty::TerminalMode;
pub use interface::DesktopMode;

const CSS: Asset = asset!("/assets/booting.css");

/// Which interface `MainPage` currently shows. Flipped by the small "Mode"
/// badge (`#toggle`) that lives in the corner of both `TerminalMode` and
/// `DesktopMode`.
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Desktop,
    Terminal,
}

/// Crossfade duration in ms. Keep this in sync with the `duration-[…ms]`
/// class on the wrapper below — one drives the JS timing, the other the
/// CSS transition, and they need to agree or the swap will flash.
const SWITCH_MS: u32 = 250;

#[component]
pub fn MainPage() -> Element {
    let mut mode = use_signal(|| Mode::Desktop);
    // While `true`, the wrapper below is faded out — hides the instant
    // swap that happens the moment `mode` actually flips.
    let mut switching = use_signal(|| false);

    // Fade the current interface out, swap `mode` once it's invisible,
    // then fade the new one back in. Mirrors the double-rAF trick used
    // elsewhere so the opacity-0 frame is guaranteed to paint before the
    // transition starts.
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
        document::Link { rel: "stylesheet", href: CSS }

        div {
            class: "transition-opacity duration-[250ms] ease-out",
            style: if switching() { "opacity: 0; pointer-events: none;" } else { "opacity: 1;" },

            if mode() == Mode::Terminal {
                TerminalMode { on_toggle: move |_| toggle_mode() }
            } else {
                DesktopMode { on_toggle: move |_| toggle_mode() }
            }
        }
    }
}
