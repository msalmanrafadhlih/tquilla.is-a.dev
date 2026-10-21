//! Penjaga teks UI:
//! 1. semua kunci terisi di setiap bahasa dan templatenya valid;
//! 2. tidak ada teks tampilan yang tertinggal langsung di kode komponen.

use super::*;

/// Semua nama placeholder yang dikenal `fill`. Menambah placeholder baru
/// berarti menambahkannya juga di sini.
const KNOWN_PLACEHOLDERS: &[&str] = &[
    "n", "number", "build", "kernel", "date", "seconds", "name", "item", "dir", "cmd", "slogan",
    "user", "mb", "status", "error", "growth", "count", "year", "c", "lo", "hi", "ago", "month",
    "activity", "language", "repo", "ideas",
];

fn placeholders(text: &str) -> Result<Vec<&str>, String> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find(['{', '}']) {
        if rest.as_bytes()[open] == b'}' {
            return Err(format!("unmatched '}}' in {text:?}"));
        }
        let after = &rest[open + 1..];
        let Some(close) = after.find(['{', '}']) else {
            return Err(format!("unclosed '{{' in {text:?}"));
        };
        if after.as_bytes()[close] == b'{' {
            return Err(format!("nested '{{' in {text:?}"));
        }
        found.push(&after[..close]);
        rest = &after[close + 1..];
    }
    Ok(found)
}

#[test]
fn every_key_is_filled_in_every_language() {
    for lang in Language::ALL {
        let entries = lang.strings().entries();
        assert!(
            entries.len() > 300,
            "suspiciously few entries: {}",
            entries.len()
        );
        for (path, text) in entries {
            assert!(!text.trim().is_empty(), "{}: {path} is empty", lang.code());
        }
    }
}

#[test]
fn keys_are_unique() {
    for lang in Language::ALL {
        let mut paths: Vec<String> = lang.strings().entries().into_iter().map(|e| e.0).collect();
        let before = paths.len();
        paths.sort();
        paths.dedup();
        assert_eq!(before, paths.len(), "{}: duplicate key", lang.code());
    }
}

#[test]
fn templates_use_balanced_known_placeholders() {
    for lang in Language::ALL {
        for (path, text) in lang.strings().entries() {
            let names = placeholders(text).unwrap_or_else(|e| panic!("{path}: {e}"));
            for name in names {
                assert!(
                    KNOWN_PLACEHOLDERS.contains(&name),
                    "{}: {path} uses unknown placeholder {{{name}}}",
                    lang.code()
                );
            }
        }
    }
}

#[test]
fn language_codes_and_names_are_unique_and_non_empty() {
    let mut codes: Vec<&str> = Language::ALL.iter().map(|l| l.code()).collect();
    assert!(codes.iter().all(|c| !c.is_empty()));
    assert!(Language::ALL
        .iter()
        .all(|l| !l.native_name().trim().is_empty()));
    let before = codes.len();
    codes.sort();
    codes.dedup();
    assert_eq!(before, codes.len());
    assert!(Language::ALL.contains(&Language::default()));
}

#[test]
fn fill_is_single_pass_and_keeps_unknown_placeholders() {
    assert_eq!(fill("a {x} b", &[("x", &"{y}")]), "a {y} b");
    assert_eq!(fill("a {zzz} b", &[("x", &1)]), "a {zzz} b");
    assert_eq!(fill("no placeholder", &[]), "no placeholder");
    assert_eq!(fill("open {", &[]), "open {");
}

