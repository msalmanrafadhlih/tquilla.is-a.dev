const test = require('node:test');
const assert = require('node:assert/strict');
const { run } = require('./helpers');

function setup() {
  const listeners = [];
  const win = {};
  const doc = { addEventListener: (t, f) => listeners.push([t, f]) };
  run('disable_context_menu.js', win, doc);
  return { win, doc, listeners };
}

test('registers one contextmenu listener, even if run twice (remount)', () => {
  const s = setup();
  run('disable_context_menu.js', s.win, s.doc);
  assert.equal(s.listeners.length, 1);
  assert.equal(s.listeners[0][0], 'contextmenu');
});

test('blocks the menu except inside inputs / data-allow-contextmenu', () => {
  const s = setup();
  const handler = s.listeners[0][1];
  let blocked = 0;
  const ev = (allowed) => ({ target: { closest: () => allowed }, preventDefault() { blocked++; } });
  handler(ev(null));
  assert.equal(blocked, 1);
  handler(ev({}));
  assert.equal(blocked, 1);
});
