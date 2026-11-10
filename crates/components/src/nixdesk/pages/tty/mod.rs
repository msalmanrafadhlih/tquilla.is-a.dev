use dioxus::prelude::*;

mod chat_preview;
mod clock;
mod coming_soon;
mod nav_rail;
mod shell;

use crate::languages::use_strings;
use chat_preview::{use_chat_feed, ChatPreviewPanel};
use clock::{use_live_clock, ClockPanel};
use coming_soon::ComingSoonPanel;
use nav_rail::NavRail;
use shell::{ShellEntry, ShellHistoryPeek, ShellPanel};

/// Which panel is showing in the mobile "popup" slot above the tab row.
/// Named after the Figma layer each tab corresponds to.
#[derive(Clone, Copy, PartialEq)]
enum MobileTab {
    Shell2,
    Shell3,
    Shell4,
    Shell5,
}

/// Terminal TTY view. Mirrors the "Terminal TTY" Figma frame: a live
/// interactive shell plus a column of info panels (clock, auto-cycling
/// chat preview, two "coming soon" slots), collapsing into a tabbed
/// single-popup layout on narrow screens.
#[component]
pub fn TerminalMode(on_toggle: EventHandler<()>) -> Element {
    let s = use_strings();
    let mut loaded = use_signal(|| false);

    // Single source of truth for the shell — shared by both the desktop
    // and mobile layout blocks below so typing stays in sync instead of
    // running two independent terminals.
    let history: Signal<Vec<ShellEntry>> = use_signal(Vec::new);
    let cwd: Signal<String> = use_signal(|| "/home/tquilla".to_string());
    let command_log: Signal<Vec<String>> = use_signal(Vec::new);

    let (time, date) = use_live_clock();
    let chat_feed = use_chat_feed();

    let mut mobile_tab = use_signal(|| MobileTab::Shell3);

    rsx! {
        main {
            class: "terminal",
            class: if loaded() { "terminal__fade--in" } else { "terminal__fade--out" },
            onmounted: move |_| {
                spawn(async move {
                    document::eval(
                        "await new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));",
                    )
                        .await
                        .ok();
                    loaded.set(true);
                });
            },

            // ---------------- Desktop layout (lg and up) ----------------
            div { class: "terminal__desktop",
                div { class: "terminal__col-main",
                    ShellPanel {
                        history,
                        cwd,
                        command_log,
                        on_toggle,
                        compact: false,
                    }
                }
                div { class: "terminal__col-side",
                    div { class: "terminal__slot", ClockPanel { time, date } }
                    div { class: "terminal__slot", ChatPreviewPanel { feed: chat_feed } }
                    div { class: "terminal__slot", ComingSoonPanel { label: s.terminal.panel(5) } }
                    div { class: "terminal__slot", ComingSoonPanel { label: s.terminal.panel(6) } }
                }
                NavRail { on_toggle, vertical: true }
            }

            // ---------------- Mobile layout (below lg) ----------------
            div { class: "terminal__mobile",
                div { class: "terminal__mobile-main",
                    ShellPanel {
                        history,
                        cwd,
                        command_log,
                        on_toggle,
                        compact: true,
                    }
                }
                div { class: "terminal__mobile-top",
                    if mobile_tab() == MobileTab::Shell2 {
                        ShellHistoryPeek { history }
                    } else if mobile_tab() == MobileTab::Shell3 {
                        ClockPanel { time, date }
                    } else if mobile_tab() == MobileTab::Shell4 {
                        ChatPreviewPanel { feed: chat_feed }
                    } else {
                        ComingSoonPanel { label: s.terminal.panel(5) }
                    }
                }
                div { class: "terminal__tabs",
                    for (label , tab) in [
                        (s.terminal.panel(2), MobileTab::Shell2),
                        (s.terminal.panel(3), MobileTab::Shell3),
                        (s.terminal.panel(4), MobileTab::Shell4),
                        (s.terminal.panel(5), MobileTab::Shell5),
                    ]
                    {
                        button {
                            key: "{label}",
                            r#type: "button",
                            class: "terminal__tab",
                            class: if mobile_tab() == tab { "terminal__tab--on" } else { "terminal__tab--off" },
                            onclick: move |_| mobile_tab.set(tab),
                            "{label}"
                        }
                    }
                }
                NavRail { on_toggle, vertical: false }
            }
        }
    }
}
