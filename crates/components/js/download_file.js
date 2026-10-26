// Mengunduh teks sebagai berkas lewat browser (tanpa server). Dijalankan lewat
// `document::eval`; sebelum badan skrip ini ditulis:
//   FILENAME  nama berkas
//   CONTENT   isi berkas (string)
//   MIME      jenis isi, mis. "application/json"
// Mengembalikan true bila tautan unduhan sudah diklik.

const blob = new Blob([CONTENT], { type: MIME + ';charset=utf-8' });
const url = URL.createObjectURL(blob);
const link = document.createElement('a');
link.href = url;
link.download = FILENAME;
link.style.display = 'none';
document.body.appendChild(link);
link.click();
link.remove();
// Beri browser waktu memulai unduhan sebelum URL-nya dicabut.
setTimeout(() => URL.revokeObjectURL(url), 1000);
return true;