/// Setiap method template dipanggil dengan argumen contoh; hasilnya tidak
/// boleh menyisakan `{` atau `}` (artinya ada placeholder yang tidak terisi).
#[test]
fn template_methods_leave_no_placeholder() {
    for lang in Language::ALL {
        let s = lang.strings();
        let mut out: Vec<String> = Vec::new();

        for unit in [
            TimeUnit::Minute,
            TimeUnit::Hour,
            TimeUnit::Day,
            TimeUnit::Week,
            TimeUnit::Month,
            TimeUnit::Year,
        ] {
            for n in [1, 2, 30] {
                out.push(s.time.ago(n, unit));
            }
        }
        out.push(s.home.generation_line(7, &"abc", "6.1", "Sep 14"));
        out.push(s.home.auto_boot_countdown(5));
        out.push(s.login.welcome_back("Moch"));
        out.push(s.navbar.menu_hint("File"));
        out.push(s.terminal.cd_no_such_dir("x"));
        out.push(s.terminal.cat_no_such_file("x"));
        out.push(s.terminal.command_not_found("x"));
        out.push(s.radio.slogan_line("x"));
        out.push(s.embience.select_ambience("Rain"));
        out.push(s.embience.volume_of("Rain"));
        out.push(s.guestbook.avatar_alt("Moch"));
        out.push(s.guestbook.chars_left(12));
        out.push(s.guestbook.chars_over(3));
        out.push(s.guestbook.image_link("cat"));
        out.push(s.ai.http_failed(500));
        out.push(s.journal.build_number(42));
        out.push(s.journal.load_failed("boom"));
        out.push(s.journal.growth_vs_last_year("+3%"));
        out.push(s.journal.streak_days(9));
        out.push(s.journal.updated("3 hours ago"));
        for lang in [None, Some("Rust"), Some("C++"), Some("Brainfuck")] {
            out.push(s.journal.activity_for(lang).to_string());
        }
        out.push(s.journal.chart_commits(12));
        out.push(s.journal.copyright(2026));
        out.push(s.calc_err.unexpected_char('?'));
        out.push(s.calc_err.needs_parens("sin"));
        out.push(s.calc_err.arg_count("nCr", 1, 1));
        out.push(s.calc_err.arg_count("nCr", 2, 2));
        out.push(s.calc_err.arg_count("log", 1, 2));
        out.push(s.calc_err.unknown_var('x'));
        out.push(s.calc_err.unknown_function("foo"));
        out.push(s.calc_err.fn_undefined_here("tan"));

        for line in s.terminal.help_lines() {
            out.push(line.to_string());
        }
        for line in s.terminal.neofetch_lines() {
            out.push(line.to_string());
        }
        for n in 2..=6 {
            out.push(s.terminal.panel(n).to_string());
        }

        for text in &out {
            assert!(!text.trim().is_empty(), "{}: empty output", lang.code());
            assert!(
                !text.contains('{') && !text.contains('}'),
                "{}: placeholder left in {text:?}",
                lang.code()
            );
        }
    }
}

#[test]
fn calendar_helpers_cover_all_indices() {
    for lang in Language::ALL {
        let c = &lang.strings().calendar;
        for i in 0..7 {
            assert!(!c.weekday(i).is_empty());
        }
        for i in 0..12 {
            assert!(!c.month(i).is_empty());
            assert!(!c.month_short(i).is_empty());
        }
    }
}

// ------------------------------------------------------------------
// Pemindai sumber: tidak boleh ada teks tampilan yang tertinggal di kode.
// ------------------------------------------------------------------

use std::path::{Path, PathBuf};

/// Pengecualian yang disengaja: (potongan jalur file, literal, alasan).
/// Alasan wajib diisi supaya daftar ini tidak jadi tempat sampah.
const ALLOWED_LITERALS: &[(&str, &str, &str)] = &[
    ("journal/components.rs", "Linux", "tool name (proper noun)"),
    ("journal/components.rs", "Neovim", "tool name (proper noun)"),
    ("journal/components.rs", "Arch", "tool name (proper noun)"),
    (
        "journal/components.rs",
        "Hyprland",
        "tool name (proper noun)",
    ),
    ("journal/chart.rs", "M {},{}", "SVG path data"),
    (
        "journal/chart.rs",
        " C {cp1x},{p0y} {cp2x},{p1y} {p1x},{p1y}",
        "SVG path data",
    ),
    (
        "gui/ai.rs",
        "AIza...",
        "API key format hint, not a sentence",
    ),
    ("interface/window.rs", " !hidden", "Tailwind class"),
    (
        "gui/filemanager.rs",
        "home/tquilla/{breadcrumb_suffix}",
        "path of the simulated filesystem",
    ),
    (
        "tty/shell.rs",
        "README",
        "file name in the simulated filesystem",
    ),
    (
        "tty/shell.rs",
        "README.md",
        "file name in the simulated filesystem",
    ),
    (
        "components/src/api.rs",
        "API_BASE_URL",
        "name of a build-time env variable",
    ),
    ("platform.rs", "Content-Type", "HTTP header name"),
    (
        "markdown/src/lib.rs",
        "www.",
        "URL prefix recognised in text, not UI text",
    ),
];

