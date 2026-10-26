const test = require('node:test');
const assert = require('node:assert/strict');
const { run } = require('./helpers');

function setup() {
  const log = [];
  const link = {
    style: {},
    click() {
      log.push(['click', this.href, this.download]);
    },
    remove() {
      log.push(['remove']);
    },
  };
  const doc = {
    createElement: (tag) => {
      log.push(['create', tag]);
      return link;
    },
    body: { appendChild: (el) => log.push(['append', el === link]) },
  };
  class FakeBlob {
    constructor(parts, opts) {
      this.parts = parts;
      this.type = opts.type;
    }
  }
  const URLlike = {
    createObjectURL: (b) => {
      log.push(['url', b.parts.join(''), b.type]);
      return 'blob:fake';
    },
    revokeObjectURL: (u) => log.push(['revoke', u]),
  };
  const timers = [];
  const setTimeoutFake = (fn, ms) => timers.push([fn, ms]);
  return { log, doc, link, timers, extra: { Blob: FakeBlob, URL: URLlike, setTimeout: setTimeoutFake } };
}

test('builds a blob, clicks a hidden download link with the file name, then cleans up', () => {
  const s = setup();
  const ok = run('download_file.js', {}, s.doc, {
    FILENAME: 'chat-a.json',
    CONTENT: '{"a":1}',
    MIME: 'application/json',
    ...s.extra,
  });
  assert.equal(ok, true);
  assert.deepEqual(s.log, [
    ['url', '{"a":1}', 'application/json;charset=utf-8'],
    ['create', 'a'],
    ['append', true],
    ['click', 'blob:fake', 'chat-a.json'],
    ['remove'],
  ]);
  assert.equal(s.link.style.display, 'none');
  // Pencabutan URL ditunda, tidak langsung.
  assert.equal(s.timers.length, 1);
  assert.equal(s.timers[0][1], 1000);
  s.timers[0][0]();
  assert.deepEqual(s.log.at(-1), ['revoke', 'blob:fake']);
});

test('the content is passed through untouched (unicode, quotes, newlines)', () => {
  const s = setup();
  const content = '{"t":"halo \\"dunia\\"\\n日本語 ✓"}';
  run('download_file.js', {}, s.doc, { FILENAME: 'x.json', CONTENT: content, MIME: 'application/json', ...s.extra });
  assert.equal(s.log[0][1], content);
});
