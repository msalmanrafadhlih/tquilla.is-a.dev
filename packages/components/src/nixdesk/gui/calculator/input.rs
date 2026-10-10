//! Logika penyuntingan ekspresi (murni, tanpa Dioxus, sehingga bisa diuji).

use super::engine::parser::NAMES;

const BINOPS: &str = "+-−×÷";
/// Teks sisipan yang melanjutkan hasil sebelumnya (bukan memulai ekspresi baru).
const CONTINUATIONS: &str = "+-×÷^!%°";

fn is_binop_text(t: &str) -> bool {
    matches!(t, "+" | "-" | "×" | "÷")
}

/// Apakah angka terakhir di `expr` sudah punya tanda desimal?
fn last_number_has_decimal(expr: &str) -> bool {
    for c in expr.chars().rev() {
        match c {
            ',' | '.' => return true,
            '0'..='9' => continue,
            _ => return false,
        }
    }
    false
}

/// Sisipkan `text` di ujung `expr`.
/// `fresh` = true bila ekspresi saat ini adalah hasil dari menekan "=".
pub fn insert(expr: &mut String, fresh: bool, text: &str) {
    if fresh {
        let continues = text
            .chars()
            .next()
            .is_some_and(|c| CONTINUATIONS.contains(c));
        if !continues {
            expr.clear();
        }
    }

    if text == "," {
        if last_number_has_decimal(expr) {
            return;
        }
        if !expr.chars().last().is_some_and(|c| c.is_ascii_digit()) {
            expr.push('0');
        }
    }

    if is_binop_text(text) {
        match expr.chars().last() {
            None => {
                // operator biner di awal tidak bermakna; hanya +/- sebagai tanda
                if text == "×" || text == "÷" {
                    return;
                }
            }
            Some(last) if BINOPS.contains(last) => {
                // 5×-3 diizinkan; selain itu operator baru menggantikan yang lama
                if !(text == "-" && (last == '×' || last == '÷')) {
                    expr.pop();
                }
            }
            Some('(') | Some(';') if text == "×" || text == "÷" => return,
            _ => {}
        }
    }

    expr.push_str(text);
}

/// Hapus satu "token" dari ujung: nama fungsi beserta kurungnya dihapus sekaligus.
pub fn backspace(expr: &mut String) {
    if expr.ends_with("√(") {
        expr.truncate(expr.len() - "√(".len());
        return;
    }
    for name in NAMES {
        let with_paren = format!("{name}(");
        if expr.ends_with(&with_paren) {
            expr.truncate(expr.len() - with_paren.len());
            return;
        }
    }
    expr.pop();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(steps: &[&str]) -> String {
        let mut s = String::new();
        for t in steps {
            insert(&mut s, false, t);
        }
        s
    }

    #[test]
    fn desimal() {
        assert_eq!(run(&["1", ",", "5"]), "1,5");
        assert_eq!(run(&["1", ",", "5", ","]), "1,5");
        assert_eq!(run(&[","]), "0,");
        assert_eq!(run(&["2", "+", ","]), "2+0,");
        assert_eq!(run(&["1,5", "+", "2", ","]), "1,5+2,");
    }

    #[test]
    fn operator_menggantikan() {
        assert_eq!(run(&["5", "+", "×"]), "5×");
        assert_eq!(run(&["5", "×", "-"]), "5×-");
        assert_eq!(run(&["5", "-", "+"]), "5+");
        assert_eq!(run(&["×"]), "");
        assert_eq!(run(&["-", "5"]), "-5");
        assert_eq!(run(&["(", "×"]), "(");
    }

    #[test]
    fn setelah_sama_dengan() {
        let mut s = "100".to_string();
        insert(&mut s, true, "+");
        assert_eq!(s, "100+");
        let mut s = "100".to_string();
        insert(&mut s, true, "7");
        assert_eq!(s, "7");
        let mut s = "100".to_string();
        insert(&mut s, true, "sin(");
        assert_eq!(s, "sin(");
        let mut s = "100".to_string();
        insert(&mut s, true, ",");
        assert_eq!(s, "0,");
        let mut s = "7".to_string();
        insert(&mut s, true, "!");
        assert_eq!(s, "7!");
    }

    #[test]
    fn hapus() {
        let mut s = "2+sin(".to_string();
        backspace(&mut s);
        assert_eq!(s, "2+");
        backspace(&mut s);
        assert_eq!(s, "2");
        backspace(&mut s);
        backspace(&mut s);
        assert_eq!(s, "");
        let mut s = "arcsin(".to_string();
        backspace(&mut s);
        assert_eq!(s, "");
        let mut s = "√(".to_string();
        backspace(&mut s);
        assert_eq!(s, "");
    }
}
