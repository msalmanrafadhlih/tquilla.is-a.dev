// Textarea yang tingginya mengikuti isi: mulai 1 baris, naik sampai MAX_ROWS
// baris, lalu bisa digulir. Dijalankan lewat `document::eval`; sebelum badan
// skrip ini ditulis:
//   ID        id textarea
//   MAX_ROWS  jumlah baris maksimum (mis. 5)
// Aman dipanggil berulang: pendengar `input` dan pengamat lebar hanya dipasang
// sekali per elemen, setiap panggilan hanya menghitung ulang tinggi. Dipanggil
// lagi setelah isi dikosongkan dari kode (mengubah `value` tidak memicu
// `input`). Mengembalikan true bila elemennya ada.

const el = document.getElementById(ID);
if (!el) return false;

function fit() {
  // Ukur dari tinggi "auto" supaya bisa menyusut juga.
  el.style.height = 'auto';
  const cs = window.getComputedStyle(el);
  const size = parseFloat(cs.fontSize) || 16;
  const line = parseFloat(cs.lineHeight) || size * 1.2;
  const pad = (parseFloat(cs.paddingTop) || 0) + (parseFloat(cs.paddingBottom) || 0);
  const border = (parseFloat(cs.borderTopWidth) || 0) + (parseFloat(cs.borderBottomWidth) || 0);
  const borderBox = cs.boxSizing === 'border-box';
  // `scrollHeight` mencakup padding tetapi bukan border.
  const content = el.scrollHeight - pad;
  const rows = Math.max(1, Math.min(MAX_ROWS, Math.round(content / line)));
  const capped = content > line * MAX_ROWS + 1;
  const visible = capped ? line * MAX_ROWS : Math.max(content, line);
  el.style.height = (borderBox ? visible + pad + border : visible) + 'px';
  el.style.overflowY = capped ? 'auto' : 'hidden';
  return rows;
}

if (!el.__autosize) {
  el.__autosize = true;
  el.addEventListener('input', fit);
  // Lebar jendela berubah -> baris terlipat berbeda -> tinggi ikut berubah.
  // Hanya lebar yang diamati: mengubah tinggi tidak boleh memicu dirinya sendiri.
  if (typeof window.ResizeObserver === 'function') {
    let lastWidth = el.clientWidth;
    new window.ResizeObserver(() => {
      if (el.clientWidth !== lastWidth) {
        lastWidth = el.clientWidth;
        fit();
      }
    }).observe(el);
  }
}
fit();
return true;
