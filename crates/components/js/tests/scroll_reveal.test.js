const test = require('node:test');
const assert = require('node:assert/strict');
const { run } = require('./helpers');

function observerClass(record) {
  return class {
    constructor(cb, opts) { record.cb = cb; record.opts = opts; record.observed = []; record.unobserved = []; }
    observe(el) { record.observed.push(el); }
    unobserve(el) { record.unobserved.push(el); }
  };
}

test('does nothing (and creates no observer) when there are no [data-reveal] elements', () => {
  let created = false;
  const doc = { querySelectorAll: () => [] };
  run('scroll_reveal.js', {}, doc, { IntersectionObserver: class { constructor() { created = true; } } });
  assert.equal(created, false);
});

test('observes every element and reveals each one only once', () => {
  const mk = () => ({ classList: { added: [], add(c) { this.added.push(c); } } });
  const els = [mk(), mk()];
  const rec = {};
  run('scroll_reveal.js', {}, { querySelectorAll: () => els }, { IntersectionObserver: observerClass(rec) });
  assert.equal(rec.observed.length, 2);
  const obs = { unobserve: (el) => rec.unobserved.push(el) };
  rec.cb([{ isIntersecting: false, target: els[0] }, { isIntersecting: true, target: els[1] }], obs);
  assert.deepEqual(els[0].classList.added, []);
  assert.deepEqual(els[1].classList.added, ['is-visible']);
  assert.deepEqual(rec.unobserved, [els[1]]);
});
