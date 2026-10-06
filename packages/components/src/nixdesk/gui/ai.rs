use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

const GEMINI_35_FLASH: &str = "gemini-3.5-flash";
const GEMINI_37_FLASH: &str = "gemini-3.7-flash";

const MODEL_OPTIONS: [(&str, &str); 2] = [
    (GEMINI_35_FLASH, "3.5 Flash"),
    (GEMINI_37_FLASH, "3.7 Flash"),
];

const ROOT_CLASS: &str = "flex flex-col items-center justify-center gap-[25px] relative w-full h-full overflow-y-scroll";
const CONVERSATION_CLASS: &str = "flex flex-col-reverse items-end gap-[25px] relative flex-1 w-full max-w-[48rem] h-full grow overflow-y-scroll";
const USER_BUBBLE_CLASS: &str = "flex flex-col rounded-b-xl rounded-tl-xl items-start w-full max-w-max p-5 relative flex-[0_0_auto] bg-[var(--variable-collection-fg-main)]";
const USER_TEXT_CLASS: &str = "relative self-stretch whitespace-pre-wrap [font:'JetBrains_Mono-Regular',Helvetica] text-[var(--variable-collection-bg-main)] text-xs text-left tracking-[0] leading-[normal]";
const ASSISTANT_ARTICLE_CLASS: &str = "flex items-start gap-[var(--size-space-200)] relative self-stretch w-full flex-[0_0_auto] rounded-lg overflow-hidden";
const ASSISTANT_ICON_CLASS: &str = "relative w-5 h-5 aspect-[1]";
const ASSISTANT_BODY_CLASS: &str = "flex flex-col gap-3 flex-1 min-w-0";
const ASSISTANT_TEXT_CLASS: &str = "relative whitespace-pre-wrap [font:'JetBrains_Mono-Regular',Helvetica] text-[var(--variable-collection-fg-main)] text-xs tracking-[0] leading-[normal]";
const CODE_FIGURE_CLASS: &str = "flex items-start bg-white relative self-stretch w-full flex-[0_0_auto] rounded-xl overflow-hidden border border-solid border-color-border-default-default";
const CODE_GUTTER_CLASS: &str = "flex flex-col w-12 items-end pt-[var(--size-space-400)] pr-[var(--size-space-200)] pb-[var(--size-space-400)] pl-[var(--size-space-200)] relative self-stretch bg-color-background-default-default border-r [border-right-style:solid] border-color-border-default-default";
const CODE_NUMBERS_CLASS: &str = "relative w-fit mt-[-1.00px] whitespace-pre font-body-code font-[number:var(--body-code-font-weight)] text-color-text-default-tertiary text-[length:var(--body-code-font-size)] text-right tracking-[var(--body-code-letter-spacing)] leading-[var(--body-code-line-height)] [font-style:var(--body-code-font-style)]";
const CODE_PRE_CLASS: &str = "inline-flex flex-col items-start pt-[var(--size-space-400)] pr-[var(--size-space-200)] pb-[var(--size-space-400)] pl-[var(--size-space-200)] relative flex-1 min-w-0 overflow-auto bg-color-background-default-secondary border-r [border-right-style:solid] border-color-border-default-default whitespace-pre font-body-code font-[number:var(--body-code-font-weight)] text-[length:var(--body-code-font-size)] tracking-[var(--body-code-letter-spacing)] leading-[var(--body-code-line-height)] [font-style:var(--body-code-font-style)]";
const FOOTER_CLASS: &str = "flex flex-col items-start relative w-full max-w-[48rem] flex-[0_0_auto] border border-solid border-[var(--variable-collection-fg-secondary)]";
const FORM_CLASS: &str = "flex items-center gap-5 p-2.5 relative self-stretch w-full h-max flex-[0_0_auto]";
const QUESTION_INPUT_CLASS: &str = "relative flex-1 min-w-0 h-[22px] bg-transparent border-0 outline-none [font:'JetBrains_Mono-Light',Helvetica] font-light text-[var(--variable-collection-fg-main)] text-sm tracking-[0] leading-[19.6px]";
const ICON_BUTTON_CLASS: &str = "flex relative h-max opacity-70 hover:opacity-100 aspect-[1] bg-transparent border-0 items-center justify-center cursor-pointer";
const KEY_ROW_CLASS: &str = "w-full h-12 flex items-center justify-center p-2.5 relative max-w-full";
const KEY_GROUP_CLASS: &str = "flex items-center gap-2.5 relative flex-1 self-stretch grow";
const KEY_LABEL_CLASS: &str = "relative flex text-left w-fit [font:'JetBrains_Mono-Thin',Helvetica] font-thin text-[var(--variable-collection-fg-secondary)] text-xs text-center tracking-[0] leading-[normal] whitespace-nowrap";
const KEY_FIELD_CLASS: &str = "w-full flex items-center gap-[5px] p-[5px] relative flex-1 self-stretch grow";
const KEY_INPUT_CLASS: &str = "flex-1 min-w-0 w-full text-xs bg-transparent outline-none border-0 border-b border-[var(--variable-collection-fg-secondary)] text-[var(--variable-collection-fg-main)] placeholder-[var(--variable-collection-fg-secondary)] opacity-50 focus:opacity-100 transition-opacity duration-300";
const SHOW_BUTTON_CLASS: &str = "text-[var(--variable-collection-fg-secondary)] hover:text-[var(--variable-collection-fg-main)] transition-colors duration-150 text-xs bg-transparent border-0 cursor-pointer";
const INFO_BUTTON_CLASS: &str = "h-full relative aspect-[1] border-0 bg-transparent p-[1px] opacity-70 hover:opacity-100 cursor-pointer";
const SETTINGS_WRAP_CLASS: &str = "relative flex-[0_0_auto] h-full";
const SETTINGS_TOGGLE_CLASS: &str = "inline-flex h-full p-[5px] items-center relative bg-transparent border-0 opacity-70 hover:opacity-100 cursor-pointer";
const MODEL_MENU_CLASS: &str = "absolute bottom-full right-0 mb-2 flex flex-col gap-1 p-1 border border-solid border-[var(--variable-collection-fg-secondary)] bg-[var(--variable-collection-bg-main)]";

