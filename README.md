# tquilla.is-a.dev

Situs personal berbasis **Rust + Dioxus 0.7 (WASM)**: menu boot bergaya NixOS (`/`), jurnal GitHub (`/profile`), dan simulasi desktop OS di browser (`/deisktify`).

Live: <https://tquilla.is-a.dev>

## Struktur workspace

| Paket | Isi |
| --- | --- |
| `packages/components` | Semua UI bersama (routing, halaman, window manager, aplikasi dock, data JSON) |
| `packages/web` | Entry point web (launch, Tailwind, `PlatformServices` browser) |
| `packages/desktop` | Entry point desktop (⚠️ belum jalan, ditunda) |
| `packages/mobile` | Entry point Android (⚠️ belum jalan, ditunda) |

## Menjalankan

Semua perintah dijalankan di dalam dev shell Nix/devenv.

```sh
web-dev      # dx serve --package web --platform web
web-build    # dx bundle --package web --platform web --release
```

> Fokus saat ini: **build web saja**. Desktop dan mobile masih error dan sengaja ditunda.

Tes engine kalkulator (logika murni, tanpa browser): `cargo test -p components`.

---

# ✅ TODO

Centang `[x]` jika sudah selesai. Nomor mengacu pada hasil audit 2026-10-09.

## 🔴 Prioritas 1: bug dan risiko nyata

- [x] **#1** Listener drag window bocor: listener `document` kini dipasang sekali dan meneruskan event ke gesture yang aktif (`window.rs`, `WINDOW_MANAGER_JS`)
- [x] **#2** `/profile` tidak lagi menampilkan data sample seolah-olah data asli: ada banner "Offline snapshot" saat fallback (`journal/mod.rs`)
  - [ ] Masukkan source Cloudflare Worker (`worker/`) ke repo. Komentar di kode merujuk ke `worker/README.md`, tetapi foldernya tidak ada
- [x] **#3** API key Gemini dikirim lewat header `x-goog-api-key`, bukan query string (`ai.rs`, `platform::http::post_json_with_headers`)
  - [ ] Cek ulang nama model `gemini-3.5-flash` / `gemini-3.7-flash` masih valid di API
- [x] **#4** Breakpoint `xs` didefinisikan di `theme.css` (`--breakpoint-xs: 30rem`), supaya `xs:text-xs` di `home.rs` benar-benar bekerja

> Catatan: perubahan P1 dibuat tanpa compile di sandbox. Jalankan `web-dev` dan uji manual: buka/tutup window berkali-kali lalu drag, matikan jaringan di `/profile`, kirim satu pesan di jendela AI.

## 🟠 Prioritas 2: performa load awal

- [ ] **#5** Muat resource per fitur, bukan global (`app.rs`)
  - [ ] `hls.js` hanya saat Radio dibuka
  - [ ] Material Symbols hanya di route `/profile`
  - [ ] Self-host font dengan subset, kurangi bobot Playfair/Inter
- [ ] **#6** Profil release: tambah `codegen-units = 1`, `panic = "abort"`, `strip = true` di `[profile.release]`, lalu ukur ulang ukuran `.wasm`
- [ ] **#7** Hilangkan ketergantungan gambar GitHub saat runtime (favicon, avatar login, avatar `/profile`, ikon embience dari `raw.githubusercontent.com`): jadikan aset lokal
- [ ] **#8** Parse JSON sekali saja (`use_memo`/`OnceLock`) di `home.rs`, `browser.rs`, `radio.rs`, `embience.rs` (akan terganti oleh fetch dari database, lihat bagian Fullstack)
- [ ] **#9** Boot screen 5 detik: tambah tombol skip atau ikuti waktu load aset sebenarnya

## 🟡 Prioritas 3: SEO, aksesibilitas, UX

- [ ] **#10** Metadata: `description`, `og:*`, `lang`, `robots.txt`, `sitemap.xml`, `<noscript>`, `index.html` kustom di `packages/web`
- [ ] **#11** Samakan judul tab, dan perbaiki ejaan `deisktify` (URL, judul, komentar), beserta redirect dari URL lama
- [ ] **#12** Aksesibilitas
  - [ ] `alt` untuk 5 `img` yang belum punya
  - [ ] `tabindex: "{total}"` di `home.rs` seharusnya `0`
  - [ ] Daftar generasi memakai `button`/`a`, bukan `li` + `onclick`
  - [ ] Cek `prefers-reduced-motion` untuk animasi boot dan login
- [ ] **#13** Auto-redirect 10 detik di Home: beri cara membatalkan / berhenti saat ada interaksi
- [ ] **#14** Ganti link placeholder di `shared.rs` (`LINKEDIN_URL`, `DISCORD_URL`)
- [ ] **#15** Tentukan satu bahasa UI (Indonesia atau Inggris), atau buat modul string sederhana

## 🔵 Prioritas 4: kualitas kode dan CI

- [ ] **#16** CI: tambah job `cargo fmt --check`, `cargo clippy`, `cargo test` sebelum build
- [ ] **#17** Pin versi di CI (`dtolnay/rust-toolchain`, `cargo-binstall`) dan verifikasi checksum binary Tailwind, atau pakai `rust-toolchain.toml`
- [ ] **#18** Tambah test: `normalize_url`, `relative_time`, parsing bookmark, state window manager
- [ ] **#19** Pindahkan JS inline (`document::eval`, `WINDOW_MANAGER_JS`) ke file `.js` sebagai aset
- [ ] **#20** Hapus duplikasi `relative_time` (livechat dan `tty/chat_preview.rs`)
- [ ] **#21** Rapikan repo: dokumentasi (file ini), `AGENTS.md`, dan pastikan hanya satu `Dioxus.toml` yang dipakai

