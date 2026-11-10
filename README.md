# tquilla.is-a.dev

Situs personal **Moch** yang ditulis seluruhnya dengan **Rust + [Dioxus](https://dioxuslabs.com) 0.7** dan dijalankan di browser sebagai WebAssembly. Live: <https://tquilla.is-a.dev>

Tampilannya bertema NixOS, dengan tiga halaman:

| Rute | Isi |
| --- | --- |
| `/` | Menu boot bergaya bootloader NixOS (daftar "generasi" yang bisa dipilih dengan keyboard) |
| `/profile` | Jurnal GitHub: statistik, repo sematan, grafik aktivitas, dan stack |
| `/deisktify` | Simulasi desktop OS di browser: layar boot, login, dock, window manager, dan aplikasi (File Manager, Calculator, Browser, Radio, Embience, Guestbook, AI Assistant, Settings, About, serta terminal TTY) |

Ejaan `deisktify` disengaja.

Beberapa hal yang menarik di dalamnya: kalkulator dengan mode Algebra, Trigonometry, dan Calculus; pemutar Radio dan suara latar (Embience); buku tamu yang merender Markdown lewat parser sendiri tanpa pernah membuat string HTML; dan AI Assistant berbasis Gemini dengan kunci API milik pengguna sendiri (riwayat chat hanya tersimpan di IndexedDB browser, tidak dikirim ke server mana pun).

Data dinamis (generasi, bookmark, radio, Embience, Guestbook, jurnal) datang dari backend terpisah, `tquilla-api` (Axum di Cloudflare Workers + Turso), lewat REST + JSON. Tanpa backend, halaman tetap tampil tetapi jendela yang butuh data menampilkan "The server address is not configured".

## Teknologi dan struktur

Rust 1.97.0 (dikunci di `rust-toolchain.toml`), Dioxus 0.7, Tailwind CSS v4, dan `dx` (Dioxus CLI) untuk build dan dev server.

| Bagian | Isi |
| --- | --- |
| `crates/components` | Semua UI bersama: routing, halaman, window manager, aplikasi, klien API, dan JS kecil di `js/` |
| `crates/components/assets/styles` | Semua CSS: `global.css` (token, reset) + satu file per UI; class di `rsx!` berupa nama semantik BEM yang diisi Tailwind lewat `@apply` (lihat `styles/README.md`) |
| `crates/markdown` | Mesin Markdown untuk Guestbook (Rust murni, tanpa Dioxus) |
| `platform/web` | Entry point web (yang dideploy ke GitHub Pages) |
| `platform/desktop`, `platform/mobile` | Entry point desktop dan Android (web adalah platform yang paling teruji) |
| `site/`, `scripts/` | Berkas statis SEO dan skrip pasca-build/tes |
| `docs/alur-project.md` | Daftar TODO, catatan verifikasi, keputusan fullstack, dan rencana bertahap |

## Prasyarat

Cara termudah adalah **Nix**: `flake.nix` + `devenv.nix` menyediakan Rust (dengan target wasm32), `dx`, Tailwind v4, Node, dan library webview untuk desktop.

```sh
nix develop --impure     # atau: direnv allow
```

Tanpa Nix, pasang: Rust 1.97.0 + target `wasm32-unknown-unknown`, `dioxus-cli` versi 0.7.x yang sama dengan `Cargo.lock`, Tailwind CSS v4 (`tailwindcss` harus v4, bukan v3), Node, dan Python 3.

## Menjalankan dan menguji secara lokal

Untuk menguji sebagai satu kesatuan, jalankan **backend dulu**, baru frontend.

### 1. Backend lokal

Di repo `tquilla-api` (dev shell-nya sendiri):

```sh
dev-local
```

Tunggu sampai muncul `Ready on http://127.0.0.1:8787`. Perintah ini menyiapkan database lokal berisi data contoh dan mengizinkan origin `127.0.0.1:8080` / `localhost:8080` (CORS). Detailnya ada di README repo backend.

### 2. Frontend

Di folder repo ini (dev shell):

```sh
web-dev-local
```

`web-dev-local` menjalankan `dx serve --package web --platform web` dengan `API_BASE_URL=http://127.0.0.1:8787`. Build pertama memakan waktu sekitar 3 menit; berikutnya inkremental dan ada hot reload. Setelah muncul `Serving web` dengan alamat `http://127.0.0.1:8080`, buka alamat itu di browser.

Tanpa Nix, perintah setara:

```sh
API_BASE_URL=http://127.0.0.1:8787 dx serve --package web --platform web
```

`API_BASE_URL` dibakar saat **build** (bukan dibaca saat jalan). Hanya `http(s)://host[:port]` tanpa path yang diterima; nilai kosong atau salah berarti "not configured". Mengubahnya butuh build ulang.

### 3. Yang perlu dicek di browser

1. `/`: menu boot tampil, daftar generasi terisi dari backend, navigasi dengan ↑/↓/j/k dan Enter berfungsi. Biarkan 10 detik tanpa menyentuh apa pun: otomatis masuk ke `/deisktify`.
2. `/profile`: data jurnal muncul (dari data contoh backend).
3. `/deisktify`: lewati layar boot dan login, lalu buka:
   - **Guestbook**: ada entri contoh. Kirim pesan Markdown, tekan Refresh, laporkan pesan, lalu sembunyikan dari dashboard admin backend (`http://127.0.0.1:8787/admin/`) dan Refresh lagi.
   - **Radio**, **Embience**, dan **Browser**: daftarnya terisi dari database.
   - **Calculator**, **File Manager**, **Settings**, terminal TTY, dan menyeret/menutup jendela.
   - **AI Assistant** (opsional): butuh kunci API Gemini milik sendiri; riwayat tersimpan di browser.
4. Tab Network di devtools: permintaan ke `http://127.0.0.1:8787/api/v1/...` berstatus 200 tanpa error CORS.

### Tanpa backend

`web-dev` (tanpa `API_BASE_URL`) tetap berjalan untuk mengerjakan tampilan; jendela yang butuh data akan menampilkan "The server address is not configured".

### Desktop dan Android

`API_BASE_URL` wajib diberikan saat build, mis. `API_BASE_URL=http://127.0.0.1:8787 desktop-dev`. Desktop (`desktop-dev`) sudah pernah dijalankan; Android (`android-dev`) belum teruji di perangkat. Keduanya belum dicoba terhadap backend lokal.

## Pengujian otomatis

Sama dengan CI (semuanya harus hijau sebelum commit):

```sh
cargo fmt --all -- --check
cargo clippy -p components -p markdown --all-targets -- -D warnings
cargo test -p components -p markdown
node --test "crates/components/js/tests/*.test.js"
python3 -m unittest discover -s scripts -p "test_*.py"
```

Target `wasm32` tidak dikompilasi oleh perintah di atas; kode khusus browser hanya terkompilasi lewat `dx`.

## Build dan deploy

```sh
web-build        # dx bundle --package web --platform web --release
```

Deploy ke GitHub Pages hanya lewat tag `generation_*` (atau tombol *Run workflow*), dengan variable repo `API_BASE_URL` berisi alamat Worker produksi. Rilis aplikasi desktop dan Android lewat tag `app-v*`. Push biasa ke `main` tidak men-deploy.

## Pemecahan masalah

| Gejala | Penyebab dan solusi |
| --- | --- |
| "The server address is not configured" | Dijalankan dengan `web-dev`; pakai `web-dev-local` (atau isi `API_BASE_URL`) dan build ulang |
| "Could not reach the server." atau error CORS di console | Backend belum jalan, atau origin di tab browser tidak ada di `ALLOWED_ORIGINS` backend (`crates/api/.dev.vars`). `dx` memakai port 8080; kalau berpindah port, tambahkan origin barunya dan mulai ulang backend |
| Port 8080 terpakai, `dx` memilih port lain | Bebaskan 8080 (port 8081 dipakai database backend) |
| Daftar kosong padahal backend jalan | Database belum di-seed: jalankan ulang `dev-local` di repo backend |
| `wasm-bindgen version mismatch` | Versi `dx` tidak cocok dengan `Cargo.lock`; lihat <https://wiki.nixos.org/wiki/Dioxus> |
| Gaya hilang di desktop/mobile | `assets/tailwind.css` belum dibuat; `dx serve/bundle` membuatnya otomatis |
| Build pertama sangat lama | Wajar (kompilasi ke WASM, ±3 menit); build berikutnya inkremental |

## Dokumentasi lain

- [docs/alur-project.md](docs/alur-project.md): TODO per prioritas, catatan verifikasi, keputusan fullstack (D0–D8), dan rencana bertahap.
- [AGENTS.md](AGENTS.md): aturan kerja khusus project ini (teks UI, JS, keamanan Markdown, dua `Dioxus.toml`, dll.).

## Lisensi

MIT, lihat [LICENSE](LICENSE).