const ICON_ASSISTANT: Asset = asset!("/assets/logo-Ai-Assistent.svg"); 
const ICON_ADD: Asset = asset!("/assets/icon-add.svg"); 
const ICON_SEND: Asset = asset!("assets/icon-send.svg"); 
const ICON_INFO: Asset = asset!("assets/icon-info.svg"); 
const ICON_TOGGLE: Asset = asset!("assets/icon-toggle-popup.svg"); 

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

    let payload = serde_json::to_string(&body).map_err(|e| e.to_string())?;

    let response = crate::platform::http::post_json(&url, payload).await?;
    let status = response.status;
    let parsed: GeminiResponse = serde_json::from_str(&response.body).unwrap_or_default();

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
/// the form submit handler and any other entry point without any
/// closure-capture sharing concerns.
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

/// Line numbers for the code gutter, one per line, joined with newlines
/// so a single `whitespace-pre` element can render them.
fn code_gutter(code: &str) -> String {
    let lines = code.lines().count().max(1);
    (1..=lines)
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn model_option_class(active: bool) -> &'static str {
    if active {
        "px-2 py-1 text-left whitespace-nowrap text-[10px] bg-[var(--variable-collection-fg-main)] text-black border-0"
    } else {
        "px-2 py-1 text-left whitespace-nowrap text-[10px] text-[var(--variable-collection-fg-secondary)] hover:text-[var(--variable-collection-fg-main)] bg-transparent border-0"
    }
}

