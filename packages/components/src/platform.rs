//! Cross-platform shims.
//!
//! The UI in this crate is shared by the `web`, `desktop` and `mobile`
//! packages. Everything that only exists in a browser (`js_sys::Date`,
//! `gloo-timers`, `gloo-net`) is hidden behind the small API below, with a
//! wasm32 implementation (exactly what the app used before) and a native
//! one for the desktop / mobile builds.
//!
//! Platform *behaviour* that differs per target (e.g. how to open an
//! external link) is injected by each platform package through
//! [`PlatformServices`] instead of being hard-coded here.

use dioxus::prelude::*;

// ---------------------------------------------------------------------------
// Services injected by the platform packages
// ---------------------------------------------------------------------------

/// Things only the host platform knows how to do. A platform package
/// provides this once at its root with `use_context_provider`; if it
/// doesn't, [`PlatformServices::default`] is used.
#[derive(Clone, Copy)]
pub struct PlatformServices {
    /// Open a link that leaves the app (http/https/mailto/...).
    /// Internal routes (`/profile`, `/deisktify`) never reach this — they
    /// go through the router.
    pub open_external: fn(&str),
}

impl Default for PlatformServices {
    fn default() -> Self {
        Self {
            open_external: default_open_external,
        }
    }
}

fn default_open_external(url: &str) {
    let url = url.replace('\\', "\\\\").replace('\'', "\\'");
    spawn(async move {
        document::eval(&format!("window.location.href = '{url}';"))
            .await
            .ok();
    });
}

/// The [`PlatformServices`] provided by the platform package (or the default).
pub fn use_platform() -> PlatformServices {
    try_use_context::<PlatformServices>().unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Timers
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
pub use gloo_timers::future::TimeoutFuture;

/// Same call shape as `gloo_timers::future::TimeoutFuture::new(ms).await`.
#[cfg(not(target_arch = "wasm32"))]
pub struct TimeoutFuture;

#[cfg(not(target_arch = "wasm32"))]
impl TimeoutFuture {
    #[allow(clippy::new_ret_no_self, clippy::manual_async_fn)]
    pub fn new(millis: u32) -> impl std::future::Future<Output = ()> {
        async move {
            tokio::time::sleep(std::time::Duration::from_millis(u64::from(millis))).await;
        }
    }
}

// ---------------------------------------------------------------------------
// Date / time
// ---------------------------------------------------------------------------

/// Mirrors the subset of `js_sys::Date` this app uses (local time).
#[cfg(target_arch = "wasm32")]
pub struct Date(js_sys::Date);

#[cfg(target_arch = "wasm32")]
impl Date {
    pub fn new_0() -> Self {
        Self(js_sys::Date::new_0())
    }
    pub fn from_iso(iso: &str) -> Self {
        Self(js_sys::Date::new(&wasm_bindgen::JsValue::from_str(iso)))
    }
    pub fn now() -> f64 {
        js_sys::Date::now()
    }
    pub fn get_time(&self) -> f64 {
        self.0.get_time()
    }
    pub fn to_iso_string(&self) -> String {
        self.0.to_iso_string().as_string().unwrap_or_default()
    }
    pub fn get_hours(&self) -> u32 {
        self.0.get_hours()
    }
    pub fn get_minutes(&self) -> u32 {
        self.0.get_minutes()
    }
    /// 0 = Sunday
    pub fn get_day(&self) -> u32 {
        self.0.get_day()
    }
    /// 0 = January
    pub fn get_month(&self) -> u32 {
        self.0.get_month()
    }
    pub fn get_date(&self) -> u32 {
        self.0.get_date()
    }
    pub fn get_full_year(&self) -> u32 {
        self.0.get_full_year()
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct Date(Option<chrono::DateTime<chrono::Local>>);

#[cfg(not(target_arch = "wasm32"))]
impl Date {
    pub fn new_0() -> Self {
        Self(Some(chrono::Local::now()))
    }
    /// Invalid input gives an "invalid date" (`get_time()` is NaN), like JS.
    pub fn from_iso(iso: &str) -> Self {
        Self(
            chrono::DateTime::parse_from_rfc3339(iso)
                .ok()
                .map(|d| d.with_timezone(&chrono::Local)),
        )
    }
    pub fn now() -> f64 {
        chrono::Utc::now().timestamp_millis() as f64
    }
    pub fn get_time(&self) -> f64 {
        self.0.map_or(f64::NAN, |d| d.timestamp_millis() as f64)
    }
    pub fn to_iso_string(&self) -> String {
        self.0
            .map(|d| {
                d.with_timezone(&chrono::Utc)
                    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
            })
            .unwrap_or_default()
    }
    pub fn get_hours(&self) -> u32 {
        use chrono::Timelike;
        self.0.map_or(0, |d| d.hour())
    }
    pub fn get_minutes(&self) -> u32 {
        use chrono::Timelike;
        self.0.map_or(0, |d| d.minute())
    }
    pub fn get_day(&self) -> u32 {
        use chrono::Datelike;
        self.0.map_or(0, |d| d.weekday().num_days_from_sunday())
    }
    pub fn get_month(&self) -> u32 {
        use chrono::Datelike;
        self.0.map_or(0, |d| d.month0())
    }
    pub fn get_date(&self) -> u32 {
        use chrono::Datelike;
        self.0.map_or(1, |d| d.day())
    }
    pub fn get_full_year(&self) -> u32 {
        use chrono::Datelike;
        self.0.map_or(1970, |d| d.year().max(0) as u32)
    }
}

// ---------------------------------------------------------------------------
// HTTP
// ---------------------------------------------------------------------------

pub mod http {
    //! Minimal HTTP client: `gloo-net` (fetch) on wasm32, `reqwest` natively.

    pub struct Response {
        pub status: u16,
        pub body: String,
    }

    impl Response {
        pub fn ok(&self) -> bool {
            (200..300).contains(&self.status)
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn get(url: &str) -> Result<Response, String> {
        let resp = gloo_net::http::Request::get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        Ok(Response { status, body })
    }

    pub async fn post_json(url: &str, body: String) -> Result<Response, String> {
        post_json_with_headers(url, body, &[]).await
    }

    /// Same as [`post_json`] plus extra request headers (e.g. an API key
    /// header, so secrets never have to travel in the URL / query string).
    #[cfg(target_arch = "wasm32")]
    pub async fn post_json_with_headers(
        url: &str,
        body: String,
        headers: &[(&str, &str)],
    ) -> Result<Response, String> {
        let mut builder =
            gloo_net::http::Request::post(url).header("Content-Type", "application/json");
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        let request = builder.body(body).map_err(|e| e.to_string())?;
        let resp = request.send().await.map_err(|e| e.to_string())?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        Ok(Response { status, body })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn get(url: &str) -> Result<Response, String> {
        let resp = reqwest::get(url).await.map_err(|e| e.to_string())?;
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        Ok(Response { status, body })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn post_json_with_headers(
        url: &str,
        body: String,
        headers: &[(&str, &str)],
    ) -> Result<Response, String> {
        let mut builder = reqwest::Client::new()
            .post(url)
            .header("Content-Type", "application/json");
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        let resp = builder.body(body).send().await.map_err(|e| e.to_string())?;
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        Ok(Response { status, body })
    }
}
