use dioxus::prelude::*;

/// Fire-and-forget JS eval — spawns the async eval so call sites (mostly
/// onclick/oninput handlers) can stay plain synchronous closures instead
/// of each wrapping their own `spawn(async move { ... })`.
pub fn eval_js(js: String) {
    spawn(async move {
        document::eval(&js).await.ok();
    });
}

/// Escapes `'` and `\` so a Rust string can be safely dropped into a
/// single-quoted JS string literal inside an `eval` call. Applied
/// wherever a dynamic string (a stream URL, a user-typed bookmark URL)
/// gets interpolated into JS.
pub fn js_string_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

const SET_FIELD_JS: &str = include_str!("../../js/set_field.js");

/// Builds the script that rewrites an `<input>`/`<textarea>` from Rust.
/// `only_if` is the text that must still be in the field for the rewrite to
/// happen (see `js/set_field.js`); `None` always rewrites.
pub fn set_field_script(id: &str, value: &str, only_if: Option<&str>) -> String {
    // JSON string literals are valid JS string literals, so no hand escaping.
    let lit = |s: &str| serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into());
    let expect = only_if.map_or_else(|| "null".to_string(), lit);
    format!(
        "const ID = {}; const VALUE = {}; const EXPECT = {};\n{SET_FIELD_JS}",
        lit(id),
        lit(value),
        expect
    )
}

/// Text fields are NOT bound to a signal (`value: "{signal}"`): on the
/// desktop/mobile WebViews that drops keystrokes. The DOM owns the text, the
/// signal only mirrors it through `oninput`, and this rewrites the DOM when
/// the program needs to (clearing after send, recalling history).
pub fn set_field(id: &str, value: &str, only_if: Option<&str>) {
    eval_js(set_field_script(id, value, only_if));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_field_script_quotes_everything_as_json() {
        let js = set_field_script("a\"b", "line\nbreak 'q' \\ </script>", Some("x\ty"));
        assert!(js.starts_with(
            "const ID = \"a\\\"b\"; const VALUE = \"line\\nbreak 'q' \\\\ </script>\"; const EXPECT = \"x\\ty\";\n"
        ));
        assert!(js.contains("el.value = VALUE"));
    }

    #[test]
    fn set_field_script_without_guard_passes_null() {
        let js = set_field_script("f", "", None);
        assert!(js.starts_with("const ID = \"f\"; const VALUE = \"\"; const EXPECT = null;\n"));
    }
}
