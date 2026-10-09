use dioxus::prelude::*;

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
pub const AUDIO_JS: &str = r#"
(function () {
  if (window.__audio) return;
  var HLS_URL = 'https://cdn.jsdelivr.net/npm/hls.js@1/dist/hls.min.js';

  function loadHls() {
    if (window.Hls) return Promise.resolve(window.Hls);
    if (window.__hlsLoading) return window.__hlsLoading;
    window.__hlsLoading = new Promise(function (resolve, reject) {
      var s = document.createElement('script');
      s.src = HLS_URL;
      s.async = true;
      s.onload = function () { resolve(window.Hls); };
      s.onerror = function () {
        window.__hlsLoading = null; // boleh dicoba lagi pada load berikutnya
        s.remove();
        reject(new Error('hls.js failed to load'));
      };
      document.head.appendChild(s);
    });
    return window.__hlsLoading;
  }

  window.__audio = {
    _hls: {},
    _ready: {},
    _token: {},
    load: function (id, url) {
      var el = document.getElementById(id);
      if (!el) return;
      var self = this;
      var token = (self._token[id] = (self._token[id] || 0) + 1);
      if (self._hls[id]) {
        self._hls[id].destroy();
        delete self._hls[id];
      }
      if (url.indexOf('.m3u8') === -1) {
        self._ready[id] = Promise.resolve();
        el.src = url;
        return;
      }
      self._ready[id] = loadHls().then(function (Hls) {
        if (self._token[id] !== token) return; // sudah digantikan load baru
        if (Hls && Hls.isSupported()) {
          var hls = new Hls();
          hls.loadSource(url);
          hls.attachMedia(el);
          self._hls[id] = hls;
        } else {
          el.src = url; // mis. Safari: HLS native
        }
      }).catch(function () {
        if (self._token[id] === token) el.src = url;
      });
    },
    play: function (id) {
      var el = document.getElementById(id);
      if (!el) return;
      (this._ready[id] || Promise.resolve()).then(function () {
        el.play().catch(function () {});
      });
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