/// Konstanta berisi data/log yang sengaja bukan teks UI: (file, nama const, alasan).
const ALLOWED_CONSTS: &[(&str, &str, &str)] = &[
    (
        "pages/booting.rs",
        "BOOT_LOG",
        "simulated systemd boot log, decoration",
    ),
    (
        "journal/util.rs",
        "MONTHS_SHORT",
        "English data key matching the Worker's dates (\"Sep 14\"), never displayed as is",
    ),
];

/// Berkas/folder yang tidak dipindai.
const SKIPPED_PATHS: &[&str] = &[
    // tabel teks itu sendiri dan test-nya
    "/languages/",
    // mesin kalkulator: isi enum galat tanpa teks, token fungsi matematika
    "/calculator/engine/",
];

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read src dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Semua literal string biasa (`"..."`) pada satu baris, tanpa tanda kutip.
fn literals(line: &str) -> Vec<String> {
    let mut found = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '/' if chars.get(i + 1) == Some(&'/') => break,
            '\'' => {
                // literal char seperti '"' atau '\'': lewati
                if chars.get(i + 2) == Some(&'\'') {
                    i += 3;
                } else if chars.get(i + 1) == Some(&'\\') && chars.get(i + 3) == Some(&'\'') {
                    i += 4;
                } else {
                    i += 1;
                }
            }
            '"' => {
                let mut text = String::new();
                i += 1;
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 1;
                    }
                    text.push(chars[i]);
                    i += 1;
                }
                found.push(text);
                i += 1;
            }
            _ => i += 1,
        }
    }
    found
}

/// Teks tanpa bagian `{...}` (interpolasi).
fn without_interpolation(text: &str) -> String {
    let mut out = String::new();
    let mut depth = 0;
    for ch in text.chars() {
        match ch {
            '{' => depth += 1,
            '}' if depth > 0 => depth -= 1,
            _ if depth == 0 => out.push(ch),
            _ => {}
        }
    }
    out
}

/// Apakah literal ini kelihatan seperti teks yang dibaca pengguna?
fn looks_like_display_text(literal: &str) -> bool {
    // URL, kode JS inline, dan markup MathML bukan teks tampilan.
    if literal.contains("://") || literal.starts_with("window.") || literal.starts_with("<m") {
        return false;
    }
    let text = without_interpolation(literal);
    let text = text.trim();
    if text.chars().count() <= 1 {
        return false; // satu karakter: "π", "C", "P"
    }
    if !text.chars().any(|c| c.is_alphabetic()) {
        return false; // simbol saja: "+", "×", "%"
    }
    let first_upper = text.chars().next().is_some_and(|c| c.is_uppercase());
    let has_sentence_marks = text.contains(['…', '!', '?', '·', '—'])
        || text.contains(". ")
        || text.contains(", ")
        || text.ends_with('.');
    let non_ascii_letter = text.chars().any(|c| c.is_alphabetic() && !c.is_ascii());
    first_upper || has_sentence_marks || non_ascii_letter
}

/// Baris yang isinya bukan teks tampilan walau berisi literal berhuruf besar.
fn is_code_context(line: &str) -> bool {
    const MARKERS: &[&str] = &[
        "class:",
        "class ",
        "const ",
        "static ",
        "asset!",
        "include_str!",
        "use ",
        "assert",
        "panic!",
        "expect(",
        "unreachable!",
        "#[",
        "href:",
        "src:",
        "key:",
        "id:",
        "r#type:",
        "rel:",
        "target:",
        "viewBox",
        "d: ",
        "fill:",
        "stroke",
        "style:",
        "format!(\"{",
        "console",
        "web_sys",
        "eval(",
        "document::",
        "Route::",
        "to: ",
    ];
    MARKERS.iter().any(|m| line.contains(m))
}

