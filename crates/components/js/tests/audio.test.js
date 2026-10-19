const test = require('node:test');
const assert = require('node:assert/strict');
const { run } = require('./helpers');

const tick = () => new Promise((r) => setTimeout(r, 5));

function setup() {
  const log = [];
  const mkEl = (id) => ({
    id, src: '', volume: 1, played: 0, paused: false,
    play() { this.played++; log.push('play ' + id); return Promise.resolve(); },
    pause() { this.paused = true; },
  });
  const els = { a: mkEl('a'), b: mkEl('b') };
  const scripts = [];
  const win = {};
  const doc = {
    getElementById: (i) => els[i],
    createElement: () => ({ remove() {} }),
    head: { appendChild(s) { scripts.push(s); } },
  };
  run('audio.js', win, doc);
  const fakeHls = (onLoad) => {
    function Hls() {
      this.loadSource = (u) => onLoad(u);
      this.attachMedia = () => {};
      this.destroy = () => {};
    }
    Hls.isSupported = () => true;
    return Hls;
  };
  return { win, els, scripts, fakeHls };
}

test('plain mp3 never loads hls.js and plays immediately', async () => {
  const s = setup();
  s.win.__audio.load('a', 'https://x/y.mp3');
  s.win.__audio.play('a');
  await tick();
  assert.equal(s.els.a.src, 'https://x/y.mp3');
  assert.equal(s.scripts.length, 0);
  assert.equal(s.els.a.played, 1);
});

test('m3u8: load() then play() back-to-back waits for hls.js', async () => {
  const s = setup();
  s.win.__audio.load('b', 'https://x/s.m3u8');
  s.win.__audio.play('b');
  await tick();
  assert.equal(s.scripts.length, 1);
  assert.equal(s.els.b.played, 0, 'must not play before hls.js arrives');
  let attached = null;
  s.win.Hls = s.fakeHls((u) => { attached = u; });
  s.scripts[0].onload();
  await tick();
  assert.equal(attached, 'https://x/s.m3u8');
  assert.equal(s.els.b.played, 1);
  assert.equal(s.els.b.src, '', 'src stays untouched when hls.js handles the stream');
});

test('a second m3u8 does not inject another script', async () => {
  const s = setup();
  s.win.__audio.load('b', 'https://x/s.m3u8');
  s.win.Hls = s.fakeHls(() => {});
  s.scripts[0].onload();
  await tick();
  s.win.__audio.load('a', 'https://x/2.m3u8');
  await tick();
  assert.equal(s.scripts.length, 1);
});

test('a load superseded before hls.js arrives is dropped', async () => {
  const s = setup();
  let attached = null;
  s.win.__audio.load('b', 'https://x/old.m3u8');
  s.win.__audio.load('b', 'https://x/new.mp3');
  s.win.Hls = s.fakeHls((u) => { attached = 'STALE:' + u; });
  s.scripts[0].onload();
  await tick();
  assert.equal(attached, null);
  assert.equal(s.els.b.src, 'https://x/new.mp3');
});

test('hls.js load failure falls back to src and allows a retry', async () => {
  const s = setup();
  s.win.__audio.load('a', 'https://x/f.m3u8');
  s.scripts[0].onerror();
  await tick();
  assert.equal(s.els.a.src, 'https://x/f.m3u8');
  assert.equal(s.win.__hlsLoading, null);
});
