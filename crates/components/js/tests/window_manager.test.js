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
  assert.equal(afterInit, 5);
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
  assert.notEqual(s.els.w7.style['--win-x'], undefined);
  assert.equal(s.els.w8.style['--win-x'], undefined);
  s.fire('mouseup', {});
  assert.equal(ends.length, 1);
  const x = s.els.w7.style['--win-x'];
  s.fire('mousemove', { clientX: 300, clientY: 300 });
  assert.equal(s.els.w7.style['--win-x'], x, 'moves after mouseup must be ignored');
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
