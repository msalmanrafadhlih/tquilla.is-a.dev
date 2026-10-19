// Menjalankan skrip di js/ persis seperti `document::eval`: isinya
// diperlakukan sebagai badan fungsi (jadi `return` di level atas, seperti di
// scroll_reveal.js, valid), dengan `window` dan `document` palsu.
const fs = require('node:fs');
const path = require('node:path');

function readScript(name) {
  return fs.readFileSync(path.join(__dirname, '..', name), 'utf8');
}

function run(name, win, doc, extra = {}) {
  const keys = ['window', 'document', ...Object.keys(extra)];
  const fn = new Function(...keys, readScript(name));
  return fn(win, doc, ...Object.values(extra));
}

module.exports = { readScript, run };
