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

pub use api_types::{
    ApiErrorBody, ApiErrorDetail, ErrorCode, HealthResponse, ServiceStatus, API_PREFIX,
};

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

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotConfigured => f.write_str("The server address is not configured."),
            Self::InvalidPath => f.write_str("Invalid request."),
            Self::BodyTooLarge => f.write_str("The request is too large."),
            Self::Network(_) => f.write_str("Could not reach the server."),
            Self::Status { status, detail } => match detail.as_ref().map(|d| d.code) {
                Some(ErrorCode::RateLimited) => f.write_str("Too many requests, try again later."),
                Some(ErrorCode::PayloadTooLarge) => f.write_str("The request is too large."),
                Some(ErrorCode::NotFound) => f.write_str("Not found."),
                _ if *status >= 500 => f.write_str("The server had a problem, try again later."),
                _ => f.write_str("The server rejected the request."),
            },
            Self::Decode(_) => f.write_str("The server sent an unexpected response."),
        }
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
    let url = join_url(base_url()?, path)?;
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

#[cfg(test)]
mod tests {
    use super::*;

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
