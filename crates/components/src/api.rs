//! Typed client for the REST API (`tquilla-api`, a separate repository).
//!
//! The API base URL is fixed at **build time** through the `API_BASE_URL`
//! environment variable (see README, "Fullstack dan database"):
//!
//! ```text
//! API_BASE_URL=http://127.0.0.1:8787 dx serve --package web --platform web
//! ```
//!
//! If it is missing or malformed every call returns [`ApiError::NotConfigured`]
//! instead of silently talking to a default host. Request and response types
//! live in the shared `api-types` crate, so the client and the server cannot
//! drift apart silently.
//!
//! The client never sends cookies or credentials, and it never trusts what the
//! server returns: bodies that do not parse become [`ApiError::Decode`], never
//! a panic. Whatever is validated here (body size, path prefix) is only for a
//! fast, friendly failure; the server validates everything again.

use std::sync::OnceLock;

use serde::{de::DeserializeOwned, Serialize};

use api_types::validate::{is_http_url, is_route_or_url};
pub use api_types::{
    ApiErrorBody, ApiErrorDetail, Bookmark, ChatMessage, Embience, ErrorCode, Generation,
    HealthResponse, Items, NewMessage, Preset, RadioStation, ServiceStatus, API_PREFIX,
};

use crate::languages::{Language, Strings};
use crate::platform::http;

/// Everything that can go wrong with a call. `Display` is short, in English
/// and safe to show in the UI (no URLs, no response bodies).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    /// `API_BASE_URL` was not set (or not a valid `http(s)://host[:port]`) at build time.
    NotConfigured,
    /// The path is not under `/api/v1/`; a bug in the caller, never sent.
    InvalidPath,
    /// The request body is larger than the server accepts; never sent.
    BodyTooLarge,
    /// The request did not reach the server (offline, DNS, CORS, ...).
    Network(String),
    /// The server answered with a non-2xx status. `detail` is `None` when the
    /// body was not the documented error shape (e.g. an HTML 502 from a proxy,
    /// or an error code from a newer server than this client knows).
    Status {
        status: u16,
        detail: Option<ApiErrorDetail>,
    },
    /// A 2xx response whose body is not the expected JSON.
    Decode(String),
}

impl ApiError {
    /// Worth trying again later (network problem, 5xx, 429)?
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Network(_) => true,
            Self::Status { status, .. } => *status == 429 || *status >= 500,
            _ => false,
        }
    }

    /// The documented error code, if the server sent one.
    pub fn code(&self) -> Option<ErrorCode> {
        match self {
            Self::Status { detail, .. } => detail.as_ref().map(|d| d.code),
            _ => None,
        }
    }
}

impl ApiError {
    /// Short message in the given language (see `languages::Strings::api`).
    /// Never contains URLs or response bodies, so it is safe to show.
    pub fn message(&self, s: &Strings) -> &'static str {
        let t = &s.api;
        match self {
            Self::NotConfigured => t.not_configured,
            Self::InvalidPath => t.invalid_request,
            Self::BodyTooLarge => t.too_large,
            Self::Network(_) => t.unreachable,
            Self::Status { status, detail } => match detail.as_ref().map(|d| d.code) {
                Some(ErrorCode::RateLimited) => t.rate_limited,
                Some(ErrorCode::PayloadTooLarge) => t.too_large,
                Some(ErrorCode::NotFound) => t.not_found,
                _ if *status >= 500 => t.server_error,
                _ => t.rejected,
            },
            Self::Decode(_) => t.bad_response,
        }
    }
}

/// `Display` is the default language (English), for logs and tests; the UI
/// should call [`ApiError::message`] with `use_strings()`.
impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message(Language::default().strings()))
    }
}

impl std::error::Error for ApiError {}

// ---------------------------------------------------------------------------
// Base URL and request URL (pure, unit-tested)
// ---------------------------------------------------------------------------

/// Validates a raw `API_BASE_URL`: only `http://` or `https://` plus host and
/// optional port (and, for convenience, a trailing `/` which is dropped).
/// No path, query, fragment, credentials or whitespace.
fn normalize_base(raw: Option<&str>) -> Result<String, ApiError> {
    let raw = raw.map(str::trim).unwrap_or_default();
    let raw = raw.strip_suffix('/').unwrap_or(raw);
    let host = raw
        .strip_prefix("https://")
        .or_else(|| raw.strip_prefix("http://"))
        .ok_or(ApiError::NotConfigured)?;
    let valid = !host.is_empty()
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':' | '[' | ']'));
    if valid {
        Ok(raw.to_owned())
    } else {
        Err(ApiError::NotConfigured)
    }
}

