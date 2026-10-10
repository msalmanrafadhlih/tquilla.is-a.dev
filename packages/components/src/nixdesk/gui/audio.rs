use crate::nixdesk::js_util::{eval_js, js_string_escape};

/// Persistent audio-control utility, `eval`'d once (idempotent) from
/// DesktopMode's mount. `.m3u8` sources are routed through hls.js since
/// Chrome/Firefox can't play HLS natively; anything else just becomes a
/// plain `<audio>` src. Used by both the Radio Player (single stream) and
/// Embience (many looping tracks mixed together).
///
/// hls.js is NOT loaded with the page any more: it is injected the first
/// time an `.m3u8` stream is actually loaded (most visitors never open the
/// radio). Because that load is asynchronous, `load` records a per-element
/// "ready" promise and `play` waits for it, so `load(...)` followed
/// immediately by `play(...)` (how Rust calls it) still works. A token per
/// element drops stale results when a new `load` supersedes an older one
/// before hls.js has arrived.
pub const AUDIO_JS: &str = include_str!("../../../js/audio.js");

pub fn load_and_play(audio_id: &str, url: &str) {
    eval_js(format!(
        "window.__audio && (window.__audio.load('{audio_id}', '{}'), window.__audio.play('{audio_id}'));",
        js_string_escape(url)
    ));
}

#[allow(dead_code)]
pub fn load_set_volume_and_play(audio_id: &str, url: &str, volume: f64) {
    eval_js(format!(
        "window.__audio && (window.__audio.load('{audio_id}', '{}'), window.__audio.setVolume('{audio_id}', {volume}), window.__audio.play('{audio_id}'));",
        js_string_escape(url)
    ));
}

pub fn pause_audio(audio_id: &str) {
    eval_js(format!(
        "window.__audio && window.__audio.pause('{audio_id}');"
    ));
}

pub fn resume_audio(audio_id: &str) {
    eval_js(format!(
        "window.__audio && window.__audio.play('{audio_id}');"
    ));
}

pub fn set_audio_volume(audio_id: &str, volume: f64) {
    eval_js(format!(
        "window.__audio && window.__audio.setVolume('{audio_id}', {volume});"
    ));
}
