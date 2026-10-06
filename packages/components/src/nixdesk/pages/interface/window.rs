use dioxus::prelude::*;

/// Which app a window instance represents — also the stable identity used
/// by the open-window registry (open/close/focus/minimize/maximize).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AppId {
    Calculator,
    About,
    Settings,
    FileManager,
    Radio,
    Browser,
    Embience,
    LiveChat,
    AiAssistant,
}

impl AppId {
    /// (x, y, width, height) starting geometry
    pub fn default_geometry(self) -> (f64, f64, f64, f64) {
        match self {
            AppId::Calculator => (640.0, 240.0, 325.0, 452.0),
            AppId::About => (745.0, 12.0, 335.0, 435.0),
            AppId::Settings => (590.0, 12.0, 660.0, 550.0),
            AppId::FileManager => (0.0, 12.0, 905.0, 565.0),
            AppId::Radio => (114.0, 45.0, 425.0, 755.0),
            AppId::Browser => (0.0, 12.0, 1250.0, 820.0),
            AppId::Embience => (1035.0, 12.0, 540.0, 610.0),
            AppId::LiveChat => (1315.0, 12.0, 505.0, 820.0),
            AppId::AiAssistant => (0.0, 12.0, 505.0, 820.0),
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            AppId::Calculator => "Calculator",
            AppId::About => "About",
            AppId::Settings => "Settings",
            AppId::FileManager => "File Manager",
            AppId::Radio => "Radio Indonesia",
            AppId::Browser => "Locked Browser",
            AppId::Embience => "Embience",
            AppId::LiveChat => "Live Chat!",
            AppId::AiAssistant => "Ai Asistant",
        }
    }

    /// Icons via Dioxus's built-in `asset!` macro: paths are checked at
    /// compile time (a typo/missing file is now a compile error, not a
    /// broken `<img>` at runtime) and resolve correctly regardless of
    /// where the built app is served from.
    pub fn window_icon(self) -> Asset {
        match self {
            AppId::Calculator => asset!("/assets/logo-calculator.svg"),
            AppId::About => asset!("/assets/Icon-nixos.svg"),
            AppId::Settings => asset!("/assets/logo-Settings.svg"),
            AppId::FileManager => asset!("/assets/logo-FileManager.svg"),
            AppId::Radio => asset!("/assets/logo-radio.svg"),
            AppId::Browser => asset!("/assets/logo-Browser.svg"),
            AppId::Embience => asset!("/assets/logo-Embience.svg"),
            AppId::LiveChat => asset!("/assets/logo-livechat.svg"),
            AppId::AiAssistant => asset!("/assets/logo-Ai-Assistent.svg"),
        }
    }

    /// Stable lowercase key — used for DOM ids and rsx `key`s, kept
    /// separate from `title()` so renaming a window's display text never
    /// touches its DOM identity.
    pub fn key(self) -> &'static str {
        match self {
            AppId::Calculator => "calculator",
            AppId::About => "about",
            AppId::Settings => "settings",
            AppId::FileManager => "file-manager",
            AppId::Radio => "radio",
            AppId::Browser => "browser",
            AppId::Embience => "embience",
            AppId::LiveChat => "live-chat",
            AppId::AiAssistant => "ai-assistant",
        }
    }
}