/// The configured base URL (computed once).
pub fn base_url() -> Result<&'static str, ApiError> {
    static BASE: OnceLock<Result<String, ApiError>> = OnceLock::new();
    BASE.get_or_init(|| normalize_base(option_env!("API_BASE_URL")))
        .as_deref()
        .map_err(Clone::clone)
}

/// `base` + `path`, where `path` must be an endpoint path from
/// `api_types::paths` (starts with `/api/v1/`; no query string, fragment,
/// `..` or whitespace).
fn join_url(base: &str, path: &str) -> Result<String, ApiError> {
    let under_prefix = path
        .strip_prefix(API_PREFIX)
        .is_some_and(|rest| rest.starts_with('/') && rest.len() > 1);
    let clean = !path.contains("..")
        && !path
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || matches!(c, '?' | '#' | '\\'));
    if under_prefix && clean {
        Ok(format!("{base}{path}"))
    } else {
        Err(ApiError::InvalidPath)
    }
}

// ---------------------------------------------------------------------------
// Response handling (pure, unit-tested)
// ---------------------------------------------------------------------------

fn parse_response<T: DeserializeOwned>(status: u16, body: &str) -> Result<T, ApiError> {
    if (200..300).contains(&status) {
        serde_json::from_str(body).map_err(|e| ApiError::Decode(e.to_string()))
    } else {
        let detail = serde_json::from_str::<ApiErrorBody>(body)
            .ok()
            .map(|b| b.error);
        Err(ApiError::Status { status, detail })
    }
}

// ---------------------------------------------------------------------------
// Calls
// ---------------------------------------------------------------------------

/// `GET` an endpoint and decode the JSON body.
pub async fn get<T: DeserializeOwned>(path: &str) -> Result<T, ApiError> {
    get_with_query(path, "").await
}

/// Like [`get`], with a ready-made query string (`"after=3&limit=50"`, no
/// leading `?`). Only numbers go in here, never user text, so nothing needs
/// escaping; see [`messages_query`].
async fn get_with_query<T: DeserializeOwned>(path: &str, query: &str) -> Result<T, ApiError> {
    let mut url = join_url(base_url()?, path)?;
    if !query.is_empty() {
        url.push('?');
        url.push_str(query);
    }
    let resp = http::get(&url).await.map_err(ApiError::Network)?;
    parse_response(resp.status, &resp.body)
}

/// `POST` a JSON body to an endpoint and decode the JSON response.
pub async fn post<B: Serialize, T: DeserializeOwned>(path: &str, body: &B) -> Result<T, ApiError> {
    let url = join_url(base_url()?, path)?;
    let body = serde_json::to_string(body).map_err(|e| ApiError::Decode(e.to_string()))?;
    if body.len() > api_types::limits::MAX_BODY_BYTES {
        return Err(ApiError::BodyTooLarge);
    }
    let resp = http::post_json(&url, body)
        .await
        .map_err(ApiError::Network)?;
    parse_response(resp.status, &resp.body)
}

/// The health endpoint answers `503` with the same JSON shape when the
/// database is down (`status: "degraded"`); that is data, not an error.
fn parse_health(status: u16, body: &str) -> Result<HealthResponse, ApiError> {
    if status == 503 {
        if let Ok(health) = serde_json::from_str::<HealthResponse>(body) {
            return Ok(health);
        }
    }
    parse_response(status, body)
}

/// `GET /api/v1/health`. A reachable server with a broken database is
/// `Ok` with `status == ServiceStatus::Degraded`.
pub async fn health() -> Result<HealthResponse, ApiError> {
    let url = join_url(base_url()?, api_types::paths::HEALTH)?;
    let resp = http::get(&url).await.map_err(ApiError::Network)?;
    parse_health(resp.status, &resp.body)
}

// ---------------------------------------------------------------------------
// Content (read-only lists)
// ---------------------------------------------------------------------------
//
// The client does not trust what the server sends: every list passes through a
// `sanitize_*` function that drops rows with unusable links before any
// component can render them (an `<iframe src>`, an `<audio src>` or a menu
// link must never be `javascript:` or `data:`). The server filters too; this is
// the second, independent check.

fn sanitize_generations(mut items: Vec<Generation>) -> Vec<Generation> {
    items.retain(|g| is_route_or_url(&g.link));
    items
}

