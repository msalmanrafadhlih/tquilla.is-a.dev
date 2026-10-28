// Mengisi ulang isi <input>/<textarea> dari kode (mengosongkan setelah kirim,
// mengisi riwayat perintah, dst). Dijalankan lewat `document::eval`; sebelum
// badan skrip ini ditulis:
//   ID      id elemen
//   VALUE   isi baru (string)
//   EXPECT  null, atau isi yang HARUS masih ada di elemen agar diganti
// Kenapa ada: kolom isian tidak lagi diikat ke signal (`value: "{signal}"`).
// Pada WebView desktop/seluler peristiwa `input` sampai ke Rust secara
// asinkron; ikatan dua arah menimpa DOM dengan nilai yang sudah usang dan
// menghilangkan ketikan yang lebih baru. Sekarang DOM yang jadi sumber
// kebenaran, dan Rust hanya mengubahnya lewat sini.
// EXPECT melindungi ketikan yang masuk setelah pengiriman: bila isi elemen
// sudah berbeda dari yang dikirim, elemen dibiarkan apa adanya.
// Mengembalikan true bila isi benar-benar diganti.

const el = document.getElementById(ID);
if (!el) return false;
if (EXPECT !== null && el.value !== EXPECT) return false;
el.value = VALUE;
if (typeof el.setSelectionRange === 'function') {
  try {
    el.setSelectionRange(VALUE.length, VALUE.length);
  } catch (_) {
    // Tipe input tertentu (mis. number) menolak setSelectionRange.
  }
}
return true;