/// A window's full state, including its *live* geometry. `x/y/w/h` start
/// from `AppId::default_geometry()` when the window opens, then get
/// overwritten by `update_geometry` whenever a drag or resize ends — they
/// are the single source of truth `WindowFrame` renders from, never
/// recomputed from the default.
///
/// `minimizing` / `closing` are the transient "animation is playing"
/// phases. The final state (`minimized = true`, or removal from the list)
/// is only applied once the CSS animation reports `animationend`.
#[derive(Clone, PartialEq)]
pub struct OpenWindow {
    pub id: AppId,
    pub z: i32,
    pub minimized: bool,
    pub maximized: bool,
    pub minimizing: bool,
    pub closing: bool,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// Open a window if it isn't already, or bring it to front (and restore
/// it from minimized) if it is. Used by the dock's launcher icons.
///
/// Also cancels a minimize/close animation that is still in flight: the
/// animation class disappears, so the open animation plays instead.
pub fn open_or_focus(mut open_windows: Signal<Vec<OpenWindow>>, mut next_z: Signal<i32>, id: AppId) {
    let mut list = open_windows();
    let z = next_z();
    if let Some(w) = list.iter_mut().find(|w| w.id == id) {
        w.z = z;
        w.minimized = false;
        w.minimizing = false;
        w.closing = false;
    } else {
        let (x, y, w, h) = id.default_geometry();
        list.push(OpenWindow {
            id,
            z,
            minimized: false,
            maximized: false,
            minimizing: false,
            closing: false,
            x,
            y,
            w,
            h,
        });
    }
    next_z.set(z + 1);
    open_windows.set(list);
}

/// Klik pada ikon dock:
/// - belum terbuka                    → buka
/// - minimized / sedang animasi tutup → pulihkan (open_or_focus)
/// - terbuka tapi tertutup window lain → bawa ke depan
/// - terbuka dan paling depan          → minimize
pub fn toggle_window(open_windows: Signal<Vec<OpenWindow>>, next_z: Signal<i32>, id: AppId) {
    let list = open_windows();

    let Some(win) = list.iter().find(|w| w.id == id) else {
        open_or_focus(open_windows, next_z, id);
        return;
    };

    if win.minimized || win.minimizing || win.closing {
        open_or_focus(open_windows, next_z, id);
        return;
    }

    let top_z = list
        .iter()
        .filter(|w| !w.minimized)
        .map(|w| w.z)
        .max()
        .unwrap_or(0);

    if win.z == top_z {
        request_minimize(open_windows, id);
    } else {
        open_or_focus(open_windows, next_z, id);
    }
}

/// Bring an already-open window to the front — used when clicking
/// anywhere on a window that isn't already topmost. Skips the write (and
/// therefore the re-render) when the window is already frontmost, so
/// ordinary clicks inside an already-focused window — including the
/// mousedown that starts a drag — don't trigger a needless re-render.
pub fn focus_window(mut open_windows: Signal<Vec<OpenWindow>>, mut next_z: Signal<i32>, id: AppId) {
    let mut list = open_windows();
    let top_z = list.iter().map(|w| w.z).max().unwrap_or(0);
    if list.iter().any(|w| w.id == id && w.z == top_z) {
        return;
    }
    let z = next_z();
    if let Some(w) = list.iter_mut().find(|w| w.id == id) {
        w.z = z;
        next_z.set(z + 1);
        open_windows.set(list);
    }
}

/// Close, phase 1: start the close animation. The window is only removed
/// from the list (phase 2, `close_window`) once the animation ends.
pub fn request_close(mut open_windows: Signal<Vec<OpenWindow>>, id: AppId) {
    let mut list = open_windows();
    // A minimized window is `display: none`, so `animationend` would never
    // fire for it — remove it immediately instead of getting stuck.
    let hidden = list.iter().any(|w| w.id == id && w.minimized);
    if hidden {
        list.retain(|w| w.id != id);
    } else if let Some(w) = list.iter_mut().find(|w| w.id == id) {
        w.closing = true;
        w.minimizing = false;
    }
    open_windows.set(list);
}

/// Close, phase 2: actually remove the window.
pub fn close_window(mut open_windows: Signal<Vec<OpenWindow>>, id: AppId) {
    let mut list = open_windows();
    list.retain(|w| w.id != id);
    open_windows.set(list);
}

/// Minimize, phase 1: start the minimize animation.
pub fn request_minimize(mut open_windows: Signal<Vec<OpenWindow>>, id: AppId) {
    let mut list = open_windows();
    if let Some(w) = list.iter_mut().find(|w| w.id == id) {
        if w.minimized || w.closing {
            return;
        }
        w.minimizing = true;
    }
    open_windows.set(list);
}

/// Minimize, phase 2: actually hide the window (called on `animationend`).
pub fn minimize_window(mut open_windows: Signal<Vec<OpenWindow>>, id: AppId) {
    let mut list = open_windows();
    if let Some(w) = list.iter_mut().find(|w| w.id == id) {
        w.minimized = true;
        w.minimizing = false;
    }
    open_windows.set(list);
}

/// Maximize is purely a CSS-level state (see `WINDOW_FRAME_CSS`'s
/// `.win-maximized` rule) — it never touches the stored `x/y/w/h`, so
/// un-maximizing naturally restores the exact spot and size the window
/// was left at. No special-case restore logic needed here anymore.
pub fn toggle_maximize(mut open_windows: Signal<Vec<OpenWindow>>, id: AppId) {
    let mut list = open_windows();
    if let Some(w) = list.iter_mut().find(|w| w.id == id) {
        w.maximized = !w.maximized;
    }
    open_windows.set(list);
}

/// Syncs the DOM's authoritative post-drag/post-resize geometry back into
/// state. This is what actually fixes the reset/flicker bug: without it,
/// `WindowFrame` had nothing but `AppId::default_geometry()` to render on
/// every re-render, so any re-render (e.g. from a focus change) snapped
/// the window straight back to its default spot and size.
pub fn update_geometry(mut open_windows: Signal<Vec<OpenWindow>>, id: AppId, x: f64, y: f64, w: f64, h: f64) {
    let mut list = open_windows();
    if let Some(win) = list.iter_mut().find(|win| win.id == id) {
        win.x = x;
        win.y = y;
        win.w = w;
        win.h = h;
        open_windows.set(list);
    }
}

/// Small persistent drag/resize utility, `eval`'d once (idempotent) so it
/// survives Dioxus re-renders as real global browser state. Continuous
/// pointer tracking is handled entirely in JS — routing every pixel of
/// a drag back through the WASM boundary would be both slower and riskier
/// to get right without a local compiler to check against.
///
/// - Position/size are written to the `--win-x/-y/-w/-h` CSS custom
///   properties, so `WINDOW_FRAME_CSS`'s media query decides whether
///   those numbers apply at all (unused below the desktop breakpoint,
///   where the window is fullscreen).
/// - `onEnd(x, y, w, h)` fires once per gesture, reporting the window's
///   final box so the caller can sync it back into Dioxus state.
/// - While a gesture is active the window carries a `data-gesture`
///   attribute, which switches the CSS position/size transitions off so
///   the window tracks the cursor 1:1. It's a `data-*` attribute rather
///   than a class because Dioxus rewrites the whole `class` attribute on
///   re-render and would wipe a JS-added class.
pub const WINDOW_MANAGER_JS: &str = r#"
(function () {
  if (window.__wm) return;
  // Must match WINDOW_FRAME_CSS's `@media (min-width: 768px)` — update
  // both together if the project's Tailwind `md:` breakpoint changes.
  var DESKTOP_BREAKPOINT = 768;

  // Listener `document` dipasang SEKALI di sini (blok ini dijaga oleh
  // `if (window.__wm) return` di atas), bukan per window. Sebelumnya tiap
  // `makeDraggable` menambah 5 listener `document` yang tak pernah dilepas,
  // jadi buka-tutup window berulang kali menumpuk handler dan menahan
  // elemen DOM lama di memori. Sekarang hanya satu gesture yang aktif pada
  // satu waktu (`activeDrag`), dan listener meneruskan event ke gesture itu.
  var activeDrag = null; // { move(x, y), stop() } milik window yang sedang di-drag
  document.addEventListener('mousemove', function (e) {
    if (activeDrag) activeDrag.move(e.clientX, e.clientY);
  });
  document.addEventListener('mouseup', function () {
    if (activeDrag) activeDrag.stop();
  });
  document.addEventListener('touchmove', function (e) {
    if (!activeDrag) return;
    var t = e.touches[0];
    if (!t) return;
    activeDrag.move(t.clientX, t.clientY);
    e.preventDefault(); // stop the page from scrolling while dragging
  }, { passive: false });
  document.addEventListener('touchend', function () {
    if (activeDrag) activeDrag.stop();
  });
  document.addEventListener('touchcancel', function () {
    if (activeDrag) activeDrag.stop();
  });

  function makeDraggable(handleId, windowId, onEnd) {
    var handle = document.getElementById(handleId);
    var win = document.getElementById(windowId);
    if (!handle || !win) return;
    var sx = 0, sy = 0, sl = 0, st = 0, dragging = false;
    var gesture = { move: move, stop: stop };
    function start(x, y, target) {
      if (window.innerWidth < DESKTOP_BREAKPOINT) return;
      if (win.classList.contains('win-maximized')) return;
      if (target && target.closest && target.closest('[data-no-drag]')) return;
      dragging = true;
      activeDrag = gesture;
      win.setAttribute('data-gesture', '');
      sx = x; sy = y;
      var rect = win.getBoundingClientRect();
      var parent = win.offsetParent ? win.offsetParent.getBoundingClientRect() : { left: 0, top: 0 };
      sl = rect.left - parent.left;
      st = rect.top - parent.top;
    }
    function move(x, y) {
      if (!dragging) return;
      var nl = sl + (x - sx);
      var nt = Math.max(0, st + (y - sy));
      win.style.setProperty('--win-x', nl + 'px');
      win.style.setProperty('--win-y', nt + 'px');
    }
    function stop() {
      if (!dragging) return;
      dragging = false;
      if (activeDrag === gesture) activeDrag = null;
      if (onEnd) onEnd(win.offsetLeft, win.offsetTop, win.offsetWidth, win.offsetHeight);
      win.removeAttribute('data-gesture');
    }
    // Mouse — hanya listener di handle (ikut hilang bersama elemennya).
    handle.addEventListener('mousedown', function (e) {
      start(e.clientX, e.clientY, e.target);
      if (dragging) e.preventDefault();
    });
    // Touch — same start/move/stop, driven by the first touch point.
    handle.addEventListener('touchstart', function (e) {
      var t = e.touches[0];
      start(t.clientX, t.clientY, e.target);
    }, { passive: true });
  }

  function makeResizable(windowId, onEnd) {
    var win = document.getElementById(windowId);
    if (!win) return;
    var MIN_W = 260, MIN_H = 180;
    var MARGIN = 12; // harus sama dengan margin 12px di WINDOW_FRAME_CSS

    function clamp(v, lo, hi) { return Math.max(lo, Math.min(v, hi)); }

    win.addEventListener('pointerdown', function (e) {
      var handle = e.target.closest && e.target.closest('[data-resize]');
      if (!handle || !win.contains(handle)) return;
      if (e.button !== 0) return;
      if (window.innerWidth < DESKTOP_BREAKPOINT) return;
      if (win.classList.contains('win-maximized')) return;
      var parent = win.offsetParent;
      if (!parent) return;

      var dir = handle.getAttribute('data-resize'); // n s e w ne nw se sw
      var pr = parent.getBoundingClientRect();
      var rc = win.getBoundingClientRect();
      var PW = parent.clientWidth, PH = parent.clientHeight;

      // Bekerja dengan 4 tepi (bukan x/y/w/h) relatif ke parent.
      var L = rc.left - pr.left - parent.clientLeft;
      var T = rc.top - pr.top - parent.clientTop;
      var R = L + rc.width;
      var B = T + rc.height;
      var sx = e.clientX, sy = e.clientY, moved = false;

      handle.setPointerCapture(e.pointerId);
      win.setAttribute('data-gesture', '');
      document.documentElement.style.userSelect = 'none';

      function move(ev) {
        moved = true;
        var dx = ev.clientX - sx, dy = ev.clientY - sy;
        var l = L, t = T, r = R, b = B;

        // Hanya tepi yang ditarik yang bergerak; tepi lawannya tetap.
        if (dir.indexOf('e') !== -1) r = clamp(R + dx, L + MIN_W, PW - MARGIN);
        if (dir.indexOf('w') !== -1) l = clamp(L + dx, MARGIN, R - MIN_W);
        if (dir.indexOf('s') !== -1) b = clamp(B + dy, T + MIN_H, PH - MARGIN);
        if (dir.indexOf('n') !== -1) t = clamp(T + dy, MARGIN, B - MIN_H);

        win.style.setProperty('--win-x', l + 'px');
        win.style.setProperty('--win-y', t + 'px');
        win.style.setProperty('--win-w', (r - l) + 'px');
        win.style.setProperty('--win-h', (b - t) + 'px');
      }

      function stop() {
        handle.removeEventListener('pointermove', move);
        handle.removeEventListener('pointerup', stop);
        handle.removeEventListener('pointercancel', stop);
        document.documentElement.style.userSelect = '';
        if (moved && onEnd) onEnd(win.offsetLeft, win.offsetTop, win.offsetWidth, win.offsetHeight);
        win.removeAttribute('data-gesture');
      }

      handle.addEventListener('pointermove', move);
      handle.addEventListener('pointerup', stop);
      handle.addEventListener('pointercancel', stop);
    });
  }

  window.__wm = { makeDraggable: makeDraggable, makeResizable: makeResizable };
})();
"#;

/// Responsive chrome + animations for every `WindowFrame`.
///
/// Mobile-first: below the `md` breakpoint a window is always fullscreen
/// (there's nothing to drag/resize/position on a phone-sized screen). At
/// `md` (768px) and up it becomes a positioned, resizable popup driven
/// entirely by the `--win-x/-y/-w/-h` custom properties `WindowFrame`
/// sets inline from `OpenWindow`'s state — CSS decides how those numbers
/// get used, Rust only ever owns the numbers themselves.
///
/// Animations:
/// - open / restore-from-minimize: `win-open` (plays whenever the element
///   appears or stops being `display: none`)
/// - minimize: `win-minimizing` → `win-minimize`, then Rust hides it
/// - close: `win-closing` → `win-close`, then Rust removes it
/// - maximize / restore: `transition` on left/top/width/height
/// - drag / resize: no transition (`[data-gesture]`), follows the cursor
///
/// Rendered once (in `DesktopMode`), not once per window.
pub const WINDOW_FRAME_CSS: &str = r#"
.win-frame {
  box-sizing: border-box;
  position: absolute;
  inset: 0;
  /* `backwards`, not `both`: once finished, transform goes back to
     `none`, so no stacking context / blurry text is left behind. */
  animation: win-open 200ms cubic-bezier(0.16, 1, 0.3, 1) backwards;
}

/* Minimize: shrink toward the bottom-center (where the dock is). */
.win-frame.win-minimizing {
  transform-origin: 50% 100%;
  animation: win-minimize 200ms cubic-bezier(0.4, 0, 1, 1) forwards;
  pointer-events: none;
}

/* Declared after `.win-minimizing` so close wins if both ever apply. */
.win-frame.win-closing {
  animation: win-close 160ms cubic-bezier(0.4, 0, 1, 1) forwards;
  pointer-events: none;
}

@keyframes win-open {
  from { opacity: 0; transform: translateY(8px) scale(0.94); }
  to   { opacity: 1; transform: none; }
}
@keyframes win-minimize {
  from { opacity: 1; transform: none; }
  to   { opacity: 0; transform: translateY(80px) scale(0.5); }
}
@keyframes win-close {
  from { opacity: 1; transform: none; }
  to   { opacity: 0; transform: scale(0.92); }
}

@media (min-width: 768px) {
  .win-frame {
    position: absolute;
    inset: auto;
    /* Width/height are capped to the parent first, then left/top are
       clamped against that *capped* size — so a window stays fully
       reachable even when its saved geometry no longer fits (e.g. the
       parent got narrower since it was last positioned). */
    width: min(var(--win-w, 480px), calc(100% - 24px));
    height: min(var(--win-h, 360px), calc(100% - 24px));
    left: clamp(12px, var(--win-x, 24px), calc(100% - min(var(--win-w, 480px), calc(100% - 24px)) - 12px));
    top: clamp(12px, var(--win-y, 24px), calc(100% - min(var(--win-h, 360px), calc(100% - 24px)) - 12px));
    transition:
      left 220ms cubic-bezier(0.2, 0.8, 0.2, 1),
      top 220ms cubic-bezier(0.2, 0.8, 0.2, 1),
      width 220ms cubic-bezier(0.2, 0.8, 0.2, 1),
      height 220ms cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  /* During drag/resize: follow the cursor instantly. */
  .win-frame[data-gesture] {
    transition: none;
  }
  /* Explicit values instead of `inset` + `auto`: `auto` can't be
     transitioned. */
  .win-frame.win-maximized {
    left: 12px;
    top: 12px;
    width: calc(100% - 24px);
    height: calc(100% - 24px);
  }
}

/* 1ms instead of `none`, so `animationend` still fires and the
   minimize/close phases can complete. */
@media (prefers-reduced-motion: reduce) {
  .win-frame,
  .win-frame.win-minimizing,
  .win-frame.win-closing {
    animation-duration: 1ms !important;
    transition-duration: 1ms !important;
  }
}
"#;

/// JS run once per opened window (from the header's `onmounted`): wires
/// its drag handle + resize handles into the shared `window.__wm` helpers
/// from `WINDOW_MANAGER_JS`, reporting the live post-gesture box back to
/// Rust via `dioxus.send`.
fn geometry_sync_js(handle_id: &str, win_id: &str) -> String {
    format!(
        r#"(function() {{
            function report(x, y, w, h) {{ dioxus.send([x, y, w, h]); }}
            window.__wm && window.__wm.makeDraggable('{handle_id}', '{win_id}', report);
            window.__wm && window.__wm.makeResizable('{win_id}', report);
        }})();"#
    )
}

