# CSS komponen

Semua gaya ada di folder ini. Entry Tailwind tiap platform (`platform/*/tailwind.css`) hanya
mengimpor `index.css`, yang mengimpor `global.css` lebih dulu lalu satu file per UI.

| File | Isi |
| --- | --- |
| `global.css` | `@theme`, `:root` (palet, gradasi, easing, z-index), reset, font, blok bangunan bersama (`app-window__*`, `visually-hidden`) |
| `booting.css`, `login.css`, `home.css`, `journal.css`, ... | satu file per layar/aplikasi |
| `window.css` | bingkai jendela (animasi buka/minimize/tutup, popup di `md`+) |

## Aturan

1. **Class di `rsx!` adalah nama semantik BEM** (`blok__elemen--modifier`), bukan utility Tailwind.
   Utility ditulis di CSS dengan `@apply`:

   ```css
   @layer components {
     .login__hint { @apply text-2xs text-fg-secondary md:text-xs; }
   }
   ```
2. **Nilai yang dipakai ulang hidup di `global.css`**: warna lewat `:root` (`--fg-main`, ...) dan
   alias utility `bg-bg-main`, `text-fg-main`, `border-fg-secondary` (`@theme inline`); ukuran font
   kecil `text-3xs` (10px) / `text-2xs` (11px); easing `--ease-out-expo` dst. Jangan menulis ulang
   `#555` atau `cubic-bezier(...)` di file UI.
3. **Tailwind tidak memindai berkas Rust** (`source(none)`): utility yang ditulis langsung di `rsx!`
   tidak menghasilkan CSS. `scripts/test_css_classes.py` menangkapnya di CI.
4. **Modifier = class tambahan, bukan `style:`**. Gunakan `class: if aktif { "blok blok--aktif" } else { "blok" }`.
   `style:` hanya untuk nilai yang datang dari data (lebar bar `%`, `--win-x`, `animation-delay`, warna dari API).
5. Urutan penting di dalam satu file: modifier harus ditulis SETELAH class dasarnya (sama spesifisitas,
   yang terakhir menang). Token berbreakpoint di class dasar (`md:items-center`) bisa kalah dari token
   polos di modifier (`items-end`): taruh keduanya di modifier yang sama.
6. Penanda yang sengaja tetap sebagai class biasa: `group` / `peer` (dibutuhkan variant
   `group-hover:` / `peer-focus-visible:`), `material-symbols-outlined` (font ikon eksternal),
   `win-frame` / `win-maximized` / `win-minimizing` / `win-closing` (dibaca `window_manager.js` dan
   `animationend`), `is-visible` (ditambah `scroll_reveal.js`).
7. Tambah file UI baru: buat `styles/<nama>.css`, lalu daftarkan di `index.css` (tes `test_tailwind_paths.py`
   mengingatkan kalau lupa).
