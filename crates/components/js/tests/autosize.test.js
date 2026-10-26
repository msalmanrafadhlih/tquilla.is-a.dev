const test = require('node:test');
const assert = require('node:assert/strict');
const { run } = require('./helpers');

// Textarea tiruan: `scrollHeight` mengikuti jumlah baris (tinggi baris 20,
// padding 3 + 3), seperti browser sungguhan setelah height = auto.
function setup({ borderBox = true, lines = 1, width = 300 } = {}) {
  const listeners = {};
  const observers = [];
  const el = {
    style: {},
    lines,
    clientWidth: width,
    get scrollHeight() {
      return this.lines * 20 + 6;
    },
    addEventListener(type, fn) {
      (listeners[type] = listeners[type] || []).push(fn);
    },
  };
  const win = {
    getComputedStyle: () => ({
      fontSize: '14px',
      lineHeight: '20px',
      paddingTop: '3px',
      paddingBottom: '3px',
      borderTopWidth: '0px',
      borderBottomWidth: '0px',
      boxSizing: borderBox ? 'border-box' : 'content-box',
    }),
    ResizeObserver: class {
      constructor(cb) {
        this.cb = cb;
        observers.push(this);
      }
      observe() {}
    },
  };
  const doc = { getElementById: (id) => (id === 'q' ? el : null) };
  const go = () => run('autosize.js', win, doc, { ID: 'q', MAX_ROWS: 5 });
  return { el, listeners, observers, go };
}

test('one line: height of one line plus padding, no scrollbar', () => {
  const s = setup({ lines: 1 });
  assert.equal(s.go(), true);
  assert.equal(s.el.style.height, '26px');
  assert.equal(s.el.style.overflowY, 'hidden');
});

test('grows line by line up to five lines', () => {
  for (const [lines, height] of [[2, 46], [3, 66], [4, 86], [5, 106]]) {
    const s = setup({ lines });
    s.go();
    assert.equal(s.el.style.height, `${height}px`, `${lines} lines`);
    assert.equal(s.el.style.overflowY, 'hidden');
  }
});

test('stops at five lines and scrolls inside from then on', () => {
  for (const lines of [6, 7, 40]) {
    const s = setup({ lines });
    s.go();
    assert.equal(s.el.style.height, '106px', `${lines} lines`);
    assert.equal(s.el.style.overflowY, 'auto');
  }
});

test('content-box: the height excludes padding', () => {
  const s = setup({ borderBox: false, lines: 3 });
  s.go();
  assert.equal(s.el.style.height, '60px');
  const big = setup({ borderBox: false, lines: 9 });
  big.go();
  assert.equal(big.el.style.height, '100px');
});

test('shrinks again when the text gets shorter', () => {
  const s = setup({ lines: 6 });
  s.go();
  assert.equal(s.el.style.height, '106px');
  s.el.lines = 1;
  s.listeners.input[0]();
  assert.equal(s.el.style.height, '26px');
  assert.equal(s.el.style.overflowY, 'hidden');
});

test('the height is reset to auto before measuring', () => {
  const s = setup({ lines: 2 });
  s.el.style.height = '999px';
  let seen = null;
  Object.defineProperty(s.el, 'scrollHeight', {
    get() {
      seen = s.el.style.height;
      return 46;
    },
  });
  s.go();
  assert.equal(seen, 'auto');
});

test('listeners are installed once, however often it runs', () => {
  const s = setup();
  s.go();
  s.go();
  s.go();
  assert.equal(s.listeners.input.length, 1);
  assert.equal(s.observers.length, 1);
});

test('only a width change re-fits (a height change must not loop)', () => {
  const s = setup({ lines: 1, width: 300 });
  s.go();
  s.el.lines = 3;
  s.observers[0].cb(); // lebar sama
  assert.equal(s.el.style.height, '26px');
  s.el.clientWidth = 200;
  s.observers[0].cb();
  assert.equal(s.el.style.height, '66px');
});

test('a missing element is not an error', () => {
  const win = {};
  const doc = { getElementById: () => null };
  assert.equal(run('autosize.js', win, doc, { ID: 'q', MAX_ROWS: 5 }), false);
});

test('works without ResizeObserver', () => {
  const s = setup();
  const win = { getComputedStyle: () => ({ fontSize: '14px', lineHeight: '20px', paddingTop: '3px', paddingBottom: '3px', borderTopWidth: '0px', borderBottomWidth: '0px', boxSizing: 'border-box' }) };
  const doc = { getElementById: () => s.el };
  assert.equal(run('autosize.js', win, doc, { ID: 'q', MAX_ROWS: 5 }), true);
  assert.equal(s.el.style.height, '26px');
});
