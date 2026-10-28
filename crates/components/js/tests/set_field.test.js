const test = require('node:test');
const assert = require('node:assert/strict');
const { run } = require('./helpers');

function setup(value) {
  const el = {
    value,
    caret: null,
    setSelectionRange(a, b) {
      this.caret = [a, b];
    },
  };
  const doc = { getElementById: (id) => (id === 'f' ? el : null) };
  const go = (VALUE, EXPECT) => run('set_field.js', {}, doc, { ID: 'f', VALUE, EXPECT });
  return { el, go, doc };
}

test('missing element: nothing happens', () => {
  const doc = { getElementById: () => null };
  assert.equal(run('set_field.js', {}, doc, { ID: 'x', VALUE: '', EXPECT: null }), false);
});

test('EXPECT null: always replaces and puts the caret at the end', () => {
  const s = setup('abc');
  assert.equal(s.go('hello', null), true);
  assert.equal(s.el.value, 'hello');
  assert.deepEqual(s.el.caret, [5, 5]);
});

test('clears when the field still holds exactly what was sent', () => {
  const s = setup('halo dunia');
  assert.equal(s.go('', 'halo dunia'), true);
  assert.equal(s.el.value, '');
});

test('keeps keystrokes typed after sending (field differs from EXPECT)', () => {
  const s = setup('halo dunia, lagi');
  assert.equal(s.go('', 'halo dunia'), false);
  assert.equal(s.el.value, 'halo dunia, lagi');
});

test('whitespace and newlines are compared exactly', () => {
  const s = setup('a\n');
  assert.equal(s.go('', 'a'), false);
  assert.equal(s.go('', 'a\n'), true);
});

test('an element without setSelectionRange still works', () => {
  const s = setup('x');
  delete s.el.setSelectionRange;
  assert.equal(s.go('y', null), true);
  assert.equal(s.el.value, 'y');
});

test('setSelectionRange throwing (input type=number) is tolerated', () => {
  const s = setup('x');
  s.el.setSelectionRange = () => {
    throw new Error('InvalidStateError');
  };
  assert.equal(s.go('7', null), true);
  assert.equal(s.el.value, '7');
});
