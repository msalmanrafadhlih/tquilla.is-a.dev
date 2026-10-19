// Toolbar actions of the Guestbook editor: wraps or prefixes the selection of
// the message textarea with Markdown syntax. Run through `document::eval`,
// with these provided before the body (see `guestbook.rs`):
//   ID          id of the textarea
//   KIND        one of bold, italic, strike, code, codeblock, link, image,
//               quote, list, tasks, heading
//   PLACEHOLDER text used when nothing is selected (comes from `Strings`)
// Returns true if the textarea was found and changed.

const WRAP = { bold: '**', italic: '_', strike: '~~', code: '`' };
const PREFIX = { quote: '> ', list: '- ', tasks: '- [ ] ', heading: '## ' };

// Selection and value of a textarea -> new value and selection. Pure, so the
// tests exercise it without a DOM. Indexes are UTF-16 units, as in the DOM.
function edit(value, start, end, kind, placeholder) {
  if (start > end) [start, end] = [end, start];
  const sel = value.slice(start, end);

  if (kind in WRAP) {
    const m = WRAP[kind];
    if (kind === 'code' && sel.includes('\n')) return edit(value, start, end, 'codeblock', placeholder);
    // Already wrapped just outside the selection: take the markers off.
    if (sel && value.slice(start - m.length, start) === m && value.slice(end, end + m.length) === m) {
      return {
        value: value.slice(0, start - m.length) + sel + value.slice(end + m.length),
        start: start - m.length,
        end: end - m.length,
      };
    }
    // Already wrapped inside the selection.
    if (sel.length >= m.length * 2 + 1 && sel.startsWith(m) && sel.endsWith(m)) {
      const inner = sel.slice(m.length, sel.length - m.length);
      return { value: value.slice(0, start) + inner + value.slice(end), start, end: start + inner.length };
    }
    const text = sel || placeholder;
    return {
      value: value.slice(0, start) + m + text + m + value.slice(end),
      start: start + m.length,
      end: start + m.length + text.length,
    };
  }

  if (kind === 'codeblock') {
    const text = sel || placeholder;
    const lead = start > 0 && value[start - 1] !== '\n' ? '\n' : '';
    const trail = end < value.length && value[end] !== '\n' ? '\n' : '';
    const open = lead + '```\n';
    return {
      value: value.slice(0, start) + open + text + '\n```' + trail + value.slice(end),
      start: start + open.length,
      end: start + open.length + text.length,
    };
  }

  if (kind === 'link' || kind === 'image') {
    // An image is a link with a `!` in front; its text is the alt text.
    const open = kind === 'image' ? '![' : '[';
    const text = sel || placeholder;
    const url = 'https://';
    const out = value.slice(0, start) + open + text + '](' + url + ')' + value.slice(end);
    // Select the address so it can be typed or pasted over.
    const from = start + open.length + text.length + 2;
    return { value: out, start: from, end: from + url.length };
  }

  if (kind in PREFIX) {
    const p = PREFIX[kind];
    // Work on whole lines.
    const lineStart = value.lastIndexOf('\n', start - 1) + 1;
    let lineEnd = value.indexOf('\n', end);
    if (lineEnd === -1) lineEnd = value.length;
    // A selection that ends right after a newline does not include the next line.
    if (end > start && value[end - 1] === '\n') lineEnd = end - 1;
    const block = value.slice(lineStart, lineEnd);
    const lines = block.split('\n');
    const all = lines.every((l) => l.startsWith(p));
    const changed = lines
      .map((l, i) => {
        if (all) return l.slice(p.length);
        if (kind === 'heading' && l.startsWith('#')) return l; // keep existing headings
        return p + l;
      })
      .join('\n');
    if (block === '' && !all) {
      const text = p + placeholder;
      return {
        value: value.slice(0, lineStart) + text + value.slice(lineEnd),
        start: lineStart + p.length,
        end: lineStart + text.length,
      };
    }
    return {
      value: value.slice(0, lineStart) + changed + value.slice(lineEnd),
      start: lineStart,
      end: lineStart + changed.length,
    };
  }

  return { value, start, end };
}

const el = document.getElementById(ID);
if (!el) return false;
const r = edit(el.value, el.selectionStart, el.selectionEnd, KIND, PLACEHOLDER);
el.value = r.value;
el.focus();
el.setSelectionRange(r.start, r.end);
el.dispatchEvent(new Event('input', { bubbles: true }));
return true;