fn sanitize_bookmarks(mut items: Vec<Bookmark>) -> Vec<Bookmark> {
    items.retain(|b| is_http_url(&b.url));
    items
}

fn sanitize_radio(mut items: Vec<RadioStation>) -> Vec<RadioStation> {
    items.retain(|r| is_http_url(&r.stream_url));
    for r in &mut items {
        r.official_link.take_if(|u| !is_http_url(u));
        r.image_url.take_if(|u| !is_http_url(u));
    }
    items
}

fn sanitize_embiences(mut items: Vec<Embience>) -> Vec<Embience> {
    items.retain(|e| is_http_url(&e.icon_url) && is_http_url(&e.stream_url));
    items
}

/// Keeps only ids that exist in `embiences` and drops presets left empty
/// (an empty preset is a button that does nothing).
pub fn sanitize_presets(mut items: Vec<Preset>, embiences: &[Embience]) -> Vec<Preset> {
    for p in &mut items {
        p.embience_ids
            .retain(|id| embiences.iter().any(|e| e.id == *id));
        let mut seen = Vec::new();
        p.embience_ids.retain(|id| {
            let fresh = !seen.contains(id);
            seen.push(*id);
            fresh
        });
    }
    items.retain(|p| !p.embience_ids.is_empty());
    items
}

/// `GET /api/v1/generations`, newest first.
pub async fn generations() -> Result<Vec<Generation>, ApiError> {
    let list: Items<Generation> = get(api_types::paths::GENERATIONS).await?;
    Ok(sanitize_generations(list.items))
}

/// `GET /api/v1/bookmarks`.
pub async fn bookmarks() -> Result<Vec<Bookmark>, ApiError> {
    let list: Items<Bookmark> = get(api_types::paths::BOOKMARKS).await?;
    Ok(sanitize_bookmarks(list.items))
}

/// `GET /api/v1/radio` (active stations).
pub async fn radio() -> Result<Vec<RadioStation>, ApiError> {
    let list: Items<RadioStation> = get(api_types::paths::RADIO).await?;
    Ok(sanitize_radio(list.items))
}

/// `GET /api/v1/embiences`.
pub async fn embiences() -> Result<Vec<Embience>, ApiError> {
    let list: Items<Embience> = get(api_types::paths::EMBIENCES).await?;
    Ok(sanitize_embiences(list.items))
}

/// `GET /api/v1/presets`, raw: pass the result through [`sanitize_presets`]
/// together with the embience list.
pub async fn presets() -> Result<Vec<Preset>, ApiError> {
    let list: Items<Preset> = get(api_types::paths::PRESETS).await?;
    Ok(list.items)
}

// ---------------------------------------------------------------------------
// Guestbook
// ---------------------------------------------------------------------------
//
// The server still calls these "messages" (route and table names stayed when
// Live Chat became the Guestbook, so the Worker could be replaced without a
// breaking rename); the UI calls them entries.

/// Which slice of the guestbook to ask for. At most one of the two cursors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    /// The newest entries.
    #[default]
    Latest,
    /// Entries newer than this id (a cheap "refresh").
    After(u32),
    /// Entries just older than this id (the "load older" button).
    Before(u32),
}

/// The query string for `GET /messages`. Numbers only, so it is always safe.
fn messages_query(page: Page, limit: Option<u32>) -> String {
    let mut parts = Vec::new();
    match page {
        Page::Latest => {}
        Page::After(id) => parts.push(format!("after={id}")),
        Page::Before(id) => parts.push(format!("before={id}")),
    }
    if let Some(n) = limit {
        parts.push(format!("limit={n}"));
    }
    parts.join("&")
}

/// Same idea as the content sanitizers, for rows that anybody on the internet
/// could have written: the link a page would render must be `http(s)`.
fn sanitize_messages(mut items: Vec<ChatMessage>) -> Vec<ChatMessage> {
    for m in &mut items {
        m.url.take_if(|u| !is_http_url(u));
    }
    items
}

/// `GET /api/v1/messages`, oldest first within the page.
pub async fn messages(page: Page, limit: Option<u32>) -> Result<Vec<ChatMessage>, ApiError> {
    let query = messages_query(page, limit);
    let list: Items<ChatMessage> = get_with_query(api_types::paths::MESSAGES, &query).await?;
    Ok(sanitize_messages(list.items))
}

