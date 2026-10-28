use crate::languages::{use_strings, Strings};
use crate::nixdesk::js_util::set_field;
use crate::platform::Date;
use dioxus::prelude::*;

use super::clock::{format_date, format_time};
use crate::nixdesk::clock::{use_clock_format, ClockFormat};

const SHELL_INPUT_ID: &str = "shell-input";

/// One resolved command + the output lines it produced.
#[derive(Debug, Clone, PartialEq)]
pub struct ShellEntry {
    pub id: usize,
    pub input: String,
    pub output: Vec<String>,
}

/// The tiny fake filesystem under `/home/tquilla` — just enough for `ls`
/// and `cd` to feel real. Mirrors the folders shown in the File Manager
/// window on the Desktop side.
const HOME_DIRS: &[&str] = &["Documents", "Downloads", "Musics", "Pictures", "Videos"];
const HOME: &str = "/home/tquilla";

fn about_text(s: &Strings) -> Vec<String> {
    s.terminal
        .neofetch_lines()
        .iter()
        .map(|line| line.to_string())
        .collect()
}

/// Runs one line of input against the fake shell. Returns the output
/// lines, an optional new working directory (for `cd`), and whether the
/// screen should be cleared.
fn run_command(
    raw: &str,
    cwd: &str,
    history: &[ShellEntry],
    clock_format: ClockFormat,
    s: &Strings,
    on_toggle: EventHandler<()>,
) -> (Vec<String>, Option<String>, bool) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return (vec![], None, false);
    }

    let mut parts = trimmed.split_whitespace();
    let cmd = parts.next().unwrap_or("");
    let args: Vec<&str> = parts.collect();
    let in_subdir = cwd != HOME;

    match cmd {
        "help" => (
            s.terminal
                .help_lines()
                .iter()
                .map(|line| line.to_string())
                .collect(),
            None,
            false,
        ),
        "pwd" => (vec![cwd.to_string()], None, false),
        "whoami" => (vec!["tquilla".to_string()], None, false),
        "ls" => {
            if in_subdir {
                (vec![s.terminal.empty_dir.to_string()], None, false)
            } else {
                (vec![HOME_DIRS.join("  ")], None, false)
            }
        }
        "cd" => match args.first().copied() {
            None | Some("~") | Some("..") => (vec![], Some(HOME.to_string()), false),
            Some(dir) if !in_subdir && HOME_DIRS.contains(&dir) => {
                (vec![], Some(format!("{HOME}/{dir}")), false)
            }
            Some(dir) => (vec![s.terminal.cd_no_such_dir(dir)], None, false),
        },
        "cat" => match args.first().copied() {
            Some("about.txt") | Some("README") | Some("README.md") => (about_text(s), None, false),
            Some(name) => (vec![s.terminal.cat_no_such_file(name)], None, false),
            None => (
                vec![s.terminal.cat_missing_operand.to_string()],
                None,
                false,
            ),
        },
        "neofetch" | "about" => (about_text(s), None, false),
        "date" => {
            let now = Date::new_0();
            (
                vec![format!(
                    "{}, {}",
                    format_date(&now, s),
                    format_time(&now, clock_format, s)
                )],
                None,
                false,
            )
        }
        "echo" => (vec![args.join(" ")], None, false),
        "history" => {
            if history.is_empty() {
                (vec![s.terminal.no_history.to_string()], None, false)
            } else {
                let lines = history
                    .iter()
                    .enumerate()
                    .map(|(i, entry)| format!("{:>3}  {}", i + 1, entry.input))
                    .collect();
                (lines, None, false)
            }
        }
        "sudo" => (vec![s.terminal.sudo_denied.to_string()], None, false),
        "switch" | "desktop" => {
            on_toggle.call(());
            (vec![s.terminal.switching.to_string()], None, false)
        }
        "clear" => (vec![], None, true),
        other => (vec![s.terminal.command_not_found(other)], None, false),
    }
}

