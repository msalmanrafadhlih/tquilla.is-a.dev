//! Which image hosts load by themselves, and which ask first (D10).
//!
//! An entry may point at any `https://` image. Loading it makes the visitor's
//! browser contact that host (IP address, browser, time), which is how a
//! tracking pixel works and also how a stranger's server learns who read the
//! guestbook. So only hosts on a short list load straight away; every other
//! image becomes a "Load image from <host>" button and **no request is made
//! until it is clicked**.
//!
//! The list is policy, not code: edit [`TRUSTED_HOSTS`] / [`TRUSTED_SUFFIXES`].
//! Hosts are matched against what a *browser* would connect to, so the parser
//! below is deliberately strict and answers "not trusted" for anything odd
//! (user info, ports, backslashes, control characters, percent-escapes,
//! non-ASCII, IP literals) instead of trying to guess what the browser does.

/// Hosts matched exactly (lowercase, no port).
pub const TRUSTED_HOSTS: &[&str] = &[
    // README badges and the owner's own pictures.
    "img.shields.io",
    "github.com",
    "tquilla.is-a.dev",
];

/// `githubusercontent.com` and every subdomain (`raw.`, `avatars.`,
/// `user-images.`, `camo.`, ...). Matched on a dot boundary, so
/// `evilgithubusercontent.com` and `raw.githubusercontent.com.evil.dev` do not
/// match.
pub const TRUSTED_SUFFIXES: &[&str] = &["githubusercontent.com"];

/// An image this small (either side, in pixels) is an icon or a badge: its
/// placeholder is a small square instead of a full button, so a row of badges
/// does not turn into a wall of buttons.
pub const ICON_MAX_SIDE: u16 = 64;

/// Where a browser would connect for an `https://` URL.
#[derive(Debug, PartialEq, Eq)]
struct Authority {
    /// Lowercase, without a trailing dot, only `a-z 0-9 . -`.
    host: String,
    /// `None` for the default port; anything else is an explicit port.
    port: Option<u16>,
    has_user_info: bool,
}

fn authority(src: &str) -> Option<Authority> {
    // Browsers drop tabs and newlines inside a URL and read `\` as `/`; a
    // check that sees something different from the browser can be fooled
    // (`https://evil.dev\@raw.githubusercontent.com/`). Refuse both.
    if src.chars().any(|c| c.is_control() || c == ' ') {
        return None;
    }
    let rest = src
        .get(..8)
        .filter(|p| p.eq_ignore_ascii_case("https://"))?;
    let rest = &src[rest.len()..];
    let end = rest.find(['/', '?', '#', '\\']).unwrap_or(rest.len());
    let auth = &rest[..end];
    let (has_user_info, hostport) = match auth.rfind('@') {
        Some(at) => (true, &auth[at + 1..]),
        None => (false, auth),
    };
    if hostport.starts_with('[') {
        return None; // IP literal
    }
    let (host, port) = match hostport.split_once(':') {
        Some((h, p)) => {
            let port = if p.is_empty() {
                None
            } else {
                Some(p.parse::<u16>().ok()?)
            };
            (h, port)
        }
        None => (hostport, None),
    };
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    let valid = !host.is_empty()
        && host.len() <= 253
        && host
            .split('.')
            .all(|label| !label.is_empty() && label.len() <= 63)
        && host
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-');
    valid.then_some(Authority {
        host,
        port: port.filter(|p| *p != 443),
        has_user_info,
    })
}

/// The host to show in "Load image from <host>" (lowercase, no user info, no
/// port); `None` if the address cannot be read safely.
pub fn image_host(src: &str) -> Option<String> {
    authority(src).map(|a| a.host)
}

/// Is `host` (already lowercase, as returned by [`image_host`]) on the list?
pub fn is_trusted_host(host: &str) -> bool {
    TRUSTED_HOSTS.contains(&host)
        || TRUSTED_SUFFIXES.iter().any(|suffix| {
            host == *suffix
                || host
                    .strip_suffix(suffix)
                    .is_some_and(|front| front.ends_with('.'))
        })
}

/// Loads without asking: `https`, no user info, default port, trusted host.
pub fn is_trusted_image(src: &str) -> bool {
    authority(src).is_some_and(|a| !a.has_user_info && a.port.is_none() && is_trusted_host(&a.host))
}

