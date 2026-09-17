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

    /// (x, y, width, height) starting geometry — roughly matching the
    /// Figma layout, staggered per app so freshly opened windows don't
    /// all stack in the exact same spot.
    pub fn default_geometry(self) -> (f64, f64, f64, f64) {
        match self {
            AppId::Calculator => (640.0, 240.0, 300.0, 440.0),
            AppId::About => (520.0, 160.0, 380.0, 420.0),
            AppId::Settings => (460.0, 130.0, 560.0, 440.0),
            AppId::FileManager => (340.0, 170.0, 620.0, 420.0),
            AppId::Radio => (300.0, 210.0, 560.0, 360.0),
            AppId::Browser => (240.0, 110.0, 700.0, 480.0),
            AppId::Embience => (360.0, 150.0, 520.0, 420.0),
            AppId::LiveChat => (700.0, 130.0, 380.0, 520.0),
            AppId::AiAssistant => (220.0, 90.0, 520.0, 560.0),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct OpenWindow {
    pub id: AppId,
    pub z: i32,
    pub minimized: bool,
    pub maximized: bool,
}

/// Open a window if it isn't already, or bring it to front (and restore
/// it from minimized) if it is. Used by the dock's launcher icons.
pub fn open_or_focus(mut open_windows: Signal<Vec<OpenWindow>>, mut next_z: Signal<i32>, id: AppId) {
    let mut list = open_windows();
    let z = next_z();
    if let Some(w) = list.iter_mut().find(|w| w.id == id) {
        w.z = z;
        w.minimized = false;
    } else {
        list.push(OpenWindow { id, z, minimized: false, maximized: false });
    }
    next_z.set(z + 1);
    open_windows.set(list);
}

/// Bring an already-open window to the front — used when clicking
/// anywhere on a window that isn't already topmost.
pub fn focus_window(mut open_windows: Signal<Vec<OpenWindow>>, mut next_z: Signal<i32>, id: AppId) {
    let mut list = open_windows();
    let z = next_z();
    if let Some(w) = list.iter_mut().find(|w| w.id == id) {
        w.z = z;
        next_z.set(z + 1);
        open_windows.set(list);
    }
}

pub fn close_window(mut open_windows: Signal<Vec<OpenWindow>>, id: AppId) {
    let mut list = open_windows();
    list.retain(|w| w.id != id);
    open_windows.set(list);
}

pub fn minimize_window(mut open_windows: Signal<Vec<OpenWindow>>, id: AppId) {
    let mut list = open_windows();
    if let Some(w) = list.iter_mut().find(|w| w.id == id) {
        w.minimized = true;
    }
    open_windows.set(list);
}

/// Toggling maximize deliberately resets to `default_geometry()` on
/// restore rather than remembering the exact pre-maximize dragged spot —
/// position while *not* maximized is owned by the drag JS, not Dioxus, so
/// there's nothing here to restore to. Acceptable trade-off for now.
pub fn toggle_maximize(mut open_windows: Signal<Vec<OpenWindow>>, id: AppId) {
    let mut list = open_windows();
    if let Some(w) = list.iter_mut().find(|w| w.id == id) {
        w.maximized = !w.maximized;
    }
    open_windows.set(list);
}

/// Small persistent drag/resize utility, `eval`'d once (idempotent) so it
/// survives Dioxus re-renders as real global browser state. Continuous
/// mousemove tracking is handled entirely in JS — routing every pixel of
/// a drag back through the WASM boundary would be both slower and riskier
/// to get right without a local compiler to check against.
pub const WINDOW_MANAGER_JS: &str = r#"
(function () {
  if (window.__wm) return;
  function makeDraggable(handleId, windowId) {
    var handle = document.getElementById(handleId);
    var win = document.getElementById(windowId);
    if (!handle || !win) return;
    var sx = 0, sy = 0, sl = 0, st = 0, dragging = false;
    handle.addEventListener('mousedown', function (e) {
      if (e.target.closest('[data-no-drag]')) return;
      dragging = true;
      sx = e.clientX; sy = e.clientY;
      var rect = win.getBoundingClientRect();
      var parent = win.offsetParent ? win.offsetParent.getBoundingClientRect() : { left: 0, top: 0 };
      sl = rect.left - parent.left;
      st = rect.top - parent.top;
      e.preventDefault();
    });
    document.addEventListener('mousemove', function (e) {
      if (!dragging) return;
      var nl = sl + (e.clientX - sx);
      var nt = Math.max(0, st + (e.clientY - sy));
      win.style.left = nl + 'px';
      win.style.top = nt + 'px';
    });
    document.addEventListener('mouseup', function () { dragging = false; });
  }
  function makeResizable(handleId, windowId) {
    var handle = document.getElementById(handleId);
    var win = document.getElementById(windowId);
    if (!handle || !win) return;
    var sx = 0, sy = 0, sw = 0, sh = 0, resizing = false;
    handle.addEventListener('mousedown', function (e) {
      resizing = true;
      sx = e.clientX; sy = e.clientY;
      var rect = win.getBoundingClientRect();
      sw = rect.width; sh = rect.height;
      e.preventDefault();
      e.stopPropagation();
    });
    document.addEventListener('mousemove', function (e) {
      if (!resizing) return;
      win.style.width = Math.max(260, sw + (e.clientX - sx)) + 'px';
      win.style.height = Math.max(180, sh + (e.clientY - sy)) + 'px';
    });
    document.addEventListener('mouseup', function () { resizing = false; });
  }
  window.__wm = { makeDraggable: makeDraggable, makeResizable: makeResizable };
})();
"#;

/// Generic chrome around every app window: title bar (icon slot via
/// `title()`, drag handle, traffic-light buttons) + a scrollable content
/// area for whatever `children` the caller renders + a resize handle.
#[component]
pub fn WindowFrame(
    id: AppId,
    z: i32,
    maximized: bool,
    minimized: bool,
    open_windows: Signal<Vec<OpenWindow>>,
    next_z: Signal<i32>,
    children: Element,
) -> Element {
    let (x, y, w, h) = id.default_geometry();
    let dom_id = format!("win-{}", id.key());
    let handle_id = format!("win-{}-handle", id.key());
    let resize_id = format!("win-{}-resize", id.key());

    let position_style = if maximized {
        "inset: 12px;".to_string()
    } else {
        format!("left:{x}px; top:{y}px; width:{w}px; height:{h}px;")
    };
    let style = format!("{position_style} z-index:{z};");

    let handle_mount_id = handle_id.clone();
    let handle_mount_win = dom_id.clone();
    let resize_mount_id = resize_id.clone();
    let resize_mount_win = dom_id.clone();

    rsx! {
        div {
            id: "{dom_id}",
            class: "window-pop-in absolute bg-black border border-white/20 flex flex-col shadow-2xl text-white font-mono",
            class: if minimized { "hidden" } else { "" },
            style: "{style}",
            onmousedown: move |_| focus_window(open_windows, next_z, id),

            div {
                id: "{handle_id}",
                class: "flex items-center justify-between px-3 py-2 border-b border-white/15 cursor-move select-none shrink-0",
                onmounted: move |_| {
                    let handle_js = handle_mount_id.clone();
                    let win_js = handle_mount_win.clone();
                    spawn(async move {
                        document::eval(
                                &format!(
                                    "window.__wm && window.__wm.makeDraggable('{handle_js}', '{win_js}');",
                                ),
                            )
                            .await
                            .ok();
                    });
                },
                span { class: "text-xs text-white/80 truncate", "{id.title()}" }
                div {
                    "data-no-drag": "true",
                    class: "flex items-center gap-1.5",
                    button {
                        r#type: "button",
                        title: "Minimize",
                        class: "w-2.5 h-2.5 rounded-full bg-yellow-400 hover:brightness-110",
                        onclick: move |_| minimize_window(open_windows, id),
                    }
                    button {
                        r#type: "button",
                        title: "Maximize",
                        class: "w-2.5 h-2.5 rounded-full bg-green-500 hover:brightness-110",
                        onclick: move |_| toggle_maximize(open_windows, id),
                    }
                    button {
                        r#type: "button",
                        title: "Close",
                        class: "w-2.5 h-2.5 rounded-full bg-red-500 hover:brightness-110",
                        onclick: move |_| close_window(open_windows, id),
                    }
                }
            }

            div { class: "flex-1 min-h-0 overflow-auto", {children} }

            if !maximized {
                div {
                    id: "{resize_id}",
                    class: "absolute bottom-0 right-0 w-4 h-4 cursor-nwse-resize",
                    onmounted: move |_| {
                        let resize_js = resize_mount_id.clone();
                        let win_js = resize_mount_win.clone();
                        spawn(async move {
                            document::eval(
                                    &format!(
                                        "window.__wm && window.__wm.makeResizable('{resize_js}', '{win_js}');",
                                    ),
                                )
                                .await
                                .ok();
                        });
                    },
                }
            }
        }
    }
}
