use dioxus::prelude::*;

use crate::desktop::js_util::{eval_js, js_string_escape};

/// Persistent audio-control utility, `eval`'d once (idempotent) from
/// DesktopMode's mount. `.m3u8` sources are routed through hls.js (loaded
/// in index.html) since Chrome/Firefox can't play HLS natively; anything
/// else just becomes a plain `<audio>` src. Used by both the Radio Player
/// (single stream) and Embience (many looping tracks mixed together).
pub const AUDIO_JS: &str = r#"
(function () {
  if (window.__audio) return;
  window.__audio = {
    _hls: {},
    load: function (id, url) {
      var el = document.getElementById(id);
      if (!el) return;
      if (this._hls[id]) {
        this._hls[id].destroy();
        delete this._hls[id];
      }
      if (url.indexOf('.m3u8') !== -1 && window.Hls && window.Hls.isSupported()) {
        var hls = new window.Hls();
        hls.loadSource(url);
        hls.attachMedia(el);
        this._hls[id] = hls;
      } else {
        el.src = url;
      }
    },
    play: function (id) {
      var el = document.getElementById(id);
      if (el) { el.play().catch(function () {}); }
    },
    pause: function (id) {
      var el = document.getElementById(id);
      if (el) el.pause();
    },
    setVolume: function (id, v) {
      var el = document.getElementById(id);
      if (el) el.volume = v;
    }
  };
})();
"#;

pub fn load_and_play(audio_id: &str, url: &str) {
    eval_js(format!(
        "window.__audio && (window.__audio.load('{audio_id}', '{}'), window.__audio.play('{audio_id}'));",
        js_string_escape(url)
    ));
}

pub fn load_set_volume_and_play(audio_id: &str, url: &str, volume: f64) {
    eval_js(format!(
        "window.__audio && (window.__audio.load('{audio_id}', '{}'), window.__audio.setVolume('{audio_id}', {volume}), window.__audio.play('{audio_id}'));",
        js_string_escape(url)
    ));
}

pub fn pause_audio(audio_id: &str) {
    eval_js(format!("window.__audio && window.__audio.pause('{audio_id}');"));
}

pub fn resume_audio(audio_id: &str) {
    eval_js(format!("window.__audio && window.__audio.play('{audio_id}');"));
}

pub fn set_audio_volume(audio_id: &str, volume: f64) {
    eval_js(format!("window.__audio && window.__audio.setVolume('{audio_id}', {volume});"));
}