/// `POST /api/v1/messages`. The server cleans and validates everything again
/// and answers with the stored entry.
pub async fn post_message(new: &NewMessage) -> Result<ChatMessage, ApiError> {
    let mut sent: ChatMessage = post(api_types::paths::MESSAGES, new).await?;
    sent.url.take_if(|u| !is_http_url(u));
    Ok(sent)
}

/// A `2xx` with no body to read (`204`), or the documented error.
fn parse_empty(status: u16, body: &str) -> Result<(), ApiError> {
    if (200..300).contains(&status) {
        Ok(())
    } else {
        parse_response::<()>(status, body)
    }
}

/// `POST /api/v1/messages/{id}/report`. The server answers `204` whether or
/// not the entry still exists, so success only means "received".
pub async fn report_message(id: u32) -> Result<(), ApiError> {
    let url = join_url(base_url()?, &api_types::paths::message_report(id))?;
    let resp = http::post_json(&url, "{}".to_owned())
        .await
        .map_err(ApiError::Network)?;
    parse_empty(resp.status, &resp.body)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chat_message(id: u32) -> ChatMessage {
        ChatMessage {
            id,
            username: "u".into(),
            url: None,
            message: "m".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn messages_query_has_numbers_only() {
        assert_eq!(messages_query(Page::Latest, None), "");
        assert_eq!(messages_query(Page::After(7), None), "after=7");
        assert_eq!(messages_query(Page::Before(7), None), "before=7");
        assert_eq!(messages_query(Page::Latest, Some(50)), "limit=50");
        assert_eq!(messages_query(Page::After(7), Some(50)), "after=7&limit=50");
        assert_eq!(
            messages_query(Page::Before(7), Some(20)),
            "before=7&limit=20"
        );
    }

    #[test]
    fn unsafe_guestbook_links_are_blanked_but_the_entry_stays() {
        let mut bad = chat_message(1);
        bad.url = Some("javascript:alert(1)".into());
        let mut good = chat_message(2);
        good.url = Some("https://a.dev".into());
        let kept = sanitize_messages(vec![bad, good]);
        assert_eq!(kept.len(), 2);
        assert_eq!(kept[0].url, None);
        assert_eq!(kept[1].url.as_deref(), Some("https://a.dev"));
    }

    #[test]
    fn guestbook_list_decodes_the_documented_shape_and_the_one_from_the_chat_era() {
        let body = r#"{"items":[{"id":3,"username":"Moch","url":null,
            "message":"hi","created_at":"2026-01-01T00:00:00Z"}]}"#;
        let list: Items<ChatMessage> = parse_response(200, body).unwrap();
        assert_eq!(list.items[0].id, 3);
        // A server that has not been updated yet still sends the image columns.
        let old = r#"{"items":[{"id":3,"username":"Moch","url":null,"avatar_url":null,
            "message":"hi","attachment_url":null,"created_at":"2026-01-01T00:00:00Z"}]}"#;
        assert!(parse_response::<Items<ChatMessage>>(200, old).is_ok());
        let limited: Result<Items<ChatMessage>, _> = parse_response(
            429,
            r#"{"error":{"code":"rate_limited","message":"Too many messages."}}"#,
        );
        assert_eq!(limited.unwrap_err().code(), Some(ErrorCode::RateLimited));
    }

    #[test]
    fn an_empty_204_is_success_and_errors_keep_their_code() {
        assert_eq!(parse_empty(204, ""), Ok(()));
        assert_eq!(parse_empty(200, "ignored"), Ok(()));
        let limited = parse_empty(
            429,
            r#"{"error":{"code":"rate_limited","message":"Too many requests."}}"#,
        );
        assert_eq!(limited.unwrap_err().code(), Some(ErrorCode::RateLimited));
        let forbidden = parse_empty(403, "<html>proxy</html>");
        assert_eq!(forbidden.unwrap_err().code(), None);
    }

    fn generation(n: u32, link: &str) -> Generation {
        Generation {
            number: n,
            label: "x".into(),
            link: link.into(),
            kernel: "1".into(),
            built_on: "2026-01-01".into(),
        }
    }

    fn embience(id: u32, stream: &str) -> Embience {
        Embience {
            id,
            name: format!("e{id}"),
            icon_url: "https://x/i.svg".into(),
            stream_url: stream.into(),
        }
    }

    #[test]
    fn unsafe_menu_links_are_dropped_but_order_is_kept() {
        let kept = sanitize_generations(vec![
            generation(5, "/deisktify"),
            generation(4, "javascript:alert(1)"),
            generation(3, "//evil.example"),
            generation(2, "https://github.com/x"),
            generation(1, "data:text/html,x"),
        ]);
        assert_eq!(kept.iter().map(|g| g.number).collect::<Vec<_>>(), [5, 2]);
    }

    #[test]
    fn only_http_bookmarks_survive() {
        let b = |u: &str| Bookmark {
            id: 1,
            label: "x".into(),
            url: u.into(),
        };
        let kept = sanitize_bookmarks(vec![b("https://a.b"), b("javascript:x"), b("/local")]);
        assert_eq!(kept.len(), 1);
    }

    #[test]
    fn radio_needs_a_good_stream_and_blanks_bad_optional_links() {
        let r = |stream: &str, link: Option<&str>, image: Option<&str>| RadioStation {
            id: 1,
            label: "x".into(),
            slogan: String::new(),
            stream_url: stream.into(),
            official_link: link.map(Into::into),
            image_url: image.map(Into::into),
        };
        let kept = sanitize_radio(vec![
            r(
                "https://a.b/s",
                Some("https://a.b"),
                Some("https://a.b/i.png"),
            ),
            r(
                "https://a.b/s",
                Some("javascript:x"),
                Some("data:image/png;base64,AAAA"),
            ),
            r("javascript:x", None, None),
        ]);
        assert_eq!(kept.len(), 2);
        assert_eq!(kept[0].official_link.as_deref(), Some("https://a.b"));
        assert_eq!(kept[1].official_link, None);
        assert_eq!(kept[1].image_url, None);
    }

    #[test]
    fn embiences_need_both_urls_to_be_http() {
        let kept = sanitize_embiences(vec![
            embience(1, "https://x/a.mp3"),
            embience(2, "file:///etc/passwd"),
        ]);
        assert_eq!(kept.iter().map(|e| e.id).collect::<Vec<_>>(), [1]);
    }

    #[test]
    fn presets_keep_only_known_unique_ids_and_drop_empty_ones() {
        let sounds = [embience(6, "https://x/a"), embience(12, "https://x/b")];
        let p = |id, ids: &[u32]| Preset {
            id,
            label: format!("p{id}"),
            embience_ids: ids.to_vec(),
        };
        let kept = sanitize_presets(
            vec![p(1, &[12, 6, 12]), p(2, &[]), p(3, &[99]), p(4, &[99, 6])],
            &sounds,
        );
        let shape: Vec<_> = kept
            .iter()
            .map(|p| (p.id, p.embience_ids.clone()))
            .collect();
        assert_eq!(shape, vec![(1, vec![12, 6]), (4, vec![6])]);
    }

    #[test]
    fn list_bodies_decode_from_the_documented_shape() {
        let list: Items<Generation> = parse_response(
            200,
            r#"{"items":[{"number":5,"label":"Go","link":"/deisktify","kernel":"6","built_on":"2026-08-30"}]}"#,
        )
        .unwrap();
        assert_eq!(list.items[0].number, 5);
        // Bare arrays (the old JSON shape) are not what the API sends.
        assert!(matches!(
            parse_response::<Items<Generation>>(200, "[]"),
            Err(ApiError::Decode(_))
        ));
    }

    #[test]
    fn base_url_accepts_only_plain_http_origins() {
        for (raw, want) in [
            (
                "https://tquilla-api.example.workers.dev",
                "https://tquilla-api.example.workers.dev",
            ),
            (
                "https://tquilla-api.example.workers.dev/",
                "https://tquilla-api.example.workers.dev",
            ),
            ("  http://127.0.0.1:8787 ", "http://127.0.0.1:8787"),
            ("http://localhost:8787", "http://localhost:8787"),
            ("http://[::1]:8787", "http://[::1]:8787"),
        ] {
            assert_eq!(normalize_base(Some(raw)).as_deref(), Ok(want), "{raw:?}");
        }
        for bad in [
            "",
            "   ",
            "example.com",
            "ftp://example.com",
            "https://",
            "https://example.com/api",
            "https://example.com//",
            "https://user:pass@example.com",
            "https://example.com?x=1",
            "https://example.com#frag",
            "https://exa mple.com",
            "javascript:alert(1)",
        ] {
            assert_eq!(
                normalize_base(Some(bad)),
                Err(ApiError::NotConfigured),
                "{bad:?}"
            );
        }
        assert_eq!(normalize_base(None), Err(ApiError::NotConfigured));
    }

    #[test]
    fn join_url_only_allows_versioned_endpoint_paths() {
        let base = "https://api.example";
        assert_eq!(
            join_url(base, api_types::paths::HEALTH).as_deref(),
            Ok("https://api.example/api/v1/health")
        );
        for bad in [
            "",
            "/",
            "/health",
            "/api/v1",
            "/api/v1/",
            "/api/v10/health",
            "/api/v1/../admin",
            "/api/v1/health?x=1",
            "/api/v1/health#x",
            "/api/v1/he alth",
            "/api/v1/health\n",
            "//evil.example/api/v1/health",
            "https://evil.example/api/v1/health",
            "/api/v1\\health",
        ] {
            assert_eq!(join_url(base, bad), Err(ApiError::InvalidPath), "{bad:?}");
        }
    }

    #[test]
    fn every_known_endpoint_path_is_joinable() {
        for p in api_types::paths::ALL {
            assert!(join_url("http://x", p).is_ok(), "{p}");
        }
    }

    #[test]
    fn success_body_is_decoded() {
        let ok: HealthResponse =
            parse_response(200, r#"{"status":"ok","version":"0.1.0","db":true}"#).unwrap();
        assert_eq!(ok.status, ServiceStatus::Ok);
        assert!(ok.db);
    }

    #[test]
    fn degraded_health_is_data_but_other_503s_are_errors() {
        let degraded =
            parse_health(503, r#"{"status":"degraded","version":"0.1.0","db":false}"#).unwrap();
        assert_eq!(degraded.status, ServiceStatus::Degraded);
        assert!(!degraded.db);
        // A 503 from a proxy, or the generic error shape, stays an error.
        for body in [
            "<html>503</html>",
            r#"{"error":{"code":"unavailable","message":"x"}}"#,
        ] {
            let err = parse_health(503, body).unwrap_err();
            assert!(
                matches!(err, ApiError::Status { status: 503, .. }),
                "{body:?}"
            );
            assert!(err.is_retryable());
        }
        // The degraded shape on any other status is not special.
        assert!(parse_health(500, r#"{"status":"degraded","version":"x","db":false}"#).is_err());
    }

    #[test]
    fn documented_error_body_becomes_status_with_detail() {
        let err = parse_response::<HealthResponse>(
            429,
            r#"{"error":{"code":"rate_limited","message":"Too many requests."}}"#,
        )
        .unwrap_err();
        assert_eq!(err.code(), Some(ErrorCode::RateLimited));
        assert!(err.is_retryable());
        assert_eq!(err.to_string(), "Too many requests, try again later.");
    }

    #[test]
    fn undocumented_error_bodies_never_panic_and_keep_the_status() {
        for body in [
            "",
            "<html>502 Bad Gateway</html>",
            "null",
            r#"{"error":"flat string"}"#,
            // Code from a newer server than this client knows.
            r#"{"error":{"code":"teapot","message":"x"}}"#,
        ] {
            let err = parse_response::<HealthResponse>(502, body).unwrap_err();
            assert_eq!(
                err,
                ApiError::Status {
                    status: 502,
                    detail: None
                },
                "{body:?}"
            );
            assert!(err.is_retryable());
        }
    }

    #[test]
    fn garbage_in_a_2xx_response_is_a_decode_error() {
        for body in [
            "",
            "<html>",
            r#"{"status":"ok"}"#,
            r#"{"status":"weird","version":"x","db":true}"#,
        ] {
            let err = parse_response::<HealthResponse>(200, body).unwrap_err();
            assert!(matches!(err, ApiError::Decode(_)), "{body:?}");
            assert!(!err.is_retryable());
        }
    }

    #[test]
    fn client_errors_are_not_retryable_and_messages_leak_nothing() {
        let not_found = ApiError::Status {
            status: 404,
            detail: Some(ApiErrorDetail {
                code: ErrorCode::NotFound,
                message: "secret internal detail".into(),
            }),
        };
        assert!(!not_found.is_retryable());
        assert_eq!(not_found.to_string(), "Not found.");
        let net = ApiError::Network("https://secret.example/api failed: dns error".into());
        assert!(net.is_retryable());
        assert!(!net.to_string().contains("secret"));
        assert!(!ApiError::NotConfigured.is_retryable());
        assert!(!ApiError::BodyTooLarge.is_retryable());
        let decode = ApiError::Decode("expected value at line 1".into());
        assert!(!decode.to_string().contains("line 1"));
    }
}