/// Icon or badge size, by the `width` / `height` hints on the image.
pub fn is_icon_sized(width: Option<u16>, height: Option<u16>) -> bool {
    width.is_some_and(|w| w <= ICON_MAX_SIDE) || height.is_some_and(|h| h <= ICON_MAX_SIDE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listed_hosts_load_by_themselves() {
        for src in [
            "https://img.shields.io/badge/x-y-green",
            "https://raw.githubusercontent.com/u/r/main/a.png",
            "https://avatars.githubusercontent.com/u/1?v=4",
            "https://user-images.githubusercontent.com/1/a.png",
            "https://githubusercontent.com/a.png",
            "https://github.com/user-attachments/assets/abc",
            "https://tquilla.is-a.dev/a.png",
            // Case, a trailing dot, the default port and fragments do not matter.
            "HTTPS://RAW.GitHubUserContent.COM/a.png",
            "https://raw.githubusercontent.com./a.png",
            "https://raw.githubusercontent.com:443/a.png",
            "https://raw.githubusercontent.com",
            "https://raw.githubusercontent.com#x",
            "https://raw.githubusercontent.com?x=1",
        ] {
            assert!(is_trusted_image(src), "{src}");
        }
    }

    #[test]
    fn every_other_host_has_to_be_asked_for() {
        for src in [
            "https://komarev.com/ghpvc/?username=x",
            "https://i.imgur.com/a.png",
            "https://example.com/a.png",
            "https://shields.io/a.png",
            "https://sub.img.shields.io/a.png",
            "https://www.github.com/a.png",
            "https://evilgithubusercontent.com/a.png",
            "https://githubusercontent.com.evil.dev/a.png",
            "https://raw.githubusercontent.com.evil.dev/a.png",
            "https://img.shields.io.evil.dev/a.png",
            "https://tquilla.is-a.dev.evil.dev/a.png",
            "https://x-githubusercontent.com/a.png",
        ] {
            assert!(!is_trusted_image(src), "{src}");
        }
    }

    #[test]
    fn tricks_that_make_a_check_see_a_different_host_than_the_browser_fail() {
        for src in [
            // User info: the browser connects to what follows `@`.
            "https://raw.githubusercontent.com@evil.dev/a.png",
            "https://raw.githubusercontent.com:x@evil.dev/a.png",
            "https://user@raw.githubusercontent.com/a.png",
            // `\` is `/` to a browser.
            "https://evil.dev\\@raw.githubusercontent.com/a.png",
            "https://evil.dev\\.raw.githubusercontent.com/a.png",
            // Ports other than 443.
            "https://raw.githubusercontent.com:8443/a.png",
            "https://raw.githubusercontent.com:80/a.png",
            // Whitespace and control characters (a browser strips some of them).
            "https://raw.githubusercontent.com\t.evil.dev/a.png",
            "https://raw.github\nusercontent.com/a.png",
            "https://raw.githubusercontent.com /a.png",
            "https://raw.githubusercontent.com\u{0}/a.png",
            // Escapes, non-ASCII, IP literals, empty labels, other schemes.
            "https://raw.githubusercontent%2ecom.evil.dev/a.png",
            "https://raw.githubusercontent.com%00.evil.dev/a.png",
            "https://raw.githubusercontеnt.com/a.png", // Cyrillic е
            "https://[::1]/a.png",
            "https://[raw.githubusercontent.com]/a.png",
            "https://127.0.0.1/a.png",
            "https://raw..githubusercontent.com/a.png",
            "https://.raw.githubusercontent.com/a.png",
            "https:///a.png",
            "https://",
            "http://raw.githubusercontent.com/a.png",
            "//raw.githubusercontent.com/a.png",
            "data:image/png;base64,AAAA",
            "javascript:alert(1)",
            "",
        ] {
            assert!(!is_trusted_image(src), "{src:?}");
        }
    }

    #[test]
    fn a_shown_host_is_the_one_the_browser_would_use() {
        assert_eq!(
            image_host("https://I.Imgur.com/a.png").as_deref(),
            Some("i.imgur.com")
        );
        assert_eq!(
            image_host("https://example.com:8443/x").as_deref(),
            Some("example.com")
        );
        assert_eq!(
            image_host("https://a@example.com/x").as_deref(),
            Some("example.com")
        );
        assert_eq!(
            image_host("https://evil.dev\\@example.com/x").as_deref(),
            Some("evil.dev")
        );
        assert_eq!(
            image_host("https://example.com./x").as_deref(),
            Some("example.com")
        );
        // Internationalised names are shown (and compared) in their `xn--` form
        // only; anything non-ASCII is unreadable here, not guessed at.
        assert_eq!(
            image_host("https://xn--e1afmkfd.xn--p1ai/").as_deref(),
            Some("xn--e1afmkfd.xn--p1ai")
        );
        assert_eq!(image_host("https://пример.рф/"), None);
        assert_eq!(image_host("https://[::1]/"), None);
        assert_eq!(image_host("ftp://example.com/"), None);
        assert_eq!(image_host("https://exa mple.com/"), None);
        assert_eq!(image_host("https://example.com:99999/"), None);
        assert_eq!(
            image_host(&format!("https://{}.com/", "a".repeat(64))),
            None
        );
    }

    #[test]
    fn trusted_host_matching_is_on_label_boundaries() {
        assert!(is_trusted_host("img.shields.io"));
        assert!(is_trusted_host("camo.githubusercontent.com"));
        assert!(is_trusted_host("a.b.githubusercontent.com"));
        assert!(!is_trusted_host("githubusercontent.com.au"));
        assert!(!is_trusted_host("xgithubusercontent.com"));
        assert!(!is_trusted_host(""));
        assert!(!is_trusted_host("."));
    }

    #[test]
    fn small_images_are_icons_big_or_unsized_ones_are_not() {
        assert!(is_icon_sized(Some(20), None));
        assert!(is_icon_sized(None, Some(64)));
        assert!(is_icon_sized(Some(600), Some(30)));
        assert!(!is_icon_sized(Some(65), None));
        assert!(!is_icon_sized(None, None));
        assert!(!is_icon_sized(Some(300), Some(200)));
    }

    #[test]
    fn the_list_itself_is_well_formed() {
        for host in TRUSTED_HOSTS.iter().chain(TRUSTED_SUFFIXES) {
            assert_eq!(&host.to_ascii_lowercase(), host);
            assert!(!host.starts_with('.') && !host.ends_with('.') && !host.contains(':'));
            assert_eq!(
                image_host(&format!("https://{host}/")).as_deref(),
                Some(*host)
            );
        }
    }
}