/// "shell 1" — the interactive terminal panel: scrolling history +
/// a live input line.
#[component]
pub fn ShellPanel(
    history: Signal<Vec<ShellEntry>>,
    cwd: Signal<String>,
    command_log: Signal<Vec<String>>,
    on_toggle: EventHandler<()>,
    compact: bool,
) -> Element {
    let s = use_strings();
    let clock_format = use_clock_format();
    let mut draft = use_signal(String::new);
    let mut log_cursor = use_signal(|| Option::<usize>::None);

    let container_class = if compact {
        "flex flex-col h-full overflow-y-auto px-4 py-3 gap-1.5 text-[12px] leading-relaxed"
    } else {
        "flex flex-col h-full overflow-y-auto px-5 py-4 gap-1.5 text-[13px] leading-relaxed"
    };

    let mut submit = move || {
        let raw = draft();
        if raw.trim().is_empty() {
            draft.set(String::new());
            set_field(SHELL_INPUT_ID, "", Some(&raw));
            return;
        }

        let mut log = command_log();
        log.push(raw.clone());
        command_log.set(log);
        log_cursor.set(None);

        let entries = history();
        let (output, new_cwd, should_clear) =
            run_command(&raw, &cwd(), &entries, clock_format(), s, on_toggle);

        if should_clear {
            history.set(vec![]);
        } else {
            let mut entries = history();
            let id = entries.len();
            entries.push(ShellEntry {
                id,
                input: raw.clone(),
                output,
            });
            history.set(entries);
        }
        if let Some(dir) = new_cwd {
            cwd.set(dir);
        }
        draft.set(String::new());
        // DOM pemilik teks (tanpa `value`), lihat `js_util::set_field`.
        set_field(SHELL_INPUT_ID, "", Some(&raw));

        spawn(async move {
            document::eval(
                "const el = document.getElementById('shell-scroll'); if (el) el.scrollTop = el.scrollHeight;",
            )
            .await
            .ok();
        });
    };

    rsx! {
        div {
            id: "shell-scroll",
            class: container_class,
            onclick: move |_| {
                spawn(async move {
                    document::eval("document.getElementById('shell-input')?.focus();").await.ok();
                });
            },

            p { class: "text-white/40 mb-1",
                "{s.terminal.welcome_before}"
                span { class: "text-white/70", "help" }
                "{s.terminal.welcome_after}"
            }

            for entry in history() {
                div { key: "{entry.id}", class: "flex flex-col gap-0.5",
                    div { class: "flex gap-2",
                        span { class: "text-white/40", "$" }
                        span { class: "text-white", "{entry.input}" }
                    }
                    for (i , line) in entry.output.iter().enumerate() {
                        div { key: "{entry.id}-{i}", class: "pl-4 text-white/60 whitespace-pre-wrap", "{line}" }
                    }
                }
            }

            div { class: "flex gap-2 items-center",
                span { class: "text-white/40", "$" }
                input {
                    id: SHELL_INPUT_ID,
                    class: "flex-1 bg-[var(--bg-main)] outline-none border-none text-white font-mono caret-white",
                    autocomplete: "off",
                    spellcheck: "false",
                    onmounted: move |evt| {
                        let data = evt.data();
                        spawn(async move {
                            let _ = data.set_focus(true).await;
                        });
                    },
                    oninput: move |evt: FormEvent| draft.set(evt.value()),
                    onkeydown: move |evt: KeyboardEvent| match evt.key() {
                        Key::Enter => {
                            evt.prevent_default();
                            submit();
                        }
                        Key::ArrowUp => {
                            evt.prevent_default();
                            let log = command_log();
                            if !log.is_empty() {
                                let next = match log_cursor() {
                                    None => log.len() - 1,
                                    Some(i) if i > 0 => i - 1,
                                    Some(i) => i,
                                };
                                log_cursor.set(Some(next));
                                draft.set(log[next].clone());
                                set_field(SHELL_INPUT_ID, &log[next], None);
                            }
                        }
                        Key::ArrowDown => {
                            evt.prevent_default();
                            let log = command_log();
                            match log_cursor() {
                                Some(i) if i + 1 < log.len() => {
                                    log_cursor.set(Some(i + 1));
                                    draft.set(log[i + 1].clone());
                                    set_field(SHELL_INPUT_ID, &log[i + 1], None);
                                }
                                _ => {
                                    log_cursor.set(None);
                                    draft.set(String::new());
                                    set_field(SHELL_INPUT_ID, "", None);
                                }
                            }
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}

/// Read-only peek at the shell history — used for the mobile "SHELL 2"
/// tab, which doesn't have room for a second full input line.
#[component]
pub fn ShellHistoryPeek(history: Signal<Vec<ShellEntry>>) -> Element {
    let s = use_strings();
    rsx! {
        div { class: "h-full overflow-y-auto px-4 py-3 text-[11px] leading-relaxed",
            if history().is_empty() {
                p { class: "text-white/30", "{s.terminal.no_commands}" }
            } else {
                for entry in history() {
                    div { key: "{entry.id}", class: "text-white/60 truncate", "$ {entry.input}" }
                }
            }
        }
    }
}