#[component]
pub fn AiAssistantWindowContent() -> Element {
    let history: Signal<Vec<ChatTurn>> = use_signal(Vec::new);
    let mut draft = use_signal(String::new);
    let mut sending = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut api_key = use_signal(String::new);
    let mut show_key = use_signal(|| false);
    let mut show_models = use_signal(|| false);
    let mut model = use_signal(|| GEMINI_35_FLASH);

    let turns = history();
    let key_input_type = if show_key() { "text" } else { "password" };
    let eye_label = if show_key() { "Hide" } else { "Show" };
    let models_expanded = if show_models() { "true" } else { "false" };
    let has_error = error().is_some();
    let error_message = error().unwrap_or_default();

    rsx! {
        div { class: ROOT_CLASS,

            // conversation — flex-col-reverse, so the DOM runs bottom to top:
            // the status indicators come first, then turns from newest to oldest
            section {
                "aria-label": "Conversation",
                class: CONVERSATION_CLASS,

                if sending() {
                    p { class: "text-[var(--variable-collection-fg-secondary)] text-xs", "Thinking..." }
                }
                if has_error {
                    p { class: "text-[var(--variable-collection-red text)]-xs", "{error_message}" }
                }

                for (idx, turn) in turns.iter().enumerate().rev() {
                    {
                        let key = format!("turn-{idx}");
                        if turn.role == Role::User {
                            let user_text = turn.text.clone();
                            rsx! {
                                article {
                                    key: "{key}",
                                    class: USER_BUBBLE_CLASS,
                                    p { class: USER_TEXT_CLASS, "{user_text}" }
                                }
                            }
                        } else {
                            let segments = parse_segments(&turn.text);
                            rsx! {
                                article {
                                    key: "{key}",
                                    class: ASSISTANT_ARTICLE_CLASS,
                                    img {
                                        class: ASSISTANT_ICON_CLASS,
                                        src: ICON_ASSISTANT,
                                        alt: "",
                                        "aria-hidden": "true",
                                    }
                                    div {
                                        class: ASSISTANT_BODY_CLASS,
                                        for (j, segment) in segments.iter().enumerate() {
                                            {
                                                let is_code = segment.0;
                                                let code = segment.1.trim_end().to_string();
                                                let gutter = code_gutter(&code);
                                                let text = segment.1.trim().to_string();
                                                rsx! {
                                                    if is_code {
                                                        figure {
                                                            key: "code-{j}",
                                                            class: CODE_FIGURE_CLASS,
                                                            figcaption { class: "sr-only", "Code block" }
                                                            div {
                                                                class: CODE_GUTTER_CLASS,
                                                                "aria-hidden": "true",
                                                                div { class: CODE_NUMBERS_CLASS, "{gutter}" }
                                                            }
                                                            pre {
                                                                class: CODE_PRE_CLASS,
                                                                code { "{code}" }
                                                            }
                                                        }
                                                    } else {
                                                        p {
                                                            key: "text-{j}",
                                                            class: ASSISTANT_TEXT_CLASS,
                                                            "{text}"
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
                }

                if turns.is_empty() {
                    p {
                        class: "text-[var(--variable-collection-fg-secondary)] text-xs",
                        "Ask me anything — bring your own Gemini API key below."
                    }
                }
            }

            // footer: question form + API key row
            footer {
                class: FOOTER_CLASS,

                form {
                    "aria-label": "Ask the AI assistant",
                    class: FORM_CLASS,
                    onsubmit: move |evt: FormEvent| {
                        evt.prevent_default();
                        send_prompt(history, draft, sending, error, api_key(), model().to_string());
                    },

                    button {
                        r#type: "button",
                        "aria-label": "Add files, and more",
                        title: "Add Attachments",
                        class: ICON_BUTTON_CLASS,
                        // salah ini,seharusnya add file gitu
                        onclick: move |_| clear_chat(history, error),
                        img { src: ICON_ADD, alt: "", "aria-hidden": "true" }
                    }

                    label {
                        r#for: "assistant-question",
                        class: "sr-only",
                        "What would you like to know?"
                    }
                    input {
                        id: "assistant-question",
                        name: "question",
                        r#type: "text",
                        autocomplete: "off",
                        placeholder: "What would you like to know?",
                        class: QUESTION_INPUT_CLASS,
                        value: "{draft}",
                        oninput: move |evt: FormEvent| draft.set(evt.value()),
                    }

                    button {
                        r#type: "submit",
                        "aria-label": "Send question",
                        class: ICON_BUTTON_CLASS,
                        img { src: ICON_SEND, alt: "", "aria-hidden": "true" }
                    }
                }

                div {
                    class: KEY_ROW_CLASS,

                    div {
                        class: KEY_GROUP_CLASS,
                        label {
                            r#for: "api-key",
                            class: KEY_LABEL_CLASS,
                            "Api key:"
                        }

                        div {
                            class: KEY_FIELD_CLASS,
                            div {
                                class: "flex items-center justify-center relative flex-1 grow",
                                input {
                                    id: "api-key",
                                    "aria-describedby": "api-key-help",
                                    placeholder: "AIza...",
                                    r#type: key_input_type,
                                    class: KEY_INPUT_CLASS,
                                    value: "{api_key}",
                                    oninput: move |evt: FormEvent| api_key.set(evt.value()),
                                }
                            }
                            button {
                                r#type: "button",
                                class: SHOW_BUTTON_CLASS,
                                onclick: move |_| show_key.set(!show_key()),
                                "{eye_label}"
                            }
                            a {
                                r#type: "button",
                                "aria-label": "API Key information",
                                target: "_blank",
                                href: "https://aistudio.google.com/app/api-keys",
                                title: "The key is only kept in memory for this session.",
                                class: INFO_BUTTON_CLASS,
                                img {
                                    src: ICON_INFO,
                                    alt: "",
                                    "aria-hidden": "true",
                                    class: "h-full",
                                }
                            }
                        }
                    }

                    // settings toggle → opens the model picker
                    div {
                        class: SETTINGS_WRAP_CLASS,
                        button {
                            r#type: "button",
                            "aria-label": "Open assistant settings",
                            "aria-expanded": "{models_expanded}",
                            class: SETTINGS_TOGGLE_CLASS,
                            onclick: move |_| show_models.set(!show_models()),
                            img {
                                src: ICON_TOGGLE,
                                alt: "",
                                "aria-hidden": "true",
                                class: "relative aspect-[1] h-full",
                            }
                        }

                        if show_models() {
                            div {
                                class: MODEL_MENU_CLASS,
                                for (id, label) in MODEL_OPTIONS {
                                    button {
                                        key: "{id}",
                                        r#type: "button",
                                        class: model_option_class(model() == id),
                                        onclick: move |_| {
                                            model.set(id);
                                            show_models.set(false);
                                        },
                                        "{label}"
                                    }
                                }
                            }
                        }
                    }
                }

                p {
                    id: "api-key-help",
                    class: "sr-only",
                    "Your API key is only kept in memory for this session and hidden by default."
                }
            }
        }
    }
}
