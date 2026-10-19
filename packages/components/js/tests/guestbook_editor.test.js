const test = require('node:test');
const assert = require('node:assert/strict');
const { run } = require('./helpers');

// Runs the script against a fake textarea and returns what it did to it.
function apply(value, start, end, kind) {
  const events = [];
  const el = {
    value,
    selectionStart: start,
    selectionEnd: end,
    focused: false,
    focus() { this.focused = true; },
    setSelectionRange(a, b) { this.selectionStart = a; this.selectionEnd = b; },
    dispatchEvent(e) { events.push(e.type); },
  };
  const doc = { getElementById: (id) => (id === 'box' ? el : null) };
  const ok = run('guestbook_editor.js', {}, doc, { ID: 'box', KIND: kind, PLACEHOLDER: 'text' });
  return { ok, el, events, picked: el.value.slice(el.selectionStart, el.selectionEnd) };
}

test('wraps the selection and keeps it selected', () => {
  const r = apply('say hello now', 4, 9, 'bold');
  assert.equal(r.el.value, 'say **hello** now');
  assert.equal(r.picked, 'hello');
  assert.deepEqual(r.events, ['input']);
  assert.ok(r.el.focused);
  assert.equal(apply('a b', 0, 1, 'italic').el.value, '_a_ b');
  assert.equal(apply('a b', 0, 1, 'strike').el.value, '~~a~~ b');
  assert.equal(apply('a b', 0, 1, 'code').el.value, '`a` b');
});

test('with nothing selected it inserts the placeholder, selected', () => {
  const r = apply('ab', 1, 1, 'bold');
  assert.equal(r.el.value, 'a**text**b');
  assert.equal(r.picked, 'text');
});

test('pressing a wrap button twice takes the markers off again', () => {
  const once = apply('hello', 0, 5, 'bold');
  const twice = apply(once.el.value, once.el.selectionStart, once.el.selectionEnd, 'bold');
  assert.equal(twice.el.value, 'hello');
  assert.equal(twice.picked, 'hello');
  // Selection that includes the markers.
  const inner = apply('**hello**', 0, 9, 'bold');
  assert.equal(inner.el.value, 'hello');
});

test('inline code over several lines becomes a fenced block', () => {
  const r = apply('a\nb', 0, 3, 'code');
  assert.equal(r.el.value, '```\na\nb\n```');
  assert.equal(r.picked, 'a\nb');
});

test('code block starts on its own line', () => {
  const r = apply('intro', 5, 5, 'codeblock');
  assert.equal(r.el.value, 'intro\n```\ntext\n```');
  assert.equal(r.picked, 'text');
  assert.equal(apply('', 0, 0, 'codeblock').el.value, '```\ntext\n```');
});

test('link selects the address so it can be typed over', () => {
  const r = apply('see docs', 4, 8, 'link');
  assert.equal(r.el.value, 'see [docs](https://)');
  assert.equal(r.picked, 'https://');
  assert.equal(apply('', 0, 0, 'link').el.value, '[text](https://)');
});

test('image is a link with a bang and selects the address', () => {
  const r = apply('a cat', 0, 5, 'image');
  assert.equal(r.el.value, '![a cat](https://)');
  assert.equal(r.picked, 'https://');
  assert.equal(apply('', 0, 0, 'image').el.value, '![text](https://)');
});

test('line prefixes apply to every selected line and toggle off', () => {
  const on = apply('a\nb\nc', 0, 5, 'list');
  assert.equal(on.el.value, '- a\n- b\n- c');
  const off = apply(on.el.value, 0, on.el.value.length, 'list');
  assert.equal(off.el.value, 'a\nb\nc');
  assert.equal(apply('a\nb', 0, 1, 'quote').el.value, '> a\nb');
  assert.equal(apply('a', 0, 0, 'tasks').el.value, '- [ ] a');
});

test('a prefix applies to the whole line the cursor is in, even mid-line', () => {
  assert.equal(apply('one\ntwo', 5, 5, 'quote').el.value, 'one\n> two');
});

test('a selection ending after a newline leaves the next line alone', () => {
  assert.equal(apply('a\nb', 0, 2, 'list').el.value, '- a\nb');
});

test('prefix on an empty line inserts the placeholder', () => {
  const r = apply('', 0, 0, 'list');
  assert.equal(r.el.value, '- text');
  assert.equal(r.picked, 'text');
});

test('heading adds ## once and does not stack on existing headings', () => {
  assert.equal(apply('Title', 0, 0, 'heading').el.value, '## Title');
  assert.equal(apply('# Big', 0, 0, 'heading').el.value, '# Big');
});

test('unknown kinds change nothing, and a missing textarea is reported', () => {
  const r = apply('abc', 0, 1, 'nope');
  assert.equal(r.el.value, 'abc');
  const events = [];
  const missing = run('guestbook_editor.js', {}, { getElementById: () => null }, {
    ID: 'x', KIND: 'bold', PLACEHOLDER: 't',
  });
  assert.equal(missing, false);
  assert.deepEqual(events, []);
});

test('a reversed selection is handled like a normal one', () => {
  assert.equal(apply('hello', 5, 0, 'bold').el.value, '**hello**');
});
