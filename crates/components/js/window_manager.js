(function () {
  if (window.__wm) return;
  // Must match `assets/styles/window.css`'s `@media (min-width: 768px)` — update
  // both together if the project's Tailwind `md:` breakpoint changes.
  var DESKTOP_BREAKPOINT = 768;
  // Jarak minimum jendela ke tepi `.desktop__stage`; harus sama dengan 12px di
  // `clamp()` pada `assets/styles/window.css`.
  var MARGIN = 12;
  // Durasi animasi maximize/restore (ms); samakan dengan nuansa `--dur-panel-in`.
  var FLIP_MS = 240;

  // Satu frame = satu pembaruan. Pointer bisa mengirim jauh lebih banyak event
  // daripada frame yang digambar (mouse 500-1000 Hz), jadi event hanya mencatat
  // posisi terakhir dan kerja yang menyentuh DOM dijalankan sekali per frame.
  // Tanpa `requestAnimationFrame` (tes DOM tiruan) dijalankan langsung.
  function raf(fn) {
    return window.requestAnimationFrame ? window.requestAnimationFrame(fn) : (fn(), 0);
  }
  function caf(id) {
    if (id && window.cancelAnimationFrame) window.cancelAnimationFrame(id);
  }
  function clamp(v, lo, hi) { return Math.max(lo, Math.min(v, hi)); }

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

  // ---- Maximize / restore: animasi FLIP (First, Last, Invert, Play) ----
  // Menganimasikan `left/top/width/height` memaksa layout ulang seluruh isi
  // jendela di setiap frame. Di sini layout berubah SEKALI ke ukuran akhir, lalu
  // jendela "ditarik balik" ke kotak awalnya dengan `transform` dan dilepas
  // menuju kotak akhir; `transform` berjalan di compositor, bukan di main thread.
  //
  // Kotak awal diambil saat tombol maximize diklik (fase capture, sebelum handler
  // Rust mengubah state); `MutationObserver` menunggu class `win-maximized`
  // berubah, lalu memutar animasinya sebelum frame berikutnya digambar.
  var flip = null; // { win, rect, maxed, at } milik klik tombol maximize yang menunggu
  var flipObserver = null;
  function playFlip(win, first) {
    if (!win.animate) return;
    if (window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;
    var last = win.getBoundingClientRect();
    if (!last.width || !last.height) return;
    var dx = first.left - last.left, dy = first.top - last.top;
    var sx = first.width / last.width, sy = first.height / last.height;
    if (!dx && !dy && sx === 1 && sy === 1) return; // mis. di bawah breakpoint: selalu fullscreen
    win.animate([
      { transformOrigin: '0 0', transform: 'translate(' + dx + 'px,' + dy + 'px) scale(' + sx + ',' + sy + ')' },
      { transformOrigin: '0 0', transform: 'translate(0px,0px) scale(1,1)' }
    ], { duration: FLIP_MS, easing: 'cubic-bezier(0.16, 1, 0.3, 1)' });
  }
  function settleFlip() {
    var f = flip;
    if (!f) return;
    if (!f.win.isConnected) { flip = null; flipObserver.disconnect(); return; }
    if (f.win.classList.contains('win-maximized') === f.maxed) return; // class lain yang berubah
    flip = null;
    flipObserver.disconnect();
    playFlip(f.win, f.rect);
  }
  document.addEventListener('click', function (e) {
    var dot = e.target && e.target.closest && e.target.closest('.window__dot--max');
    var win = dot && dot.closest('.win-frame');
    if (!win || !window.MutationObserver) return;
    if (!flipObserver) flipObserver = new window.MutationObserver(settleFlip);
    var f = { win: win, rect: win.getBoundingClientRect(), maxed: win.classList.contains('win-maximized') };
    flip = f;
    flipObserver.observe(document.documentElement, { attributes: true, attributeFilter: ['class'], subtree: true });
    // Tidak ada perubahan class (mis. klik diabaikan): lepas pengamat, jangan menggantung.
    window.setTimeout(function () {
      if (flip === f) { flip = null; flipObserver.disconnect(); }
    }, 1000);
  }, true);

  function makeDraggable(handleId, windowId, onEnd) {
    var handle = document.getElementById(handleId);
    var win = document.getElementById(windowId);
    if (!handle || !win) return;
    // sl/st = posisi awal; maxL/maxT = batas kanan/bawah (sama dengan `clamp()` di
    // window.css); nl/nt = posisi terakhir yang sudah dihitung.
    var sx = 0, sy = 0, sl = 0, st = 0, maxL = Infinity, maxT = Infinity;
    var nl = 0, nt = 0, px = 0, py = 0, frame = 0, dragging = false;
    var gesture = { move: move, stop: stop };
    function start(x, y, target) {
      if (window.innerWidth < DESKTOP_BREAKPOINT) return;
      if (win.classList.contains('win-maximized')) return;
      if (target && target.closest && target.closest('[data-no-drag]')) return;
      dragging = true;
      activeDrag = gesture;
      win.setAttribute('data-gesture', '');
      sx = x; sy = y; px = x; py = y;
      var rect = win.getBoundingClientRect();
      var parent = win.offsetParent;
      var pr = parent ? parent.getBoundingClientRect() : { left: 0, top: 0 };
      sl = rect.left - pr.left - (parent ? parent.clientLeft : 0);
      st = rect.top - pr.top - (parent ? parent.clientTop : 0);
      nl = sl; nt = st;
      maxL = parent ? parent.clientWidth - win.offsetWidth - MARGIN : Infinity;
      maxT = parent ? parent.clientHeight - win.offsetHeight - MARGIN : Infinity;
    }
    // Selama drag HANYA `transform` yang berubah (compositor; tanpa layout dan tanpa
    // style recalc untuk isi jendela). `--win-x/-y` ditulis sekali di `stop`.
    function paint() {
      frame = 0;
      nl = clamp(sl + (px - sx), MARGIN, maxL);
      nt = clamp(st + (py - sy), MARGIN, maxT);
      win.style.transform = 'translate3d(' + (nl - sl) + 'px,' + (nt - st) + 'px,0)';
    }
    function move(x, y) {
      if (!dragging) return;
      px = x; py = y;
      if (!frame) frame = raf(paint);
    }
    function stop() {
      if (!dragging) return;
      dragging = false;
      if (activeDrag === gesture) activeDrag = null;
      caf(frame);
      paint();
      // Serahkan posisi akhir ke CSS (`--win-x/-y`) dan lepas transform dalam satu
      // langkah, supaya tidak ada frame yang menampilkan keduanya sekaligus.
      win.style.setProperty('--win-x', nl + 'px');
      win.style.setProperty('--win-y', nt + 'px');
      win.style.transform = '';
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
      var px = sx, py = sy, frame = 0;

      handle.setPointerCapture(e.pointerId);
      win.setAttribute('data-gesture', '');
      document.documentElement.style.userSelect = 'none';

      // Ukuran berubah, jadi layout isi memang harus dihitung ulang; setidaknya
      // sekali per frame saja, tidak sekali per event pointer.
      function paint() {
        frame = 0;
        var dx = px - sx, dy = py - sy;
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

      function move(ev) {
        moved = true;
        px = ev.clientX; py = ev.clientY;
        if (!frame) frame = raf(paint);
      }

      function stop() {
        handle.removeEventListener('pointermove', move);
        handle.removeEventListener('pointerup', stop);
        handle.removeEventListener('pointercancel', stop);
        document.documentElement.style.userSelect = '';
        if (moved) { caf(frame); paint(); }
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
