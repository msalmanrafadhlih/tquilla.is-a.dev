const test = require('node:test');
const assert = require('node:assert/strict');
const { run } = require('./helpers');

function setup() {
  const docListeners = {};
  const count = () => Object.values(docListeners).reduce((a, b) => a + b.length, 0);
  const els = {};
  const mkEl = (id) => {
    const l = {};
    return {
      id,
      classList: { contains: () => false },
      style: { setProperty(k, v) { this[k] = v; } },
      offsetLeft: 10, offsetTop: 20, offsetWidth: 300, offsetHeight: 200, offsetParent: null,
      getBoundingClientRect: () => ({ left: 50, top: 60, width: 300, height: 200 }),
      setAttribute() {}, removeAttribute() {},
      addEventListener(t, f) { (l[t] = l[t] || []).push(f); },
      _l: l,
      closest: () => null,
    };
  };
  const win = { innerWidth: 1200 };
  const doc = {
    getElementById: (id) => els[id],
    addEventListener(t, f) { (docListeners[t] = docListeners[t] || []).push(f); },
    documentElement: { style: {} },
  };
  run('window_manager.js', win, doc);
  const fire = (t, e) => (docListeners[t] || []).forEach((f) => f(e));
  const add = (i, onEnd) => {
    els['h' + i] = mkEl('h' + i);
    els['w' + i] = mkEl('w' + i);
    win.__wm.makeDraggable('h' + i, 'w' + i, onEnd);
  };
  return { win, els, count, fire, add };
}

test('registers a fixed number of document listeners, however many windows open', () => {
  const s = setup();
  const afterInit = s.count();
  for (let i = 0; i < 50; i++) s.add(i, () => {});
  assert.equal(s.count(), afterInit);
  // mousemove, mouseup, touchmove, touchend, touchcancel + click (maximize/FLIP).
  assert.equal(afterInit, 6);
});

test('init is idempotent', () => {
  const s = setup();
  const before = s.count();
  run('window_manager.js', s.win, { addEventListener() { throw new Error('re-registered'); } });
  assert.equal(s.count(), before);
});

test('mouse drag moves only the dragged window and reports once on release', () => {
  const s = setup();
  const ends = [];
  s.add(7, (...a) => ends.push(a));
  s.add(8, () => {});
  s.els.h7._l.mousedown[0]({ clientX: 100, clientY: 100, target: { closest: () => null }, preventDefault() {} });
  s.fire('mousemove', { clientX: 160, clientY: 130 });
  // Selama drag hanya `transform` yang berubah; --win-x/-y baru ditulis saat dilepas.
  assert.equal(s.els.w7.style.transform, 'translate3d(60px,30px,0)');
  assert.equal(s.els.w7.style['--win-x'], undefined);
  assert.equal(s.els.w8.style.transform, undefined);
  s.fire('mouseup', {});
  assert.equal(ends.length, 1);
  assert.equal(s.els.w7.style['--win-x'], '110px');
  assert.equal(s.els.w7.style['--win-y'], '90px');
  assert.equal(s.els.w7.style.transform, '', 'transform must be released with the final position');
  const x = s.els.w7.style['--win-x'];
  s.fire('mousemove', { clientX: 300, clientY: 300 });
  assert.equal(s.els.w7.style['--win-x'], x, 'moves after mouseup must be ignored');
  assert.equal(s.els.w7.style.transform, '');
});

