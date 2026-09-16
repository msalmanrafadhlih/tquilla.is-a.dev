use dioxus::prelude::*;

mod chat_preview;
mod clock;
mod coming_soon;
mod nav_rail;
mod shell;

use chat_preview::{use_chat_cycle, ChatPreviewPanel};
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
    let mut loaded = use_signal(|| false);

    // Single source of truth for the shell — shared by both the desktop
    // and mobile layout blocks below so typing stays in sync instead of
    // running two independent terminals.
    let history: Signal<Vec<ShellEntry>> = use_signal(Vec::new);
    let cwd: Signal<String> = use_signal(|| "/home/tquilla".to_string());
    let command_log: Signal<Vec<String>> = use_signal(Vec::new);

    let (time, date) = use_live_clock();
    let chat_index = use_chat_cycle();

    let mut mobile_tab = use_signal(|| MobileTab::Shell3);

    rsx! {
        main {
            class: "relative h-screen w-full overflow-hidden bg-black text-white font-mono transition-opacity duration-500 ease-in",
            class: if loaded() { "opacity-100" } else { "opacity-0" },
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
            div { class: "hidden lg:flex h-full w-full gap-2.5 p-2.5",
                div { class: "flex-1 min-w-0 border border-white/15",
                    ShellPanel {
                        history,
                        cwd,
                        command_log,
                        on_toggle,
                        compact: false,
                    }
                }
                div { class: "w-[300px] xl:w-[340px] shrink-0 flex flex-col gap-2.5",
                    div { class: "flex-1 min-h-0", ClockPanel { time, date } }
                    div { class: "flex-1 min-h-0", ChatPreviewPanel { index: chat_index } }
                    div { class: "flex-1 min-h-0", ComingSoonPanel { label: "SHELL 5" } }
                    div { class: "flex-1 min-h-0", ComingSoonPanel { label: "SHELL 6" } }
                }
                NavRail { on_toggle, vertical: true }
            }

            // ---------------- Mobile layout (below lg) ----------------
            div { class: "flex lg:hidden gap-2 px-2 py-2 flex-col h-full w-full",
                div { class: "flex-1 min-h-0 border border-white/15",
                    ShellPanel {
                        history,
                        cwd,
                        command_log,
                        on_toggle,
                        compact: true,
                    }
                }
                div { class: "h-[150px] shrink-0 border-b border-white/15",
                    if mobile_tab() == MobileTab::Shell2 {
                        ShellHistoryPeek { history }
                    } else if mobile_tab() == MobileTab::Shell3 {
                        ClockPanel { time, date }
                    } else if mobile_tab() == MobileTab::Shell4 {
                        ChatPreviewPanel { index: chat_index }
                    } else {
                        ComingSoonPanel { label: "SHELL 5" }
                    }
                }
                div { class: "flex shrink-0 border border-white/15",
                    for (label , tab) in [
                        ("SHELL 2", MobileTab::Shell2),
                        ("SHELL 3", MobileTab::Shell3),
                        ("SHELL 4", MobileTab::Shell4),
                        ("SHELL 5", MobileTab::Shell5),
                    ]
                    {
                        button {
                            key: "{label}",
                            r#type: "button",
                            class: "flex-1 py-2.5 text-[10px] tracking-wide transition-colors duration-150",
                            class: if mobile_tab() == tab { "bg-white text-black" } else { "text-white/60 hover:text-white" },
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
