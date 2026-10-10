# tquilla.is-a.dev

Situs personal berbasis **Rust + Dioxus 0.7 (WASM)**: menu boot bergaya NixOS (`/`), jurnal GitHub (`/profile`), dan simulasi desktop OS di browser (`/deisktify`).

Live: <https://tquilla.is-a.dev>

## Struktur workspace

| Paket | Isi |
| --- | --- |
| `packages/components` | Semua UI bersama (routing, halaman, window manager, aplikasi dock, data JSON) |
| `packages/components/js/` | JS yang dijalankan lewat `document::eval` (window manager, audio/hls, scroll reveal, context menu) + test node di `js/tests/` |
| `packages/web` | Entry point web (launch, Tailwind, `PlatformServices` browser) |
| `packages/desktop` | Entry point desktop (compile dan clippy lolos; belum diuji jalan, lihat "Platform desktop dan mobile") |
| `packages/mobile` | Entry point Android (compile dan clippy lolos di host; build Android belum diverifikasi) |
| `site/` | File statis untuk deploy: `robots.txt`, `sitemap.xml`, snippet `<head>` (SEO/Open Graph) dan `<noscript>` |
| `scripts/postbuild.py` | Langkah pasca-build di CI: menyuntik `site/*` ke output `dx bundle` (tes: `scripts/test_postbuild.py`) |
| `.github/workflows/` | `ci.yml` (fmt, clippy, test Rust, test node/python; jalan di tiap PR dan sebelum deploy), `deploy-page.yml` (build + deploy ke Pages), dan `platforms.yml` (cek compile desktop/mobile; tidak menahan deploy) |
| `rust-toolchain.toml` | Versi Rust dikunci (1.97.0); naikkan bersama workflow |

## Menjalankan

Semua perintah dijalankan di dalam dev shell Nix/devenv.

```sh
web-dev      # dx serve --package web --platform web
web-build    # dx bundle --package web --platform web --release
```

> Web adalah satu-satunya platform yang sudah diuji di runtime. Desktop dan mobile sudah bisa di-`cargo check` (lihat "Platform desktop dan mobile" di bawah); `desktop-dev` dan `android-dev` belum pernah diuji.

Cek yang sama dengan CI (semuanya harus hijau sebelum commit):

