# tquilla.is-a.dev

Situs personal berbasis **Rust + Dioxus 0.7 (WASM)**: menu boot bergaya NixOS (`/`), jurnal GitHub (`/profile`), dan simulasi desktop OS di browser (`/deisktify`).

Live: <https://tquilla.is-a.dev>

## Struktur workspace

| Paket | Isi |
| --- | --- |
| `packages/components` | Semua UI bersama (routing, halaman, window manager, aplikasi dock, data JSON) |
| `packages/components/js/` | JS yang dijalankan lewat `document::eval` (window manager, audio/hls, scroll reveal, context menu) + test node di `js/tests/` |
| `packages/web` | Entry point web (launch, Tailwind, `PlatformServices` browser) |
| `packages/desktop` | Entry point desktop (⚠️ belum jalan, ditunda) |
| `packages/mobile` | Entry point Android (⚠️ belum jalan, ditunda) |
| `site/` | File statis untuk deploy: `robots.txt`, `sitemap.xml`, snippet `<head>` (SEO/Open Graph) dan `<noscript>` |
| `scripts/postbuild.py` | Langkah pasca-build di CI: menyuntik `site/*` ke output `dx bundle` (tes: `scripts/test_postbuild.py`) |
| `.github/workflows/` | `ci.yml` (fmt, clippy, test Rust, test node/python; jalan di tiap PR dan sebelum deploy) dan `deploy-page.yml` (build + deploy ke Pages) |
| `rust-toolchain.toml` | Versi Rust dikunci (1.97.0); naikkan bersama workflow |

## Menjalankan

Semua perintah dijalankan di dalam dev shell Nix/devenv.

```sh
web-dev      # dx serve --package web --platform web
web-build    # dx bundle --package web --platform web --release
```

> Fokus saat ini: **build web saja**. Desktop dan mobile masih error dan sengaja ditunda.

Cek yang sama dengan CI (semuanya harus hijau sebelum commit):

```sh
cargo fmt --all -- --check
cargo clippy -p components --all-targets -- -D warnings
cargo test -p components                                   # 50 test: engine kalkulator, window manager, parsing data, URL, waktu
node --test "packages/components/js/tests/*.test.js"       # 13 test JS dengan DOM tiruan
python3 -m unittest discover -s scripts -p "test_*.py"     # 6 test postbuild
```

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

> P1 sudah diuji manual dan berjalan baik (2026-10-09).

## 🟠 Prioritas 2: performa load awal

- [ ] **#5** Muat resource per fitur, bukan global (`app.rs`)
  - [x] `hls.js` dimuat lazy, hanya saat stream `.m3u8` pertama diputar (`AUDIO_JS` di `audio.rs`)
  - [x] Material Symbols, Inter, dan Playfair Display hanya dimuat di `/profile`; Playfair juga di kalkulator (tombol italik). Font global tinggal JetBrains Mono. Inter 600 dibuang (tidak dipakai)
  - [ ] Self-host font (woff2 + `@font-face` lokal) dengan subset latin, supaya tidak bergantung ke Google Fonts
- [x] **#6** Profil release: `debug = false`, `codegen-units = 1`, `panic = "abort"`, `strip = true` ditambahkan di `[profile.release]`. Ukuran `.wasm` sebelum/sesudah belum terukur, cek dengan `web-build` lalu bandingkan isi `target/dx/web/release/web/public/assets/*.wasm`
- [x] **#7** Hilangkan ketergantungan gambar GitHub saat runtime
  - [x] Favicon, avatar login, dan avatar `/profile` memakai `shared::AVATAR` (`assets/avatar.jpg`, gambar yang kamu kirim 2026-10-09). Ganti file itu kalau mau foto lain (tetap bernama `avatar.jpg`, atau ubah path di `shared.rs`)
  - [x] Ikon embience memakai aset lokal `logo-*.svg` (dicocokkan lewat nama file di JSON)
