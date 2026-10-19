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
