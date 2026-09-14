use dioxus::prelude::*;
use js_sys::Date;

use super::clock::{format_date, format_time};

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

const HELP_TEXT: &[&str] = &[
    "Available commands:",
    "  help            show this message",
    "  pwd             print working directory",
    "  whoami          print current user",
    "  ls              list directory contents",
    "  cd <dir>        change directory (.. or no arg goes home)",
    "  cat <file>      read a file (try: cat about.txt)",
    "  neofetch        system info",
    "  date            current date & time",
    "  echo <text>     print text",
    "  history         show command history",
    "  switch          jump to Desktop Mode",
    "  clear           clear the screen",
];

fn about_text() -> Vec<String> {
    vec![
        "NixOS 26.11 (Zakar)".to_string(),
        "-------------------".to_string(),
        "OS:         NixOS 26.11 (Zakar)".to_string(),
        "Kernel:     Linux 6.18.38".to_string(),
        "Arch:       x86_64".to_string(),
        "DE:         DeistifyNix (Wayland)".to_string(),
        "Built in:   Dioxus (Rust) + Tailwind".to_string(),
        "Design:     Black & White".to_string(),
        "Philosophy: Declaratif Desktop OS (Joke)".to_string(),
    ]
}

/// Runs one line of input against the fake shell. Returns the output
/// lines, an optional new working directory (for `cd`), and whether the
/// screen should be cleared.
fn run_command(
    raw: &str,
    cwd: &str,
    history: &[ShellEntry],
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
        "help" => (HELP_TEXT.iter().map(|s| s.to_string()).collect(), None, false),
        "pwd" => (vec![cwd.to_string()], None, false),
        "whoami" => (vec!["tquilla".to_string()], None, false),
        "ls" => {
            if in_subdir {
                (vec!["(empty)".to_string()], None, false)
            } else {
                (vec![HOME_DIRS.join("  ")], None, false)
            }
        }
        "cd" => match args.first().copied() {
            None | Some("~") | Some("..") => (vec![], Some(HOME.to_string()), false),
            Some(dir) if !in_subdir && HOME_DIRS.contains(&dir) => {
                (vec![], Some(format!("{HOME}/{dir}")), false)
            }
            Some(dir) => (vec![format!("cd: no such directory: {dir}")], None, false),
        },
        "cat" => match args.first().copied() {
            Some("about.txt") | Some("README") | Some("README.md") => (about_text(), None, false),
            Some(name) => (vec![format!("cat: {name}: No such file or directory")], None, false),
            None => (vec!["cat: missing file operand".to_string()], None, false),
        },
        "neofetch" | "about" => (about_text(), None, false),
        "date" => {
            let now = Date::new_0();
            (vec![format!("{}, {}", format_date(&now), format_time(&now))], None, false)
        }
        "echo" => (vec![args.join(" ")], None, false),
        "history" => {
            if history.is_empty() {
                (vec!["(no history yet)".to_string()], None, false)
            } else {
                let lines = history
                    .iter()
                    .enumerate()
                    .map(|(i, entry)| format!("{:>3}  {}", i + 1, entry.input))
                    .collect();
                (lines, None, false)
            }
        }
        "sudo" => (
            vec!["tquilla is not in the sudoers file. This incident will be reported.".to_string()],
            None,
            false,
        ),
        "switch" | "desktop" => {
            on_toggle.call(());
            (vec!["switching to Desktop Mode...".to_string()], None, false)
        }
        "clear" => (vec![], None, true),
        other => (vec![format!("command not found: {other}. Type 'help' for a list.")], None, false),
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
            return;
        }

        let mut log = command_log();
        log.push(raw.clone());
        command_log.set(log);
        log_cursor.set(None);

        let entries = history();
        let (output, new_cwd, should_clear) = run_command(&raw, &cwd(), &entries, on_toggle);

        if should_clear {
            history.set(vec![]);
        } else {
            let mut entries = history();
            let id = entries.len();
            entries.push(ShellEntry { id, input: raw.clone(), output });
            history.set(entries);
        }
        if let Some(dir) = new_cwd {
            cwd.set(dir);
        }
        draft.set(String::new());

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
                "Welcome to NixOS 26.11 (Zakar) — Dioxus terminal simulator. Type "
                span { class: "text-white/70", "help" }
                " to get started."
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
                    id: "shell-input",
                    class: "flex-1 bg-transparent outline-none border-none text-white font-mono caret-white",
                    autocomplete: "off",
                    spellcheck: "false",
                    value: "{draft}",
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
                            }
                        }
                        Key::ArrowDown => {
                            evt.prevent_default();
                            let log = command_log();
                            match log_cursor() {
                                Some(i) if i + 1 < log.len() => {
                                    log_cursor.set(Some(i + 1));
                                    draft.set(log[i + 1].clone());
                                }
                                _ => {
                                    log_cursor.set(None);
                                    draft.set(String::new());
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
    rsx! {
        div { class: "h-full overflow-y-auto px-4 py-3 text-[11px] leading-relaxed",
            if history().is_empty() {
                p { class: "text-white/30", "No commands run yet." }
            } else {
                for entry in history() {
                    div { key: "{entry.id}", class: "text-white/60 truncate", "$ {entry.input}" }
                }
            }
        }
    }
}
