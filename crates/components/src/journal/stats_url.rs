//! Validation of the build-time `STATS_API_URL` (pure, unit-tested).
//!
//! The value is baked into the wasm with `option_env!`, so a missing variable
//! must be a *runtime* "not configured" state, never a compile error and never
//! a guessed host (same rule as `API_BASE_URL` in `crate::api`).

/// Returns the URL when it is an absolute `http(s)://host[:port][/path][?query]`
/// with nothing suspicious in it; `None` otherwise (unset, empty, relative,
/// other scheme, credentials, fragment, whitespace/control characters).
pub(super) fn normalize(raw: Option<&str>) -> Option<String> {
    let raw = raw?.trim();
    let rest = raw
        .strip_prefix("https://")
        .or_else(|| raw.strip_prefix("http://"))?;
    let host_end = rest.find(['/', '?']).unwrap_or(rest.len());
    let host = &rest[..host_end];
    let host_ok = !host.is_empty()
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':' | '[' | ']'));
    // Printable ASCII only: no whitespace, control or bidi characters anywhere.
    let clean = raw
        .chars()
        .all(|c| c.is_ascii_graphic() && !matches!(c, '#' | '\\' | '@'));
    (host_ok && clean).then(|| raw.to_owned())
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn accepts_https_and_http_with_path() {
        assert_eq!(
            normalize(Some("https://stats.example.workers.dev/api/stats")).as_deref(),
            Some("https://stats.example.workers.dev/api/stats")
        );
        assert_eq!(
            normalize(Some("  http://localhost:8787/api/stats \n")).as_deref(),
            Some("http://localhost:8787/api/stats")
        );
        assert_eq!(
            normalize(Some("https://a.dev")).as_deref(),
            Some("https://a.dev")
        );
    }

    #[test]
    fn unset_or_empty_is_not_configured() {
        assert_eq!(normalize(None), None);
        assert_eq!(normalize(Some("")), None);
        assert_eq!(normalize(Some("   ")), None);
    }

    #[test]
    fn rejects_relative_and_other_schemes() {
        for bad in [
            "/api/stats",
            "data.json",
            "stats.example.dev/api/stats",
            "ftp://stats.example.dev",
            "javascript:alert(1)",
            "//stats.example.dev/api/stats",
            "https://",
            "https:///api/stats",
            "https://?x=1",
        ] {
            assert_eq!(normalize(Some(bad)), None, "{bad:?}");
        }
    }

    #[test]
    fn rejects_credentials_fragments_and_odd_characters() {
        for bad in [
            "https://user:pw@stats.example.dev/api/stats",
            "https://stats.example.dev/api/stats#x",
            "https://stats.example.dev/api stats",
            "https://stats.example.dev\\evil",
            "https://stats.exa\u{0}mple.dev",
            "https://stats.example.dev\u{202e}/x",
            "https://stats_example.dev/x",
            "https://evil.dev@stats.example.dev/x",
        ] {
            assert_eq!(normalize(Some(bad)), None, "{bad:?}");
        }
    }
}