- [x] **#8** Parse JSON sekali per sesi (`OnceLock`) di `home.rs` (re-render tiap detik karena countdown dan tiap hover) dan `tty/chat_preview.rs` (re-render tiap 3 detik). `browser.rs`, `radio.rs`, `embience.rs`, dan `livechat.rs` sudah parse sekali per mount lewat `use_signal`/`use_hook`, jadi tidak perlu diubah (semuanya akan diganti fetch dari database)
- [x] **#9** Boot screen bisa di-skip: tombol "Skip ⏎" atau tekan tombol apa saja (`booting.rs`). Menyesuaikan durasi dengan waktu load aset sebenarnya belum dikerjakan

> Catatan: perubahan P2 dicek dengan `cargo check` + `cargo test` (native) dan uji JS dengan DOM tiruan, tetapi belum dijalankan di browser. Uji manual: (1) buka `/`, `/profile`, `/deisktify` dan cek tab Network, tidak ada request ke `cdn.jsdelivr.net` sebelum Radio diputar, dan `/` tidak memuat Inter/Playfair/Material Symbols; (2) putar stasiun HLS (mis. Jak FM) dan satu suara Embience; (3) skip boot dengan tombol dan dengan keyboard; (4) cek tombol italik kalkulator dan ikon Embience.

## 🟡 Prioritas 3: SEO, aksesibilitas, UX

- [x] **#10** Metadata
  - [x] `PageMeta` (`shared.rs`) di tiap route: judul, `description`, canonical, Open Graph, `twitter:card`
  - [x] Tag statis untuk crawler preview link (tidak menjalankan WASM): `site/head.html` disuntik ke `index.html` oleh `scripts/postbuild.py` di CI, lengkap dengan `<html lang="en">` dan `<noscript>`
  - [x] `robots.txt` dan `sitemap.xml` (`site/`), disalin ke output oleh `postbuild.py`
  - [ ] Cek hasil deploy: lihat sumber halaman live dan tes link di <https://opengraph.xyz> atau debugger Facebook/WhatsApp
  - [x] Kartu preview besar (`summary_large_image`): gambar 1200×630 ada di `site/og-image.jpg`, disalin ke root output oleh `postbuild.py` (URL tetap `/og-image.jpg`). Edit gambarnya kalau mau tampilan lain; ukuran dan nama file jangan diubah
- [x] **#11** Judul tab seragam ("`<Halaman>` | tquilla.is-a.dev"). Ejaan **`deisktify`** sengaja dipertahankan (URL, judul, komentar), jadi tidak ada rename atau redirect. Typo label di `generations.json` ("Github Jounal") diperbaiki jadi "GitHub Journal"
- [x] **#12** Aksesibilitas
  - [x] `alt=""` untuk 2 ikon dekoratif di livechat (hasil audit ulang: hanya 2 `img` yang belum punya `alt`, bukan 5)
  - [x] Daftar generasi memakai pola listbox ARIA (`role="listbox"`/`option`, `aria-activedescendant`); fokus keyboard pindah dari `<section>` ke `<ul>`, `tabindex: "{total}"` diganti `0`
  - [x] `prefers-reduced-motion` berlaku global (`base.css`); animasi boot dan login sudah punya aturan sendiri
  - [ ] Uji dengan screen reader (NVDA/VoiceOver) dan navigasi keyboard penuh
- [x] **#13** Auto-boot di Home dibatalkan oleh interaksi apa pun (tombol, hover, klik, sentuh); teks berubah jadi "Auto boot cancelled"
- [x] **#14** `LINKEDIN_URL` memakai profilmu dari `generations.json` (`linkedin.com/in/msalmanrafadhlih`). `DISCORD_URL` **sengaja** tetap invite komunitas "Motion IME" (keputusan 2026-10-09, jangan diganti)
- [x] **#15** Bahasa UI: **Inggris** untuk semua teks antarmuka (hint di Home sekarang berbahasa Inggris, sama seperti AI chat, login, dan banner). Pengecualian disengaja: **kalkulator tetap berbahasa Indonesia** (Aljabar/Trigonometri/Kalkulus, "Benar/Salah", koma desimal) karena memang dibuat dengan locale Indonesia. Ubah keputusan ini kalau mau kalkulator juga berbahasa Inggris (perlu menyesuaikan `engine/tests.rs`)