const DISPLAY_ATTRS: &[&str] = &[
    "title:",
    "alt:",
    "placeholder:",
    "aria_label:",
    "aria-label:",
    "label:",
];

fn scan(path: &Path, source: &str, findings: &mut Vec<String>) {
    let shown = path.to_string_lossy().replace('\\', "/");
    // Modul test selalu di akhir file dan boleh berisi data contoh.
    let body = match source.find("#[cfg(test)]") {
        Some(at) => &source[..at],
        None => source,
    };

    let mut in_allowed_const = false;
    for (index, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.is_empty() {
            continue;
        }
        if in_allowed_const {
            in_allowed_const = !trimmed.ends_with(';');
            continue;
        }
        if let Some((_, name, _)) = ALLOWED_CONSTS.iter().find(|(file, name, _)| {
            shown.contains(file) && trimmed.contains(&format!("const {name}"))
        }) {
            let _ = name;
            in_allowed_const = !trimmed.ends_with(';');
            continue;
        }
        let lits = literals(trimmed);
        if lits.is_empty() {
            continue;
        }

        let is_display_attr = DISPLAY_ATTRS.iter().any(|a| trimmed.starts_with(a));
        let is_code = is_code_context(trimmed) && !is_display_attr;

        for lit in lits {
            let flagged = if is_display_attr {
                // atribut tampilan harus lewat `Strings` atau interpolasi murni
                !lit.contains("://")
                    && without_interpolation(&lit)
                        .chars()
                        .any(|c| c.is_alphabetic())
            } else {
                !is_code && looks_like_display_text(&lit)
            };
            if !flagged {
                continue;
            }
            let allowed = ALLOWED_LITERALS
                .iter()
                .any(|(file, text, _why)| shown.contains(file) && *text == lit);
            if !allowed {
                findings.push(format!("{shown}:{}: {lit:?}", index + 1));
            }
        }
    }
}

#[test]
fn no_display_strings_left_in_components() {
    // `components` and the `markdown` crate (parser; must hold no UI text either).
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_files(&manifest.join("src"), &mut files);
    let before_markdown = files.len();
    rust_files(&manifest.join("../markdown/src"), &mut files);
    assert!(before_markdown > 20, "scanner found too few files");
    assert!(
        files.len() > before_markdown,
        "scanner did not find the markdown crate sources"
    );

    let mut findings = Vec::new();
    for file in files {
        let shown = file.to_string_lossy().replace('\\', "/");
        if SKIPPED_PATHS.iter().any(|p| shown.contains(p)) {
            continue;
        }
        let source = std::fs::read_to_string(&file).expect("read source");
        scan(&file, &source, &mut findings);
    }

    assert!(
        findings.is_empty(),
        "display text outside languages/language.rs (move it into Strings, or add it to \
         ALLOWED_LITERALS with a reason):\n{}",
        findings.join("\n")
    );
}

#[test]
fn scanner_flags_what_it_should() {
    assert!(looks_like_display_text("Hello there"));
    assert!(looks_like_display_text("loading…"));
    assert!(looks_like_display_text("terjadi kesalahan, coba lagi"));
    assert!(!looks_like_display_text("+"));
    assert!(!looks_like_display_text("{s.home.title}"));
    assert!(!looks_like_display_text("w-full flex"));
    assert_eq!(literals(r#"a: "x", b: "y\"z" // "no""#), vec!["x", "y\"z"]);
    assert_eq!(literals(r#"if c == '"' { "ok" }"#), vec!["ok"]);

    let mut findings = Vec::new();
    scan(
        Path::new("/x/demo.rs"),
        "rsx! {\n  button { \"Save changes\" }\n  img { alt: \"Logo\" }\n  p { \"{s.a.b}\" }\n}\n",
        &mut findings,
    );
    assert_eq!(findings.len(), 2, "{findings:?}");
}