```sh
cargo fmt --all -- --check
cargo clippy -p components --all-targets -- -D warnings
cargo test -p components                                   # 53 test: engine kalkulator, window manager, parsing data, URL, waktu, format jam
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
  - [x] Run pertama gagal hanya di langkah `cargo fmt` (baris kosong ekstra di `packages/mobile/src/main.rs`); clippy dan test lulus. Diperbaiki dengan `cargo fmt --all`
  - [ ] Pastikan run berikutnya hijau sampai job `build` dan `deploy`
  - [ ] Makro `rsx!` tidak diformat oleh `cargo fmt`; kalau mau seragam, jalankan `dx fmt` (belum dipakai di CI)
- [x] **#17** Pin versi
  - [x] `rust-toolchain.toml` + `dtolnay/rust-toolchain@1.97.0` (komponen rustfmt, clippy; target wasm32 di job build)
  - [x] Binary Tailwind v4.1.11: `curl -f`, dan checksum diverifikasi kalau variable repo `TAILWIND_SHA256` di-set; kalau belum, step memberi **warning berisi hash**. Tindakan kamu: jalankan deploy sekali, salin hash dari warning ke Settings → Secrets and variables → Actions → **Variables** → `TAILWIND_SHA256`
  - [x] `cargo-bins/cargo-binstall` dipin ke tag `v1.25.2` (sebelumnya `@main`, yang berarti setiap build memakai commit terbaru branch itu). Naikkan tag-nya sengaja saat perlu
  - [x] Versi `dx` di CI disamakan dengan crate `dioxus` di `Cargo.lock` (0.7.10, bukan 0.7.1)
- [x] **#18** Test: 53 test Rust (`normalize_url`, `relative_time_between`, parsing semua JSON data, link `generations.json` valid sebagai `Route`, state window manager, mapping ikon embience, engine kalkulator) + 13 test JS + 6 test postbuild. Logika state window kini fungsi murni (`open_or_focus_in`, `close_in`, `toggle_maximize_in`, ...) dengan wrapper `Signal` tipis
- [x] **#19** JS inline dipindah ke `packages/components/js/*.js` (`include_str!`, semantik `document::eval` tidak berubah), lengkap dengan test node. JS satu-dua baris di `livechat.rs` dan `geometry_sync_js` (berisi interpolasi id) sengaja dibiarkan inline
- [x] **#20** `relative_time` digabung ke `time.rs` (dipakai livechat dan `tty/chat_preview.rs`), dengan test
- [x] **#21** Rapikan repo: `AGENTS.md` kini diawali panduan khusus project, kedua `Dioxus.toml` diberi catatan "ubah bersamaan" (belum terverifikasi mana yang dibaca `dx`, jadi keduanya dipertahankan), README diperbarui
  - [x] Baris `/packages/components/src/nixdesk/mod.rs` di `.gitignore` dihapus

> Bug yang ikut ketemu dan diperbaiki saat refactor: tombol brightness tidak berpengaruh karena overlay peredupnya tidak pernah dirender (`pages/interface/mod.rs`). Dicek dengan `cargo fmt`/`clippy`/`test` dan test JS, belum dijalankan di browser.

---

## 🔧 Tindak lanjut (2026-10-10)

- [x] CI: perbaikan di #16/#17/#21 di atas (fmt, pin `cargo-binstall`, versi `dx`, `.gitignore`)
- [x] Navbar: jam + tanggal kini **tombol** yang beralih 24 jam ↔ 12 jam (`nixdesk/clock.rs`: `ClockFormat`, `format_hm`; dipakai navbar, panel jam Terminal, dan perintah `date`). Default **24 jam**; ubah di `ClockFormat::default()`. Pilihan disimpan di context `Signal<ClockFormat>` (provider di `MainPage`), jadi Settings nanti tinggal memakai `use_clock_format()`. Belum disimpan ke `localStorage`, jadi kembali ke default saat halaman dimuat ulang
- [ ] Brightness: **ditunda**. Rencana: diganti popup window untuk theme/preferences yang tersambung ke aplikasi Settings. Overlay peredup (`pages/interface/mod.rs`) sudah ada, tinggal disambungkan
- [X] **Ukur ukuran wasm** (kerjaanmu, langkah di bawah)
  - update: wasm_compresed: size: 1.4mb gzip: 452141  brotli: 389603
  - update: wasm_uncompresed: size: 3.5mb gzip: 762299  brotli: 654407
- [ ] **Self-host font** (kerjaanmu, langkah di bawah)

### Mengukur ukuran wasm

```sh
# 1. Build produksi persis seperti CI
web-build        # = dx bundle --package web --platform web --release
W=target/dx/web/release/web/public/assets
ls -l $W/*.wasm                                   # ukuran mentah
for f in $W/*.wasm; do
  echo "$f  gzip: $(gzip -9 -c "$f" | wc -c)  brotli: $(brotli -9 -c "$f" | wc -c)"
done                                              # ukuran yang benar-benar diunduh pengunjung (GitHub Pages memakai gzip)
```

Untuk membandingkan sebelum/sesudah `[profile.release]` (isi: `opt-level = "z"`, `lto`, `codegen-units = 1`, `panic = "abort"`, `strip`): catat angka di atas, ganti sementara blok `[profile.release]` di `Cargo.toml` dengan `[profile.release]` kosong, `cargo clean -p web -p components`, build ulang, catat lagi, lalu kembalikan. Hasil yang masuk akal dicatat di README: ukuran mentah dan gzip.

Kalau mau tahu isi yang paling besar: `cargo install twiggy`, lalu `twiggy top -n 25 $W/*.wasm`. Build berbeda (`debug = false`, `strip = true`) menghapus nama fungsi, jadi untuk analisis build dengan `strip = false` dulu.

### Self-host font

Font yang dipakai sekarang (semuanya lewat Google Fonts):

| Font | Dipakai di | Berat/gaya | Catatan |
| --- | --- | --- | --- |
| JetBrains Mono | semua route (`app.rs`) | 400 | |
| Inter | `/profile` (`journal/mod.rs`) | 300, 400, 500 | |
| Playfair Display | `/profile` dan tombol italik kalkulator | 400, 700, dan italik 400, 700 | |
| Material Symbols Outlined | `/profile` | enam ikon: `bolt`, `code`, `public`, `star`, `timeline`, `trending_up` | file penuhnya sangat besar; lihat opsi di bawah |

**Unduh (sumber yang sama dengan Google Fonts, sudah dipecah per subset, format woff2):**

- [JetBrains Mono di Fontsource](https://fontsource.org/fonts/jetbrains-mono/cdn), [Inter](https://fontsource.org/fonts/inter/cdn), [Playfair Display](https://fontsource.org/fonts/playfair-display/cdn). Lewat npm, di folder sementara di luar repo:

  ```sh
  mkdir /tmp/fonts && cd /tmp/fonts && npm init -y >/dev/null
  npm i @fontsource/jetbrains-mono @fontsource/inter @fontsource/playfair-display
  F=node_modules/@fontsource
  D=<repo>/packages/components/assets/fonts && mkdir -p $D
  cp $F/jetbrains-mono/files/jetbrains-mono-latin-400-normal.woff2 $D/
  cp $F/inter/files/inter-latin-{300,400,500}-normal.woff2 $D/
  cp $F/playfair-display/files/playfair-display-latin-{400,700}-{normal,italic}.woff2 $D/   # {normal,italic} untuk 400 dan 700
  ```

  Cek nama file yang sebenarnya dengan `ls $F/<nama>/files | grep latin-`. Hanya subset `latin` yang perlu; teks situs ini berbahasa Inggris dan Indonesia.
- Alternatif GUI: [JetBrains Mono (rilis resmi, ada folder webfonts)](https://github.com/JetBrains/JetBrainsMono).

**Memasang:** hapus `FONTS_URL`/`INTER_FONT_URL`/`PLAYFAIR_FONT_URL` dan dua `preconnect` ke Google di `app.rs`, lalu deklarasikan `@font-face` dari aset lokal. Taruh CSS-nya lewat `document::Style` supaya URL berhash dari `asset!` ikut masuk (URL relatif di dalam file `.css` tidak dijamin ditulis ulang oleh bundler):

```rust
const JB_400: Asset = asset!("/assets/fonts/jetbrains-mono-latin-400-normal.woff2");

fn font_face(family: &str, weight: u16, style: &str, file: &Asset) -> String {
    format!(
        "@font-face{{font-family:'{family}';font-weight:{weight};font-style:{style};\
         font-display:swap;src:url({file}) format('woff2');}}"
    )
}
// di rsx!: document::Style { {font_face("JetBrains Mono", 400, "normal", &JB_400)} }
// + document::Link { rel: "preload", href: JB_400, r#as: "font", r#type: "font/woff2", crossorigin: "" }  // hanya font utama
```

Lalu pindahkan pemanggilan Inter/Playfair ke komponen yang memakainya (seperti sekarang: `journal/mod.rs`, `calculator/mod.rs`). Cek di tab Network: tidak ada lagi request ke `fonts.googleapis.com` / `fonts.gstatic.com`.

**Material Symbols (pilih salah satu):**

1. *Paling mudah:* tetap pakai Google tetapi hanya enam ikon, tambahkan `&icon_names=bolt,code,public,star,timeline,trending_up` ke `MATERIAL_SYMBOLS_URL` (nama ikon **harus berurutan alfabet**, lihat [panduan Material Symbols](https://developers.google.com/fonts/docs/material_symbols)). Ukuran turun dari ratusan KB ke beberapa KB. Belum saya uji (sandbox tidak bisa menjangkau Google Fonts).
2. *Tanpa font sama sekali:* ganti enam ikon itu dengan SVG inline. Enam saja, jadi ringan dan tanpa request tambahan.
3. Self-host font penuh tidak disarankan: file variabelnya besar dan kamu juga perlu menulis sendiri CSS kelas `.material-symbols-outlined` (ligature).

---

## 🖥️ Platform desktop dan mobile

Diagnosis (2026-10-10), `packages/desktop` dan `packages/mobile`:

- **Penyebab error yang terkonfirmasi:** kedua `main.rs` memakai `asset!("/assets/tailwind.css")`, tetapi file itu hasil generate Tailwind dan di-`.gitignore`, jadi tidak ada di checkout bersih. `cargo check` gagal dengan `Asset at /assets/tailwind.css doesn't exist`. Kode Rust-nya sendiri sehat: dengan file itu ada, `cargo check` dan `clippy -D warnings` lolos di Linux (rustc 1.97.0, `--locked`).
- **Syarat di luar kode:** `cargo check -p desktop`/`-p mobile` di host Linux butuh library sistem webview (webkit2gtk-4.1, GTK3, libxdo). Di Nix sudah disediakan `devenv.nix`; di CI lihat `platforms.yml`.

- [x] Diagnosis di atas
- [x] `build.rs` di `packages/desktop` dan `packages/mobile`: membuat `assets/tailwind.css` kosong kalau belum ada (tidak pernah menimpa file yang ada). `dx`, CI, dan `nix build` tetap menimpanya dengan CSS asli. `packages/web` sengaja **tidak** diubah supaya jalur deploy yang berjalan tidak tersentuh
- [x] `.github/workflows/platforms.yml`: `cargo check` + `clippy -D warnings` untuk desktop dan mobile. Workflow **terpisah** dari `ci.yml`, jadi tidak menahan deploy Pages
- [ ] Uji manual `desktop-dev` (`dx serve --package desktop --platform desktop`): window terbuka, CSS termuat, jam, window manager, kalkulator, Radio/Embience (audio di webview GTK butuh GStreamer), AI chat (HTTP lewat `reqwest`)
- [ ] Uji manual `android-dev` / `android-build`: belum pernah diverifikasi sama sekali (sandbox tidak punya NDK dan tidak bisa mengunduh target Android). Hal yang paling mungkin bermasalah: `reqwest` + `rustls` (kompilasi `ring` dengan NDK), `[bundle]` identifier di `Dioxus.toml`, izin `INTERNET`
- [ ] Mobile: `open_external` masih menavigasi webview itu sendiri (lihat TODO di `packages/mobile/src/main.rs`); ganti dengan `Intent.ACTION_VIEW`
- [ ] Setelah D3: URL API untuk desktop/mobile (di web memakai variable build, lihat D1)

---

## 🗄️ Fullstack dan database

Tujuan: mengganti file JSON dummy di `packages/components/data/` dengan database sungguhan lewat API terpisah.

### Keputusan (D0, 2026-10-10)

| Hal | Keputusan |
| --- | --- |
| Hosting | **Opsi B**: frontend tetap di GitHub Pages (workflow, `CNAME`, `404.html`, `postbuild.py` tidak berubah) + API terpisah |
| Runtime API | **Axum** (Rust) di **Cloudflare Workers** lewat `workers-rs` (fitur `http`). Bukan Cloudflare Containers |
| Database | **Turso (libSQL)** untuk produksi; SQLite lokal untuk development (`turso dev`, protokol HTTP yang sama) |
| Repo | **Dua repo**: ini (frontend, GitHub Actions ke Pages) dan repo backend baru, **publik** (nama sementara `tquilla-api`), deploy ke Cloudflare lewat Actions miliknya sendiri |
| Klien ↔ server | **REST + JSON** lewat `platform::http`. Tanpa `#[server]` dan tanpa `dx serve --fullstack`, karena server function Dioxus butuh binary server Dioxus |
| Tipe bersama | crate `api-types` (struct serde, batas validasi, `normalize_url`) hidup di repo backend; frontend memakainya lewat git dependency dengan **tag** |
| Migrasi | file `.sql` biasa di repo backend, dijalankan lewat Turso CLI. `sqlx` tidak dipakai: tidak bisa compile ke wasm32 dan tidak bicara ke Turso |
| Domain API | `*.workers.dev` dulu. Belum dipastikan apakah `is-a.dev` mengizinkan `api.tquilla.is-a.dev` |
| CORS | hanya origin `https://tquilla.is-a.dev`; `localhost` hanya lewat env var di development |

```text
Browser / Desktop / Android  (Dioxus, repo ini)
        │  REST + JSON, tipe dari crate api-types
        ▼
Cloudflare Worker  (Axum via workers-rs, repo tquilla-api)
        │  HTTP ke Turso            │ R2 untuk lampiran (D4)
        ▼                           ▼
Turso (libSQL)               Cron Trigger (D5, D7)
```

Rencana isi repo backend: `crates/api-types` (dites native), `crates/api` (Worker, hanya wasm32), `migrations/*.sql`, `wrangler.toml`, `.github/workflows/{ci,deploy}.yml`.

### Risiko dan yang belum terverifikasi

- `workers-rs` sendiri mengaku "rough edges" ([README](https://github.com/cloudflare/workers-rs)). Tokio dan semua crate native tidak bisa dipakai; semua dependensi harus compile ke `wasm32-unknown-unknown`.
- Klien Rust Turso untuk Workers: `libsql-client` (backend Workers) sudah deprecated, dan dukungan wasm32 crate `libsql` belum terkonfirmasi. Rencananya klien HTTP kecil sendiri (`worker::Fetch`) ke API HTTP Turso. Bahwa `turso dev` memakai protokol yang sama belum diverifikasi.
- Batas plan gratis Workers (ukuran wasm, waktu CPU per request) harus dicek di dokumentasi Cloudflare sebelum D4. Kalau terlampaui, jalur cadangannya Workers Paid atau Cloudflare Containers ($5/bulan, tanpa free tier).
- Sandbox tidak bisa membangun wasm32 maupun menjangkau Cloudflare/Turso: semua yang menyentuh keduanya baru terbukti setelah Anda mengujinya sendiri.

### Rencana bertahap

- [x] **D0** Putuskan opsi hosting dan database produksi (lihat tabel di atas)
- [ ] **D1** Fondasi
  - [ ] Repo backend: workspace Cargo, `crates/api-types`, Worker Axum dengan `GET /api/v1/health`, klien Turso, `wrangler.toml`, CI (fmt, clippy, test native, cek wasm32)
  - [ ] Dev lokal: `turso dev` + `wrangler dev`; `.dev.vars.example` (tanpa kredensial asli)
  - [ ] Repo ini: modul `api` di `components` (base URL dari `option_env!("API_BASE_URL")`, tipe error, fungsi `get`/`post` bertipe di atas `platform::http`), `api-types` sebagai git dependency; variable build `API_BASE_URL` di `deploy-page.yml`
  - [ ] Update `devenv.nix` dan `flake.nix` (turso-cli, wrangler)
- [ ] **L1** Bahasa UI (tepat setelah D1)
  - [ ] Modul `languages/language.rs` di `components`: semua teks UI default **Inggris** terkumpul di satu tempat
  - [ ] Kalkulator ikut Inggris (label mode, `Benar/Salah` → `True/False`, pemisah desimal); sesuaikan `engine/tests.rs`
  - [ ] Rancangan agar nanti bisa multi-bahasa dari preferences di Settings (`Signal<Language>` lewat context, seperti `ClockFormat`)
- [ ] **D2** Skema dan migrasi dari JSON ke tabel

  | File JSON (dummy) | Tabel | Kolom utama |
  | --- | --- | --- |
  | `generations.json` | `generations` | `number` (PK), `label`, `link`, `kernel`, `built_on` |
  | `browser.json` | `bookmarks` | `id`, `label`, `url`, `sort_order` |
  | `radio.json` | `radio_stations` | `id`, `label`, `slogan`, `stream_url`, `official_link`, `image_url`, `sort_order`, `is_active` |
  | `embience.json` | `embiences`, `embience_presets`, `embience_preset_items` | `id`, `name`, `icon_url`, `stream_url`; preset dan relasi ke embience |
  | `chat_sample.json` | `chat_messages` | `id`, `username`, `url`, `avatar_url`, `message`, `attachment_url`, `created_at` |
  | `journal.sample.json` | `journal_snapshots` | `payload` (JSON), `fetched_at`; pengganti cache Worker |

  - [ ] Migrasi `.sql` di repo backend
  - [ ] Script seed satu kali dari file JSON yang ada (menghasilkan SQL)
  - [ ] Hapus `include_str!` data JSON setelah fitur terkait pindah ke API
- [ ] **D3** Endpoint baca (read-only dulu): `GET /api/v1/{generations,bookmarks,radio,embiences,presets}`; komponen memakai `use_resource` dengan state loading dan error
- [ ] **D4** LiveChat sungguhan (sekarang hanya seed lokal)
  - [ ] `GET/POST /api/v1/messages` (+ polling untuk pesan baru)
  - [ ] Validasi di server: panjang pesan, normalisasi URL ulang di server (server tidak boleh percaya `normalize_url` dari klien), rate limit per IP (`CF-Connecting-IP`)
  - [ ] Avatar dan lampiran **tidak disimpan sebagai base64 di DB**: simpan di Cloudflare R2, DB hanya menyimpan URL
  - [ ] Moderasi dasar: laporkan/hapus pesan, daftar kata terlarang
- [ ] **D5** Jurnal GitHub: pindahkan logika Worker lama ke Cron Trigger yang mengisi `journal_snapshots`, hapus fallback ke sample, tampilkan `fetched_at` di UI. Source Worker lama (`worker/`) tidak ada di repo; perlu dikirim
- [ ] **D6** Konten bisa diubah tanpa deploy ulang (endpoint admin atau CLI) untuk radio, bookmark, embience
  - [ ] Autentikasi admin lewat secret Cloudflare (jangan hardcode kredensial; password demo di `login.rs` hanya untuk animasi)
- [ ] **D7** Perbaiki data yang usang: URL stream radio mengandung token (`rj-tok`) yang bisa kedaluwarsa; Cron Trigger memvalidasi berkala dan menandai `is_active = false`
- [ ] **D8** Deploy dan operasi
  - [ ] Workflow `wrangler deploy` di repo backend (`CLOUDFLARE_API_TOKEN` sebagai secret Actions, token Turso lewat `wrangler secret`)
  - [ ] Backup database (dump terjadwal), health check, rotasi secret
  - [ ] Frontend: deploy Pages tidak berubah; `404.html` fallback tetap

---

## Lisensi

Lihat `LICENSE`.