> Catatan: P3 dicek dengan `cargo check`, `cargo test`, dan uji `postbuild.py` pada `index.html` tiruan (termasuk dijalankan dua kali, dan kasus gagal). Belum dijalankan di browser. Uji manual: (1) `/`: biarkan 10 detik (redirect ke `/deisktify`), lalu ulangi dan tekan tombol apa saja (hitungan berhenti); navigasi ↑/↓/j/k/Enter; klik area kosong lalu tekan ↓; (2) cek judul tab tiap route; (3) setelah deploy: "view-source" dan cek `og:*`, `lang="en"`, `robots.txt`, `sitemap.xml`; (4) aktifkan "reduce motion" di OS dan cek boot, login, dan jendela.

## 🔵 Prioritas 4: kualitas kode dan CI

- [x] **#16** CI: `.github/workflows/ci.yml` menjalankan `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, test node, dan test postbuild. Jalan di tiap pull request, dan dipanggil `deploy-page.yml` (`needs: check`), jadi deploy tidak jalan kalau ada yang merah. Seluruh kode sudah diformat dengan `cargo fmt` (diff besar satu kali, sekali ini saja) dan 21 warning clippy dibereskan
  - [ ] Jalankan sekali di GitHub dan pastikan hijau. Workflow ini belum pernah dijalankan (sandbox tidak punya akses ke GitHub Actions)
  - [ ] Makro `rsx!` tidak diformat oleh `cargo fmt`; kalau mau seragam, jalankan `dx fmt` (belum dipakai di CI)
- [x] **#17** Pin versi
  - [x] `rust-toolchain.toml` + `dtolnay/rust-toolchain@1.97.0` (komponen rustfmt, clippy; target wasm32 di job build)
  - [x] Binary Tailwind v4.1.11: `curl -f`, dan checksum diverifikasi kalau variable repo `TAILWIND_SHA256` di-set; kalau belum, step memberi **warning berisi hash**. Tindakan kamu: jalankan deploy sekali, salin hash dari warning ke Settings → Secrets and variables → Actions → **Variables** → `TAILWIND_SHA256`
  - [ ] `cargo-bins/cargo-binstall@main` masih mengikuti branch `main` (tidak bisa saya verifikasi tag-nya dari sandbox). Ganti ke tag/commit SHA terbaru yang kamu percayai
- [x] **#18** Test: 50 test Rust (`normalize_url`, `relative_time_between`, parsing semua JSON data, link `generations.json` valid sebagai `Route`, state window manager, mapping ikon embience, engine kalkulator) + 13 test JS + 6 test postbuild. Logika state window kini fungsi murni (`open_or_focus_in`, `close_in`, `toggle_maximize_in`, ...) dengan wrapper `Signal` tipis
- [x] **#19** JS inline dipindah ke `packages/components/js/*.js` (`include_str!`, semantik `document::eval` tidak berubah), lengkap dengan test node. JS satu-dua baris di `livechat.rs` dan `geometry_sync_js` (berisi interpolasi id) sengaja dibiarkan inline
- [x] **#20** `relative_time` digabung ke `time.rs` (dipakai livechat dan `tty/chat_preview.rs`), dengan test
- [x] **#21** Rapikan repo: `AGENTS.md` kini diawali panduan khusus project, kedua `Dioxus.toml` diberi catatan "ubah bersamaan" (belum terverifikasi mana yang dibaca `dx`, jadi keduanya dipertahankan), README diperbarui
  - [ ] `.gitignore` berisi baris `/packages/components/src/nixdesk/mod.rs` ("activating splash screen"). File itu sudah di-track dan dipakai, jadi ignore tidak berpengaruh, tetapi file baru dengan path itu tidak akan masuk git. Hapus baris itu kalau memang tidak disengaja

> Bug yang ikut ketemu dan diperbaiki saat refactor: tombol brightness tidak berpengaruh karena overlay peredupnya tidak pernah dirender (`pages/interface/mod.rs`). Dicek dengan `cargo fmt`/`clippy`/`test` dan test JS, belum dijalankan di browser.

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
