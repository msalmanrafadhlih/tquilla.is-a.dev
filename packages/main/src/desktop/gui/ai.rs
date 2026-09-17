use dioxus::prelude::*;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

const GEMINI_35_FLASH: &str = "gemini-3.5-flash";
const GEMINI_37_FLASH: &str = "gemini-3.7-flash";

#[derive(Clone, Copy, PartialEq)]
enum Role {
    User,
    Assistant,
}

#[derive(Clone, PartialEq)]
struct ChatTurn {
    role: Role,
    text: String,
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize, Clone)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize, Default)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
    error: Option<GeminiError>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContentResp,
}

#[derive(Deserialize)]
struct GeminiContentResp {
    parts: Vec<GeminiPart>,
}

#[derive(Deserialize)]
struct GeminiError {
    message: String,
}

async fn call_gemini(api_key: String, model: String, history: Vec<ChatTurn>) -> Result<String, String> {
    let contents: Vec<GeminiContent> = history
        .iter()
        .map(|turn| GeminiContent {
            role: match turn.role {
                Role::User => "user".to_string(),
                Role::Assistant => "model".to_string(),
            },
            parts: vec![GeminiPart { text: turn.text.clone() }],
        })
        .collect();

    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}");
    let body = GeminiRequest { contents };

    let request = Request::post(&url).header("Content-Type", "application/json").json(&body).map_err(|e| e.to_string())?;

    let response = request.send().await.map_err(|e| e.to_string())?;
    let status = response.status();
    let parsed: GeminiResponse = response.json().await.unwrap_or_default();

    if let Some(err) = parsed.error {
        return Err(err.message);
    }
    if status < 200 || status >= 300 {
        return Err(format!("Request failed (HTTP {status})."));
    }

    let text = parsed
        .candidates
        .unwrap_or_default()
        .first()
        .and_then(|c| c.content.parts.first())
        .map(|p| p.text.clone())
        .unwrap_or_else(|| "(empty response)".to_string());
    Ok(text)
}

/// A plain function (not a shared closure) so it can be called from both
/// the Enter-key handler and the Send button without any closure-capture
/// sharing concerns.
fn send_prompt(
    mut history: Signal<Vec<ChatTurn>>,
    mut draft: Signal<String>,
    mut sending: Signal<bool>,
    mut error: Signal<Option<String>>,
    api_key: String,
    model: String,
) {
    if sending() {
        return;
    }
    let text = draft();
    if text.trim().is_empty() {
        return;
    }
    if api_key.trim().is_empty() {
        error.set(Some("Enter your Gemini API key below first.".to_string()));
        return;
    }
    error.set(None);
    let mut list = history();
    list.push(ChatTurn { role: Role::User, text: text.clone() });
    history.set(list.clone());
    draft.set(String::new());
    sending.set(true);

    spawn(async move {
        match call_gemini(api_key, model, list).await {
            Ok(reply) => {
                let mut updated = history();
                updated.push(ChatTurn { role: Role::Assistant, text: reply });
                history.set(updated);
            }
            Err(e) => error.set(Some(e)),
        }
        sending.set(false);
    });
}

fn clear_chat(mut history: Signal<Vec<ChatTurn>>, mut error: Signal<Option<String>>) {
    history.set(vec![]);
    error.set(None);
}

/// Splits a reply on ``` fences so code blocks render in a monospace box
/// distinct from prose — a light touch, not real syntax highlighting.
/// Returns `(is_code, content)` pairs rather than an enum so the render
/// loop can branch with a plain `if/else` instead of `match` inside rsx.
fn parse_segments(text: &str) -> Vec<(bool, String)> {
    let mut segments = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("```") {
        if start > 0 {
            segments.push((false, rest[..start].to_string()));
        }
        let after_fence = &rest[start + 3..];
        let code_start = after_fence.find('\n').map(|i| i + 1).unwrap_or(0);
        let code_region = &after_fence[code_start..];
        match code_region.find("```") {
            Some(end) => {
                segments.push((true, code_region[..end].to_string()));
                rest = &code_region[end + 3..];
            }
            None => {
                segments.push((true, code_region.to_string()));
                rest = "";
                break;
            }
        }
    }
    if !rest.is_empty() {
        segments.push((false, rest.to_string()));
    }
    segments
}