/// (direction, position + cursor). Edge handles leave 12px at each end so
/// they don't overlap the corners; everything sits inside the frame
/// because of its `overflow-hidden`. Class strings are written literally
/// so Tailwind's scanner picks them up.
const RESIZE_HANDLES: [(&str, &str); 8] = [
    ("n",  "top-0 left-3 right-3 h-1.5 cursor-ns-resize"),
    ("s",  "bottom-0 left-3 right-3 h-1.5 cursor-ns-resize"),
    ("w",  "left-0 top-3 bottom-3 w-1.5 cursor-ew-resize"),
    ("e",  "right-0 top-3 bottom-3 w-1.5 cursor-ew-resize"),
    ("nw", "top-0 left-0 w-3 h-3 cursor-nwse-resize"),
    ("ne", "top-0 right-0 w-3 h-3 cursor-nesw-resize"),
    ("sw", "bottom-0 left-0 w-3 h-3 cursor-nesw-resize"),
    ("se", "bottom-0 right-0 w-3 h-3 cursor-nwse-resize"),
];

/// Generic chrome around every app window: title bar (icon, drag handle,
/// traffic-light buttons) + a scrollable content area for whatever
/// `children` the caller renders + eight resize handles.
#[component]
pub fn WindowFrame(
    window: OpenWindow,
    open_windows: Signal<Vec<OpenWindow>>,
    next_z: Signal<i32>,
    children: Element,
) -> Element {
    let id = window.id;
    let dom_id = format!("win-{}", id.key());
    let handle_id = format!("win-{}-handle", id.key());
    let icon = id.window_icon();
    let closing = window.closing;
    let minimizing = window.minimizing;

    // Rust only ever writes these four numbers + z-index — everything
    // about *how* they translate to layout (fullscreen vs. popup,
    // clamping) lives in WINDOW_FRAME_CSS.
    let vars = format!(
        "--win-x:{x}px;--win-y:{y}px;--win-w:{w}px;--win-h:{h}px;z-index:{z};",
        x = window.x, y = window.y, w = window.w, h = window.h, z = window.z,
    );

    let frame_class = format!(
        "parent win-frame md:absolute flex flex-col items-start gap-5 bg-[var(--bg-secondary)] p-5 md:pt-0 overflow-hidden md:border rounded-[10px] border-solid border-[var(--fg-main)]{maximized}{minimizing_cls}{closing_cls}{minimized}",
        maximized = if window.maximized { " win-maximized" } else { "" },
        minimizing_cls = if window.minimizing { " win-minimizing" } else { "" },
        closing_cls = if window.closing { " win-closing" } else { "" },
        minimized = if window.minimized { " hidden" } else { "" },
    );

    // Resize handles are a desktop-popup affordance only — hidden on
    // mobile (fullscreen) and force-hidden while maximized (`!hidden`
    // beats `md:block`'s higher-specificity breakpoint rule).
    let resize_base = format!(
        "absolute touch-none hidden md:block{}",
        if window.maximized { " !hidden" } else { "" },
    );

    // Cloned once, right before `rsx!`: the `onmounted` closure takes
    // ownership of these, while the rsx interpolations above still need
    // the originals.
    let handle_mount_id = handle_id.clone();
    let win_mount_id = dom_id.clone();

    rsx! {
        div {
            id: "{dom_id}",
            class: "{frame_class}",
            style: "{vars}",
            onmousedown: move |_| focus_window(open_windows, next_z, id),
            onanimationend: move |e| {
                // `animationend` bubbles up from children too (spinners
                // etc.), so only react to our own animations by name.
                let name = e.data().animation_name();
                if closing && name == "win-close" {
                    close_window(open_windows, id);
                } else if minimizing && name == "win-minimize" {
                    minimize_window(open_windows, id);
                }
            },

            // TOP BAR
            header {
                id: "{handle_id}",
                class: "flex items-center justify-center md:pt-5 gap-2.5 relative self-stretch w-full flex-[0_0_auto] cursor-move select-none shrink-0",
                onmounted: move |_| {
                    let script = geometry_sync_js(&handle_mount_id, &win_mount_id);
                    spawn(async move {
                        let mut geo_eval = document::eval(&script);
                        loop {
                            match geo_eval.recv::<(f64, f64, f64, f64)>().await {
                                Ok((gx, gy, gw, gh)) => update_geometry(open_windows, id, gx, gy, gw, gh),
                                Err(_) => break,
                            }
                        }
                    });
                },

                // window
                div { class: "flex items-center gap-2.5 relative flex-1 grow",
                    img { class: "relative w-5 h-5 aspect-[1]",
                        src: "{icon}",
                        alt: "{id.title()}",
                        "aria-hidden": "true",
                    }
                    h1 { class: "relative flex items-center justify-center w-max [font:'JetBrains_Mono-ExtraLight',Helvetica] font-extralight text-fg-main text-[12px] text-start tracking-[0] leading-[normal] whitespace-nowrap",
                        "{id.title()}"
                    }
                }

                div {
                    "data-no-drag": "true",
                    class: "inline-flex items-center justify-end gap-2.5 relative flex-[0_0_auto] grow",
                    button {
                        r#type: "button",
                        title: "Minimize",
                        class: "relative w-3 h-3 bg-yellow-500 rounded-[6.5px] aspect-[1] border-0 p-0 opacity-30 [.parent:hover_&]:opacity-70 [.parent:hover_&]:hover:opacity-100 transition-opacity duration-300",
                        onclick: move |_| request_minimize(open_windows, id),
                    }
                    button {
                        r#type: "button",
                        title: "Maximize",
                        class: "hidden md:block relative w-3 h-3 bg-green-500 rounded-[6.5px] aspect-[1] border-0 p-0 opacity-30 [.parent:hover_&]:opacity-70 [.parent:hover_&]:hover:opacity-100 transition-opacity duration-300",
                        onclick: move |_| toggle_maximize(open_windows, id),
                    }
                    button {
                        r#type: "button",
                        title: "Close",
                        class: "relative w-3 h-3 bg-red-500 rounded-[6.5px] aspect-[1] border-0 p-0 opacity-30 [.parent:hover_&]:opacity-70 [.parent:hover_&]:hover:opacity-100 transition-opacity duration-300",
                        onclick: move |_| request_close(open_windows, id),
                    }
                }
            }

            // MAIN CONTENT
            div { class: "@container flex-1 min-h-0 w-full overflow-auto", {children} }

            // Always mounted (so the pointer listener bound in JS only
            // ever attaches once) — visibility is handled purely by CSS.
            for (dir, pos) in RESIZE_HANDLES.iter() {
                div {
                    key: "{dir}",
                    "data-resize": "{dir}",
                    class: "{resize_base} {pos}",
                }
            }
        }
    }
}