---

## 🗄️ Fullstack dan database

Tujuan: mengganti file JSON dummy di `packages/components/data/` dengan database sungguhan, dan mengubah project menjadi fullstack (Dioxus server functions).

### Keputusan yang perlu dikonfirmasi lebih dulu

GitHub Pages hanya menyajikan file statis, jadi **tidak bisa menjalankan server atau database**. Begitu ada backend, hosting dan workflow deploy harus berubah. Pilihan:

| Opsi | Cara kerja | Catatan |
| --- | --- | --- |
| **A. Dioxus fullstack (Axum) di satu host** | `#[server]` functions + DB di server yang sama (VPS, Fly.io, Railway, atau container Nix) | Satu codebase Rust, tanpa CORS. Perlu hosting yang menjalankan proses |
| **B. Frontend tetap di Pages + API terpisah** | WASM statis tetap di Pages, API (Rust/Axum atau Cloudflare Worker + D1/Turso) di host lain | Deploy sekarang hampir tidak berubah, tapi perlu CORS dan dua deploy |

Database: **SQLite lokal untuk development**, lalu **Postgres atau Turso (libSQL)** untuk produksi. Akses lewat `sqlx` dengan migrasi.

- [ ] **D0** Putuskan opsi hosting (A atau B) dan database produksi

### Rencana bertahap

- [ ] **D1** Fondasi fullstack
  - [ ] Tambah fitur `server` pada `components`/`web` dan jalankan lewat `dx serve --fullstack`
  - [ ] Pisahkan kode yang hanya untuk server dengan `#[cfg(feature = "server")]` (agar tidak ikut ke WASM)
  - [ ] Setup `sqlx` + folder `migrations/`, koneksi DB lewat env var (`DATABASE_URL`)
  - [ ] Update `devenv.nix` (sqlx-cli, sqlite/postgres) dan `flake.nix`
- [ ] **D2** Skema dan migrasi dari JSON ke tabel

  | File JSON (dummy) | Tabel | Kolom utama |
  | --- | --- | --- |
  | `generations.json` | `generations` | `number` (PK), `label`, `link`, `kernel`, `built_on` |
  | `browser.json` | `bookmarks` | `id`, `label`, `url`, `sort_order` |
  | `radio.json` | `radio_stations` | `id`, `label`, `slogan`, `stream_url`, `official_link`, `image_url`, `sort_order`, `is_active` |
  | `embience.json` | `embiences`, `embience_presets`, `embience_preset_items` | `id`, `name`, `icon_url`, `stream_url`; preset dan relasi ke embience |
  | `chat_sample.json` | `chat_messages` | `id`, `username`, `url`, `avatar_url`, `message`, `attachment_url`, `created_at` |
  | `journal.sample.json` | `journal_snapshots` | `payload` (JSON), `fetched_at`; pengganti cache Worker |

  - [ ] Script seed satu kali dari file JSON yang ada ke tabel
  - [ ] Hapus `include_str!` data JSON setelah fitur terkait pindah ke DB
- [ ] **D3** Server functions (read-only dulu): `list_generations`, `list_bookmarks`, `list_radio_stations`, `list_embiences`, `list_presets`; komponen memakai `use_server_future`/`use_resource` dengan state loading dan error
- [ ] **D4** LiveChat sungguhan (sekarang hanya seed lokal)
  - [ ] `list_messages` / `post_message` (+ polling atau SSE/WebSocket untuk pesan baru)
  - [ ] Validasi di server: panjang pesan, normalisasi URL (server tidak boleh percaya `normalize_url` dari klien), rate limit per IP
  - [ ] Avatar dan lampiran **tidak disimpan sebagai base64 di DB**: simpan di object storage (S3/R2) atau disk, dan simpan URL-nya saja
  - [ ] Moderasi dasar: laporkan/hapus pesan, daftar kata terlarang
- [ ] **D5** Jurnal GitHub: pindahkan logika Worker ke server (cron/background task yang mengisi `journal_snapshots`), hapus fallback ke sample, tampilkan `fetched_at` di UI
- [ ] **D6** Konten bisa diubah tanpa deploy ulang (halaman admin sederhana atau CLI) untuk radio, bookmark, embience
  - [ ] Autentikasi admin (jangan hardcode kredensial; password demo di `login.rs` hanya untuk animasi)
- [ ] **D7** Perbaiki data yang usang: URL stream radio mengandung token (`rj-tok`) yang bisa kedaluwarsa, validasi berkala dan tandai `is_active = false`
- [ ] **D8** Deploy dan operasi
  - [ ] Ganti workflow `deploy-page.yml` sesuai opsi hosting (Dockerfile / `nix build` / deploy ke host)
  - [ ] Backup database, health check, variabel rahasia di secret manager
  - [ ] Pertahankan fallback 404 untuk routing sisi klien jika memilih opsi B

---

## Lisensi

Lihat `LICENSE`.