test('drag is clamped like the clamp() in window.css (12px margin)', () => {
  const s = setup();
  s.add(1, () => {});
  s.els.w1.offsetParent = {
    getBoundingClientRect: () => ({ left: 0, top: 0 }),
    clientWidth: 1000, clientHeight: 700, clientLeft: 0, clientTop: 0,
  };
  const press = () => s.els.h1._l.mousedown[0]({ clientX: 100, clientY: 100, target: { closest: () => null }, preventDefault() {} });
  press();
  s.fire('mousemove', { clientX: 5000, clientY: 5000 });
  s.fire('mouseup', {});
  // 1000 - 300 - 12 dan 700 - 200 - 12
  assert.equal(s.els.w1.style['--win-x'], '688px');
  assert.equal(s.els.w1.style['--win-y'], '488px');
  press();
  s.fire('mousemove', { clientX: -5000, clientY: -5000 });
  s.fire('mouseup', {});
  assert.equal(s.els.w1.style['--win-x'], '12px');
  assert.equal(s.els.w1.style['--win-y'], '12px');
});

test('many pointer events in one frame cost a single DOM write', () => {
  const s = setup();
  const queue = [];
  s.win.requestAnimationFrame = (fn) => { queue.push(fn); return queue.length; };
  s.win.cancelAnimationFrame = () => {};
  s.add(2, () => {});
  s.els.h2._l.mousedown[0]({ clientX: 0, clientY: 0, target: { closest: () => null }, preventDefault() {} });
  for (let i = 1; i <= 20; i++) s.fire('mousemove', { clientX: i, clientY: i });
  assert.equal(queue.length, 1, 'only one frame may be scheduled');
  assert.equal(s.els.w2.style.transform, undefined, 'nothing written before the frame runs');
  queue.shift()();
  assert.equal(s.els.w2.style.transform, 'translate3d(20px,20px,0)', 'the last pointer position wins');
});

test('maximize plays a FLIP transform instead of animating layout', () => {
  const s = setup();
  const observers = [];
  s.win.MutationObserver = class { constructor(cb) { this.cb = cb; observers.push(this); } observe() {} disconnect() {} };
  s.win.setTimeout = () => 0;
  let rect = { left: 50, top: 60, width: 300, height: 200 };
  let maximized = false;
  const calls = [];
  const frame = {
    isConnected: true,
    classList: { contains: (c) => c === 'win-maximized' && maximized },
    getBoundingClientRect: () => rect,
    animate: (keys, opts) => calls.push({ keys, opts }),
  };
  const dot = { closest: (sel) => (sel === '.win-frame' ? frame : null) };
  s.fire('click', { target: { closest: (sel) => (sel === '.window__dot--max' ? dot : null) } });
  assert.equal(observers.length, 1);
  // Belum ada perubahan class: tidak ada animasi.
  observers[0].cb();
  assert.equal(calls.length, 0);
  // Rust menandai maximized, layout langsung ke kotak akhir.
  maximized = true;
  rect = { left: 0, top: 0, width: 1000, height: 700 };
  observers[0].cb();
  assert.equal(calls.length, 1);
  assert.deepEqual(Object.keys(calls[0].keys[0]).sort(), ['transform', 'transformOrigin']);
  assert.match(calls[0].keys[0].transform, /^translate\(50px,60px\) scale\(0\.3,0\.28/);
  assert.equal(calls[0].keys[1].transform, 'translate(0px,0px) scale(1,1)');
  // Satu klik = satu animasi.
  observers[0].cb();
  assert.equal(calls.length, 1);
});

test('clicks elsewhere never start an animation', () => {
  const s = setup();
  s.win.MutationObserver = class { constructor() { throw new Error('observer must not be created'); } };
  s.fire('click', { target: { closest: () => null } });
  s.fire('click', { target: {} });
});

test('touch drag prevents default scrolling and reports on touchend', () => {
  const s = setup();
  const ends = [];
  s.add(3, (...a) => ends.push(a));
  s.els.h3._l.touchstart[0]({ touches: [{ clientX: 10, clientY: 10 }], target: { closest: () => null } });
  let prevented = false;
  s.fire('touchmove', { touches: [{ clientX: 30, clientY: 40 }], preventDefault() { prevented = true; } });
  s.fire('touchend', {});
  assert.ok(prevented);
  assert.equal(ends.length, 1);
});