#[component]
pub fn AiAssistantWindowContent() -> Element {
    let history: Signal<Vec<ChatTurn>> = use_signal(Vec::new);
    let mut draft = use_signal(String::new);
    let mut sending = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut api_key = use_signal(String::new);
    let mut show_key = use_signal(|| false);
    let mut model = use_signal(|| GEMINI_35_FLASH);

    let key_input_type = if show_key() { "text" } else { "password" };
    let eye_label = if show_key() { "Hide" } else { "Show" };
    let has_error = error().is_some();
    let error_message = error().unwrap_or_default();

    rsx! {
        div { class: "flex flex-col h-full text-[11px]",
            div { class: "flex-1 min-h-0 overflow-y-auto px-3 py-3 flex flex-col gap-3",
                if history().is_empty() {
                    p { class: "text-white/30", "Ask me anything — bring your own Gemini API key below." }
                }
                for (i , turn) in history().iter().enumerate() {
                    {
                        let is_user = turn.role == Role::User;
                        let segments = parse_segments(&turn.text);
                        rsx! {
                            div {
                                key: "{i}",
                                class: "flex flex-col gap-1",
                                class: if is_user { "items-end" } else { "items-start" },
                                div {
                                    class: "max-w-[85%] px-3 py-2 border",
                                    class: if is_user { "bg-white text-black border-white" } else { "border-white/15 text-white/80" },
                                    for (j , segment) in segments.iter().enumerate() {
                                        {
                                            let is_code = segment.0;
                                            let content = segment.1.clone();
                                            rsx! {
                                                if is_code {
                                                    pre {
                                                        key: "{j}",
                                                        class: "bg-black/80 text-white/90 border border-white/10 px-2 py-1.5 my-1 overflow-x-auto whitespace-pre",
                                                        "{content.trim_end()}"
                                                    }
                                                } else {
                                                    p {
                                                        key: "{j}",
                                                        class: "whitespace-pre-wrap leading-snug",
                                                        "{content.trim()}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if sending() {
                    p { class: "text-white/30", "Thinking..." }
                }
                if has_error {
                    p { class: "text-red-400", "{error_message}" }
                }
            }

            div { class: "shrink-0 border-t border-white/15 px-3 py-2 flex items-center gap-2",
                button {
                    r#type: "button",
                    title: "Clear conversation",
                    class: "text-white/40 hover:text-white transition-colors duration-150",
                    onclick: move |_| clear_chat(history, error),
                    "+"
                }
                input {
                    r#type: "text",
                    placeholder: "What would you like to know?",
                    class: "flex-1 bg-transparent outline-none text-white placeholder-white/30",
                    value: "{draft}",
                    oninput: move |evt: FormEvent| draft.set(evt.value()),
                    onkeydown: move |evt: KeyboardEvent| match evt.key() {
                        Key::Enter => {
                            evt.prevent_default();
                            send_prompt(history, draft, sending, error, api_key(), model().to_string());
                        }
                        _ => {}
                    },
                }
                button {
                    r#type: "button",
                    class: "text-white/50 hover:text-white transition-colors duration-150",
                    onclick: move |_| send_prompt(history, draft, sending, error, api_key(), model().to_string()),
                    "\u{25B7}"
                }
            }

            div { class: "shrink-0 border-t border-white/15 px-3 py-2 flex items-center gap-2 flex-wrap",
                span { class: "text-white/40", "Api key:" }
                input {
                    r#type: key_input_type,
                    placeholder: "AIza...",
                    class: "flex-1 min-w-[80px] bg-transparent outline-none border-b border-white/10 text-white placeholder-white/20",
                    value: "{api_key}",
                    oninput: move |evt: FormEvent| api_key.set(evt.value()),
                }
                button {
                    r#type: "button",
                    class: "text-white/40 hover:text-white transition-colors duration-150",
                    onclick: move |_| show_key.set(!show_key()),
                    "{eye_label}"
                }
                div { class: "flex items-center gap-1 ml-auto",
                    button {
                        r#type: "button",
                        class: "px-2 py-1 border text-[10px]",
                        class: if model() == GEMINI_35_FLASH { "bg-white text-black border-white" } else { "border-white/20 text-white/50 hover:text-white" },
                        onclick: move |_| model.set(GEMINI_35_FLASH),
                        "3.5 Flash"
                    }
                    button {
                        r#type: "button",
                        class: "px-2 py-1 border text-[10px]",
                        class: if model() == GEMINI_37_FLASH { "bg-white text-black border-white" } else { "border-white/20 text-white/50 hover:text-white" },
                        onclick: move |_| model.set(GEMINI_37_FLASH),
                        "3.7 Flash"
                    }
                }
            }
        }
    }
}
