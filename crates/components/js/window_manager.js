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
