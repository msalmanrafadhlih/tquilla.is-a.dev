# tquilla.is-a.dev

Situs personal berbasis **Rust + Dioxus 0.7 (WASM)**: menu boot bergaya NixOS (`/`), jurnal GitHub (`/profile`), dan simulasi desktop OS di browser (`/deisktify`).

Live: <https://tquilla.is-a.dev>

## Struktur workspace

| Paket | Isi |
| --- | --- |
| `crates/components` | Semua UI bersama (routing, halaman, window manager, aplikasi dock, data JSON), termasuk `markdown_view.rs` yang menggambar pohon Markdown jadi komponen Dioxus |
| `crates/markdown` | Mesin Markdown ("readme engine"): Rust murni, **tanpa Dioxus dan tanpa teks UI**. `lib.rs` (parser + daftar putih + pohon `Block`/`Inline`) dan `html.rs` (tokenizer tag). Dites native tanpa mengompilasi Dioxus |
| `crates/components/js/` | JS yang dijalankan lewat `document::eval` (window manager, audio/hls, scroll reveal, context menu) + test node di `js/tests/` |
| `platform/web` | Entry point web (launch, Tailwind, `PlatformServices` browser) |
| `platform/desktop` | Entry point desktop (compile dan clippy lolos; belum diuji jalan, lihat "Platform desktop dan mobile") |
| `platform/mobile` | Entry point Android (compile dan clippy lolos di host; build Android gagal di percobaan 1 karena SDK, perbaikan belum diuji) |
| `site/` | File statis untuk deploy: `robots.txt`, `sitemap.xml`, snippet `<head>` (SEO/Open Graph) dan `<noscript>` |
| `scripts/postbuild.py` | Langkah pasca-build di CI: menyuntik `site/*` ke output `dx bundle` (tes: `scripts/test_postbuild.py`) |
| `.github/workflows/` | `ci.yml` (fmt, clippy, test Rust, test node/python; jalan di tiap PR dan sebelum deploy), `deploy-page.yml` (build + deploy ke Pages; jalan saat tag `generation_*` di-push atau lewat tombol manual, bukan tiap push), dan `platforms.yml` (cek compile desktop/mobile; tidak menahan deploy) |
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
cargo clippy -p components -p markdown --all-targets -- -D warnings
cargo test -p components -p markdown                       # 146 test: 92 di components (engine kalkulator, window manager, parsing data, URL, API, waktu, guestbook, teks UI) + 54 di markdown
node --test "crates/components/js/tests/*.test.js"       # 27 test JS dengan DOM tiruan
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
- [x] **#8** Parse JSON sekali per sesi (`OnceLock`) di `home.rs` (re-render tiap detik karena countdown dan tiap hover) dan `tty/chat_preview.rs` (re-render tiap 3 detik). `browser.rs`, `radio.rs`, `embience.rs`, dan `guestbook.rs` (dulu `livechat.rs`) sudah parse sekali per mount lewat `use_signal`/`use_hook`, jadi tidak perlu diubah (semuanya akan diganti fetch dari database)
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
  - [x] `alt=""` untuk 2 ikon dekoratif di Live Chat (kini Guestbook) (hasil audit ulang: hanya 2 `img` yang belum punya `alt`, bukan 5)
  - [x] Daftar generasi memakai pola listbox ARIA (`role="listbox"`/`option`, `aria-activedescendant`); fokus keyboard pindah dari `<section>` ke `<ul>`, `tabindex: "{total}"` diganti `0`
  - [x] `prefers-reduced-motion` berlaku global (`base.css`); animasi boot dan login sudah punya aturan sendiri
  - [ ] Uji dengan screen reader (NVDA/VoiceOver) dan navigasi keyboard penuh
- [x] **#13** Auto-boot di Home dibatalkan oleh interaksi apa pun (tombol, hover, klik, sentuh); teks berubah jadi "Auto boot cancelled"
- [x] **#14** `LINKEDIN_URL` memakai profilmu dari `generations.json` (`linkedin.com/in/msalmanrafadhlih`). `DISCORD_URL` **sengaja** tetap invite komunitas "Motion IME" (keputusan 2026-10-09, jangan diganti)
- [x] **#15** Bahasa UI: **Inggris** untuk semua teks antarmuka (hint di Home sekarang berbahasa Inggris, sama seperti AI chat, login, dan banner). Kalkulator ikut Inggris sejak L1 (Algebra/Trigonometry/Calculus, True/False, titik desimal)

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
- [x] **#19** JS inline dipindah ke `crates/components/js/*.js` (`include_str!`, semantik `document::eval` tidak berubah), lengkap dengan test node. JS satu-dua baris di Live Chat (kini diganti `guestbook_editor.js`) dan `geometry_sync_js` (berisi interpolasi id) sengaja dibiarkan inline
- [x] **#20** `relative_time` digabung ke `time.rs` (dipakai Guestbook dan `tty/chat_preview.rs`), dengan test
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
  D=<repo>/crates/components/assets/fonts && mkdir -p $D
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

Diagnosis (2026-10-10), `platform/desktop` dan `platform/mobile`:

- **Penyebab error yang terkonfirmasi:** kedua `main.rs` memakai `asset!("/assets/tailwind.css")`, tetapi file itu hasil generate Tailwind dan di-`.gitignore`, jadi tidak ada di checkout bersih. `cargo check` gagal dengan `Asset at /assets/tailwind.css doesn't exist`. Kode Rust-nya sendiri sehat: dengan file itu ada, `cargo check` dan `clippy -D warnings` lolos di Linux (rustc 1.97.0, `--locked`).
- **Syarat di luar kode:** `cargo check -p desktop`/`-p mobile` di host Linux butuh library sistem webview (webkit2gtk-4.1, GTK3, libxdo). Di Nix sudah disediakan `devenv.nix`; di CI lihat `platforms.yml`.

- [x] Diagnosis di atas
- [x] `build.rs` di `platform/desktop` dan `platform/mobile`: membuat `assets/tailwind.css` kosong kalau belum ada (tidak pernah menimpa file yang ada). `dx`, CI, dan `nix build` tetap menimpanya dengan CSS asli. `platform/web` sengaja **tidak** diubah supaya jalur deploy yang berjalan tidak tersentuh
- [x] `.github/workflows/platforms.yml`: `cargo check` + `clippy -D warnings` untuk desktop dan mobile. Workflow **terpisah** dari `ci.yml`, jadi tidak menahan deploy Pages
- [x] `platforms.yml` terbukti lulus di runner GitHub sungguhan (log dari Moch, 2026-10-11)
- [x] Uji manual `desktop-dev` (2026-10-11, Moch): build selesai (±246 dtk), aplikasi terbuka, keluar dengan normal. Bagian dalamnya belum dirinci: CSS termuat, jam, window manager, kalkulator, Radio/Embience (audio di webview GTK butuh GStreamer), AI chat (HTTP lewat `reqwest`)
- [ ] Uji manual `android-dev` / `android-build`:
  - Percobaan 1 (2026-10-11) **gagal**: `Failed to install the following SDK components: platforms;android-34, build-tools;34.0.0 ... The SDK directory is not writable (/nix/store/...)`. Penyebab: project Gradle buatan `dx` 0.7.10 (AGP 8.7.0) memakai `compileSdk`/`targetSdk` 34 dan build-tools 34.0.0, sedangkan modul template `android` di `devenv.nix` hanya memasang platform dan build-tools 35/36. Gradle lalu mencoba memasang sendiri ke `/nix/store` yang read-only
  - Perbaikan (belum diuji): `devenv.nix` kini membangun SDK sendiri lewat `android-nixpkgs` (platform 34/35/36, build-tools 34.0.0/35.0.0/36.0.0, NDK 26.1.10909125, `platform-tools`, `cmdline-tools`) dan menimpa `env.ANDROID_HOME` dengan `lib.mkForce`. Sintaks Nix sudah dicek, evaluasinya belum (sandbox tidak bisa mengunduh flake). Jalankan ulang `android-build`; kalau Nix menolak nama atribut, kirim pesan errornya
  - Masih mungkin bermasalah setelah itu: `reqwest` + `rustls` (kompilasi `ring` dengan NDK), `[bundle]` identifier di `Dioxus.toml`, izin `INTERNET`
- [ ] Catatan versi: shell devenv memakai Rust 1.98.1 (fenix stable), sedangkan `rust-toolchain.toml` dan CI dikunci 1.97.0. `cargo clippy` lokal bisa menemukan lint yang tidak muncul di CI (atau sebaliknya). Untuk hasil identik dengan CI: `cargo +1.97.0 clippy ...` (kalau toolchain itu terpasang) atau naikkan pin di `rust-toolchain.toml` dan kedua workflow bersamaan
- [ ] Mobile: `open_external` masih menavigasi webview itu sendiri (lihat TODO di `platform/mobile/src/main.rs`); ganti dengan `Intent.ACTION_VIEW`
- [ ] Setelah D3: URL API untuk desktop/mobile (di web memakai variable build, lihat D1)

---

## 🗄️ Fullstack dan database

Tujuan: mengganti file JSON dummy di `crates/components/data/` dengan database sungguhan lewat API terpisah.

### Keputusan (D0, 2026-10-10)

| Hal | Keputusan |
| --- | --- |
| Hosting | **Opsi B**: frontend tetap di GitHub Pages (workflow, `CNAME`, `404.html`, `postbuild.py` tidak berubah) + API terpisah |
| Runtime API | **Axum** (Rust) di **Cloudflare Workers** lewat `workers-rs` (fitur `http`). Bukan Cloudflare Containers |
| Database | **Turso (libSQL)** untuk produksi; SQLite lokal untuk development (`turso dev`, protokol HTTP yang sama) |
| Repo | **Dua repo**: ini (frontend, GitHub Actions ke Pages) dan repo backend baru, **publik** (nama rencana `tquilla-api`, tetapi repo yang dibuat Moch bernama **`deisktify-worker.rs`**; URL itu yang dipakai di git dependency), deploy ke Cloudflare lewat Actions miliknya sendiri |
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
        │  HTTP ke Turso            │ (R2 dibatalkan: butuh kartu)
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
- [x] **D1** Fondasi, dipecah dua: **D1a** (repo backend) dan **D1b** (sisi repo ini). Keduanya selesai dan sudah dicek Moch (2026-10-15): `Cargo.lock` sudah di-commit, variable `API_BASE_URL` sudah di-set
  - [x] **D1a** Repo backend `tquilla-api` (zip terpisah): workspace Cargo, `crates/api-types`, `crates/turso-http` (klien Hrana-over-HTTP, tanpa dependensi native), `crates/api` (Axum + `workers-rs`, `GET /api/v1/health`, CORS allowlist, header keamanan, batas body 16 KiB), `wrangler.toml`, CI (fmt, clippy, test native, build wasm32, fixture dari klien referensi). Terverifikasi native: fmt, clippy `-D warnings`, 41 test. **Terverifikasi oleh Moch (2026-10-14):** CI hijau, `worker-build` + `wrangler dev` jalan, `GET /api/v1/health` → 200 `{"status":"ok","db":true}` (`SELECT 1` lewat protokol Hrana ke server libSQL). **Belum:** `batch`/transaksi (baru dipakai di D2), `wrangler deploy` ke Cloudflare, batas plan gratis. Temuan di jalan: `strip = true` membuat `worker-build` gagal (`externref table required for catch wrappers`), diganti `strip = "debuginfo"`; `turso dev` butuh `sqld`; `worker-build` dari `cargo install` gagal di NixOS (`libssl.so.3`), pakai paket nixpkgs + `WASM_OPT_BIN`
  - [x] D1a Dev lokal: `turso dev` + `wrangler dev`; `.dev.vars.example` (tanpa kredensial asli); `flake.nix`/`devenv.nix` backend (turso-cli, wrangler; sintaks dicek, evaluasi belum)
  - [x] **D1b** Repo ini (kode selesai, 2026-10-14): `crates/components/src/api.rs` (`pub mod api`): base URL dari `option_env!("API_BASE_URL")` (hanya `http(s)://host[:port]`, tanpa path; kalau kosong/salah → `ApiError::NotConfigured`, bukan host tebakan), `ApiError` (`NotConfigured`, `InvalidPath`, `BodyTooLarge`, `Network`, `Status{status, detail}`, `Decode`; `Display` singkat berbahasa Inggris dan tidak membocorkan URL/body), `get`/`post` bertipe di atas `platform::http`, `health()` (503 `degraded` dianggap data). Path hanya boleh di bawah `/api/v1/`. `api-types` jadi git dependency di `[workspace.dependencies]` (`tag = "v0.1.1"`). `API_BASE_URL` diteruskan ke `dx bundle` di `deploy-page.yml`. 9 test baru (62 total). Terverifikasi di sandbox dengan `api-types` lewat `path` (clippy `-D warnings` + test); lewat git belum, lihat langkah di bawah
    - Dev lokal web: `API_BASE_URL=http://127.0.0.1:8787 dx serve --package web --platform web` (backend: `db-dev` + `api-dev`; tambahkan `http://127.0.0.1:8080` ke `ALLOWED_ORIGINS` di `.dev.vars`)
    - Belum ada UI yang memakai `api` (tahap D3). `api::health()` belum dipanggil dari mana pun
  - Tidak dikerjakan: `devenv.nix`/`flake.nix` repo ini tidak perlu `turso-cli` dan `wrangler` selama dev backend dijalankan dari repo backend
- [x] **L1** Bahasa UI (kode selesai 2026-10-15; lihat uji manual di bawah)
  - [x] Modul `languages/` di `components` (`mod.rs` + `language.rs`): semua teks UI default **Inggris** terkumpul di satu tabel `Strings`/`EN` (grup `home`, `login`, `navbar`, `terminal`, `calc`, `calc_err`, dst.), ditulis lewat macro `strings!` supaya struct, tabel, dan daftar kunci untuk test lahir dari satu daftar. Template memakai `{nama}` dan diisi lewat method bertipe (`Time::ago`, `Home::generation_line`, ...), bukan `format!` di komponen
  - [x] Kalkulator ikut Inggris: mode **Algebra/Trigonometry/Calculus**, `True/False`, pemisah desimal `.`. Mesin kalkulator netral-locale (angka kanonik `.`; tampilan lewat `Strings.calc.decimal_separator`), input `,` tetap diterima. `CalcError` kini enum bertipe dan baru diterjemahkan saat tampil (`message(&Strings)`), `engine/tests.rs` disesuaikan
  - [x] Multi-bahasa: `enum Language` (sekarang hanya `En`) + `Signal<Language>` lewat context (`use_language_provider()` di `App`, `use_language()`, `use_strings()`), pola yang sama dengan `ClockFormat`. Menambah bahasa = varian baru di `Language` + satu literal `Strings { .. }` (compiler memaksa semua kunci terisi). Pesan error lain juga bertipe: `ApiError::message`, `Notice`, `UploadError`, `AiError`. **Belum ada** pemilih bahasa di Settings dan belum ada penyimpanan ke localStorage (sesuai rencana)
  - [x] Test (`languages/tests.rs`, 9 test): semua kunci terisi dan unik, kurung `{}` seimbang dan nama placeholder dikenal, semua method template tanpa sisa placeholder, kode bahasa unik, dan **pemindai sumber** yang gagal bila ada teks tampilan (teks node rsx, `title`/`alt`/`placeholder`/`aria-label`/`label`, literal berhuruf kapital/berkalimat) tertinggal di luar `languages/`. Pengecualian harus didaftarkan dengan alasan di `ALLOWED_LITERALS`/`ALLOWED_CONSTS`
  - Bukan teks UI (sengaja tidak dipindah): nama perintah shell, nama ikon Material Symbols, URL, id/kunci DOM, nama file/folder dari filesystem simulasi terminal, nama model AI, `BOOT_LOG` (log `systemd` palsu di layar boot), `MONTHS_SHORT` di `journal/util.rs` (kunci data "Sep 14" yang dicocokkan dengan data Worker), dan isi `data/*.json`
  - [ ] **Uji manual (Moch):** buka `/`, `/profile`, `/deisktify`; cek login, boot menu, navbar (menu dan hint), dock, Settings, File Manager, Radio, Embience, Guestbook (kirim entri; lihat uji manual D4), AI chat (tanpa key dan key salah), terminal (`help`, `neofetch`, `cd`/`cat` salah, perintah tak dikenal), jam (ganti 12/24 jam), jurnal; kalkulator (tiga mode, `1/0`, `x+1`, `2+2=4`, `0.1+0.2`, mengetik `,`)
- [x] **D2** Skema dan migrasi dari JSON ke tabel (selesai 2026-10-16 di repo backend; `up` dan `seed --samples` sudah dicoba Moch di `turso dev` sungguhan: 6 migrasi diterapkan, ulangan "nothing to do", seed idempotent)

  | File JSON (dummy) | Tabel | Kolom utama |
  | --- | --- | --- |
  | `generations.json` | `generations` | `number` (PK), `label`, `link`, `kernel`, `built_on` |
  | `browser.json` | `bookmarks` | `id`, `label`, `url`, `sort_order` |
  | `radio.json` | `radio_stations` | `id`, `label`, `slogan`, `stream_url`, `official_link`, `image_url`, `sort_order`, `is_active` |
  | `embience.json` | `embiences`, `embience_presets`, `embience_preset_items` | `id`, `name`, `icon_url`, `stream_url`; preset dan relasi ke embience |
  | `chat_sample.json` (dihapus di D4a) | `chat_messages` | `id`, `username`, `url`, `avatar_url`, `message`, `attachment_url`, `created_at` |
  | `journal.sample.json` | `journal_snapshots` | `payload` (JSON), `fetched_at`; pengganti cache Worker (D5: sekarang diisi cron, sampel hanya untuk dev) |

  - [x] Migrasi `.sql` di repo backend: `migrations/0001`–`0006` (satu berkas per fitur; `0004` memuat tiga tabel embience). Dialek SQLite tanpa `STRICT`. Kolom URL punya `CHECK` skema `http(s)://` (avatar dan lampiran chat hanya `https://`, jadi base64/`data:` ditolak di tingkat database); `generations.link` boleh rute internal tetapi menolak `//host` dan `/\`. Panjang teks dibatasi longgar (pagar terakhir; batas ketat ada di API, D4). Kolom tambahan dari rencana awal: `journal_snapshots.source` (`github` atau `seed`)
  - [x] Runner migrasi di `turso-http::migrate` + CLI `crates/migrate` (`status`, `up`, `seed [--samples]`). Satu migrasi = satu transaksi bersama pencatatan di tabel `_migrations` (checksum FNV-1a, CRLF dianggap LF). Ditolak: migrasi yang isinya berubah setelah dijalankan, migrasi yang hilang dari daftar, nomor baru di bawah nomor terakhir yang dijalankan, `BEGIN`/`COMMIT`/`ROLLBACK` di dalam berkas, kutip/komentar tak tertutup. Pemecah statement paham kutip, komentar, dan badan `CREATE TRIGGER`. Database non-lokal butuh `--yes`; `--samples` ditolak di sana. Token hanya dari environment dan tidak pernah dicetak
  - [x] Seed satu kali dari JSON yang ada: `scripts/gen_seed.mjs` (repo backend) menghasilkan `seeds/0001_content.sql` (generations, bookmarks, radio, embience + preset) dan `seeds/0002_samples.sql` (pesan chat contoh + satu snapshot jurnal bertanda `seed`). Dipisah supaya data palsu ("Username 1", dst.) tidak ikut ke produksi tanpa sengaja. Semua `INSERT OR IGNORE` dengan id eksplisit (aman diulang). Temuan di data sumber: `embience.json` punya dua preset ber-`preset_id` 2 ("Chill" dan "Busy"); id preset dibuat dari urutan (1, 2, 3). URL ikon embience masih menunjuk `raw.githubusercontent.com` repo frontend ini, jadi memindahkan atau mengganti nama `assets/logo-*.svg` mematahkannya
  - Terverifikasi di sandbox (rustc 1.97.0): fmt, clippy `-D warnings` (workspace), 68 test Rust, 18 test node. Migrasi + seed diterapkan di SQLite 3.45 sungguhan (idempotent, constraint menolak input buruk), dan CLI dijalankan end-to-end lewat server Hrana tiruan di atas SQLite. **Belum:** terhadap `sqld`/`turso dev` dan Turso cloud sungguhan; clippy/build wasm32 (target tidak ada di sandbox, jalan di CI backend)
  - [x] Tindakanmu 2 (dicoba terhadap `turso dev` sungguhan, hasil sesuai harapan). Tindakanmu 1 (push backend, CI hijau, tag) digabung dengan langkah D3 di bawah, karena tag yang sama membawa endpoint baru
  - Catatan: tidak ada kode di repo ini yang berubah di D2 selain README. `include_str!` data JSON dihapus per fitur di D3 (lihat di bawah)
- [x] **D3** Endpoint baca (read-only): `GET /api/v1/{generations,bookmarks,radio,embiences,presets}`; komponen memakai `use_resource` dengan state loading dan error (kode selesai 2026-10-16; tinggal uji manual dan deploy, urutannya di bawah) **Diuji dan beres oleh Moch (2026-10-20).**
  - [x] **Backend** (zip terpisah, tag `v0.2.0`): lima endpoint `GET`, query di `crates/api/src/queries/*.sql` dengan `ORDER BY` eksplisit (`generations.number DESC`, `sort_order`, `embience_preset_items.position`). Query yang sama dijalankan di test node terhadap SQLite sungguhan. Bentuk respons `{"items":[...]}` (tipe di `api-types`: `Generation`, `Bookmark`, `RadioStation`, `Embience`, `Preset`); preset memakai **id** embience, bukan nama. Baris dengan URL tidak aman (bukan `http(s)`, atau link menu `//host`) dibuang di server; URL opsional radio yang buruk dikosongkan. Sukses diberi `Cache-Control: public, max-age=60, stale-while-revalidate=300`; error tidak di-cache. DB tidak terjangkau → 503 `unavailable`
  - [x] **Frontend**: `api::{generations, bookmarks, radio, embiences, presets}` dengan sanitizer kedua di sisi klien (`api_types::validate`, definisi yang sama dengan server): baris dengan URL yang tidak bisa dipakai dibuang sebelum sampai ke `<iframe src>`, `<audio>`, atau tautan. `remote.rs`: komponen `Loading` dan `LoadFailed` (pesan dari `ApiError::message`, tombol "Try again" hanya untuk error yang layak diulang). Home, Browser, Radio, Embience memuat datanya lewat `use_resource`; di Embience preset yang gagal dimuat tidak menghalangi papan suara, dan preset kosong disembunyikan
  - [x] Home bergantung pada API sekarang. Kalau gagal atau kosong, halaman menampilkan error + "Try again" + dua tautan biasa ke `/deisktify` dan `/profile`, jadi situs tetap bisa dijangkau saat server mati. Hitung mundur auto-boot baru mulai setelah data termuat. Link menu dari server diperiksa: rute yang tidak dikenal **tidak** lagi dibuka sebagai URL eksternal, tetapi diabaikan
  - [x] File `generations.json`, `browser.json`, `radio.json`, `embience.json` dan semua `include_str!`-nya dihapus (isi aslinya hidup di `seeds/source/` repo backend sebagai sumber seed). `chat_sample.json` (dihapus di D4a) dan `journal.sample.json` tetap sampai D4/D5
  - [x] Terverifikasi di sandbox: frontend fmt, clippy `-D warnings`, 78 test Rust, 13 test JS, 6 test Python; backend fmt, clippy workspace, 86 test Rust, 23 test node. **Belum:** build wasm32 dan `dx` (target tidak ada di sandbox), menjalankan UI di browser, endpoint terhadap `sqld`/Turso cloud, CI job `wasm`
  - Rilis frontend sekarang lewat tag: `git tag generation_N && git push origin generation_N` (atau "Run workflow" di tab Actions). Push biasa ke `main` tidak men-deploy. Backend sama: tag `v*` menjalankan cek + migrasi database + seed (lihat README backend, "Migrasi otomatis di CI")
  - [x] Tindakanmu 1–3 selesai (tag backend `v0.2.0`, `Cargo.lock` frontend naik ke `v0.2.0`, Worker terdeploy dengan secret)
  - [x] **Temuan 2026-10-16 (laporan Moch):** Home menampilkan "The boot menu could not be loaded". Kodenya tidak salah; dua gejala berbeda, dua penyebab:
    1. `GET /api/v1/generations` → **404** `{"error":{"code":"not_found"...}}` dari Worker production: itu Worker **lama** (sebelum D3) yang belum punya rute baru. Setelah Worker D3 terdeploy, gejalanya berganti jadi
    2. **500** di `/radio`, `/embiences`, `/presets`, `/bookmarks`; di `wrangler dev` lokal pesannya `no such table: generations`. Artinya Worker sudah baru, tetapi **database yang dibacanya belum dimigrasi**. Migrasi yang kamu jalankan masuk ke database lain (kandidat: `turso dev` lokal, atau database Turso lain, mis. yang ada di `TURSO_DATABASE_URL2`), atau `turso dev` dijalankan ulang tanpa `--db-file` sehingga isinya hilang (mode memori)
    - Cek: di repo backend, dengan `TURSO_DATABASE_URL` dan `TURSO_AUTH_TOKEN` yang **sama persis** dengan secret Worker, jalankan `cargo run -p migrate -- status`. Kalau `pending 0001_generations` dan seterusnya, database itu kosong. Untuk lokal: `set -a; . crates/api/.dev.vars; set +a` lalu perintah yang sama
    - Perbaikan production: isi kedua secret GitHub di bawah dengan nilai yang sama seperti secret Worker, lalu jalankan job `migrate` (Run workflow dari `main`, atau tag `v*` baru). Perbaikan lokal: `cargo run -p migrate -- up` dan `seed --samples` ke database yang dipakai `.dev.vars`
  - [x] **Tindakanmu (setelah temuan di atas):**
    1. Repo backend → Settings > Secrets and variables > Actions: tambah `TURSO_DATABASE_URL` dan `TURSO_AUTH_TOKEN` (nilai sama dengan secret Worker)
    2. Push `ci.yml` baru, lalu jalankan job `migrate` (Run workflow dari `main`). Log harus berakhir "up to date (6 applied)"
    3. Buka `https://<worker>.workers.dev/api/v1/generations`: harus `200` dengan 5 item. Lalu buka https://tquilla.is-a.dev (hard refresh)
    4. Deploy frontend hanya lewat tag `generation_*` dengan `API_BASE_URL` = alamat Worker (variable repo)
    5. Lokal: backend (`db-dev`, `api-dev`) lalu `API_BASE_URL=http://127.0.0.1:8787 dx serve --package web --platform web`, dengan `http://127.0.0.1:8080` di `ALLOWED_ORIGINS`. Desktop dan mobile juga butuh `API_BASE_URL` saat build
  - [x] **Uji manual:** Home (loading, error saat Worker dimatikan, Try again, tautan cadangan, navigasi keyboard, auto-boot setelah termuat); Browser (bookmark tampil, tambah bookmark valid dan `javascript:...`); Radio (putar, ganti stasiun, stasiun tanpa gambar/tautan); Embience (nyalakan beberapa suara, preset, preset ikut menyala/mati); semua jendela saat API mati
- [x] **D4** Live Chat sungguhan, lalu **dirombak jadi Guestbook** (2026-10-19). Kode selesai; tinggal tindakan dan uji manual Moch **Tindakan dan uji manual selesai, diuji Moch (2026-10-20): dianggap beres.**
  - **Keputusan (2026-10-19):** R2 butuh kartu pembayaran dan Moch tidak punya, jadi **unggah gambar dibatalkan seluruhnya** (D4b versi lama). Avatar = avatar bawaan acak yang stabil per username (`default_avatar_for`, lima SVG). Live Chat diganti **Guestbook** yang mendukung **Markdown dengan preview**. Moderasi (laporkan, daftar admin, sembunyikan) ikut diselesaikan, di luar permintaan tertulis (revisi sore harinya: tag HTML dan gambar lewat `<img>` ikut didukung, lihat renderer): scaffolding-nya sudah ada di zip backend (tipe, path, migrasi `0008`) dan repo gagal `cargo test` tanpanya, dan buku tamu publik yang menerima Markdown butuh tombol mati
  - [x] **Backend** (tag **`v0.4.0`**, migrasi `0008_chat_moderation`): kontrak `ChatMessage` = `{id, username, url?, message, created_at}` (tanpa `avatar_url`/`attachment_url`; klien lama tetap bisa membaca karena field yang tak dikenal diabaikan). Pesan 1–**2000** karakter, maks **40** baris, tab jadi 4 spasi, ZWJ (emoji gabungan) diizinkan; disimpan **apa adanya**, server tidak merender. `GET /messages` mendapat `before=<id>` (halaman lebih lama); entri tersembunyi tidak pernah keluar. `POST /messages/{id}/report` (`204`, satu laporan per pengirim per entri, 20 laporan/jam per pengirim, tanpa auto-hide). `GET /admin/reports` dan `DELETE /admin/messages/{id}` lewat `Authorization: Bearer <ADMIN_TOKEN>` (secret ≥ 32 karakter, perbandingan waktu-konstan, gagal tertutup; CORS tidak mengizinkan `Authorization`/`DELETE`, jadi hanya lewat `curl`). Nama tabel `chat_messages` dan path `/messages` tidak diganti supaya Worker lama tetap jalan selama migrasi
  - [x] **Frontend, jendela Guestbook** (`nixdesk/gui/guestbook.rs`; app id `Guestbook`, kunci window `guestbook`, ikon baru `logo-guestbook.svg`): daftar entri **terbaru dulu**, tanpa polling (dimuat saat jendela dibuka; tombol **Refresh** hanya meminta entri lebih baru dari yang tampil, sampai 5 putaran; **Load older** memakai `before`; 20 entri per halaman, jadi tidak ada request saat tak ada yang melihat). Form: nama, website (opsional), editor dengan tab **Write / Preview**, toolbar (heading, bold, italic, strikethrough, code, link, image, quote, list, task list; hasilnya lewat `js/guestbook_editor.js`, dites dengan DOM palsu), penghitung karakter (merah saat lewat batas), **Ctrl/Cmd+Enter** untuk kirim, Enter biasa = baris baru. Preview menampilkan teks seperti yang akan disimpan server. Setiap entri punya tombol **Report** dua klik (klik pertama meminta konfirmasi). Entri sendiri langsung muncul dari jawaban POST dan daftar digulir ke atas
  - [x] **Renderer Markdown** (crate `crates/markdown`: `src/lib.rs` + `src/html.rs`; digambar oleh `crates/components/src/markdown_view.rs`; `pulldown-cmark` 0.13 tanpa fitur bawaan): peristiwa parser → pohon sendiri (`Block`/`Inline`) → komponen Dioxus. **Tidak pernah membuat string HTML dan tidak memakai `dangerous_inner_html`**. Didukung: paragraf, heading, **bold**, *italic*, ~~strikethrough~~, `code` dan blok kode berpagar (label bahasa disaring), kutipan, daftar bertingkat dan bernomor, **task list**, tabel (perataan kolom), garis pemisah, tautan, URL telanjang jadi tautan, **alert GitHub** (`> [!NOTE]`, `TIP`, `IMPORTANT`, `WARNING`, `CAUTION`), **gambar** (`![alt](https://…)` dan `<img src="https://…">`), dan **subset HTML ala README** (rincian di bawah)
    - **Subset HTML:** `<img>` (`src`, `alt`, `width`, `height`, `align`: left/right = mengambang di tepi dan teks mengalir di sampingnya, center = baris sendiri di tengah; nilai lain diabaikan), `<br>`, `<hr>`, `<b>`/`<strong>`, `<i>`/`<em>`, `<s>`/`<del>`/`<strike>`, `<u>`, `<mark>`, `<sub>`, `<sup>`, `<kbd>`, `<code>`, `<a href>`, `<h1>`–`<h6>`, `<p>`/`<div>`/`<center>` (atribut `align`: left/center/right), `<details>` dan `<summary>`. `markdown/html.rs` hanya memecah teks jadi token; daftar putihnya ada di `Builder::html_open`. Tag lain dibuang dan isinya dipertahankan; `<script>`, `<style>`, `<iframe>`, `<svg>`, `<object>`, dsb. dibuang **beserta isinya**. Komentar HTML dibuang. Atribut di luar daftar di atas (`style`, `class`, `id`, `onerror`, `srcset`, ...) tidak pernah dibaca: pohonnya tidak punya tempat untuk menyimpannya, jadi tidak bisa sampai ke halaman. Entitas (`&amp;`, `&#65;`) didekode, kecuali yang menghasilkan karakter kontrol atau pembalik arah (U+202E, zero-width); filter yang sama berlaku untuk teks yang didekode parser Markdown. Tag HTML tidak bisa melewati batas elemen Markdown tempat ia dibuka (tidak ada kebocoran gaya ke item/paragraf berikutnya)
    - **Gambar:** hanya `https://` yang dimuat (`http://` akan diblokir browser di halaman https); `loading="lazy"`, `decoding="async"`, `referrerpolicy="no-referrer"`, tinggi maksimum 20rem dan lebar maksimum selebar entri; `width`/`height` hanya petunjuk piksel (1–1000, persen diabaikan) dan hanya satu sisi dipakai supaya proporsi terjaga. Gagal dimuat → diganti teks alt. Sumber lain (`javascript:`, `data:`, relatif, `//host`, `http://`) tidak pernah dimuat; tampil sebagai teks alt. **Konsekuensi yang perlu kamu sadari:** siapa pun bisa menaruh gambar dari server mana saja, jadi IP pengunjung terlihat oleh server itu dan isi gambar tidak bisa dicek sebelum tampil. Pengamannya hanya tombol Report dan `DELETE` admin (menyembunyikan seluruh entri). Kalau nanti jadi masalah: batasi ke daftar host (satu `const` di `markdown/view.rs`) atau ubah jadi "klik untuk memuat"
    - Aturan lain: hanya `http(s)` dan `mailto:` polos yang jadi tautan (`javascript:`, `data:`, dll. kehilangan tautan tetapi teksnya tetap); tautan memakai `rel="noopener noreferrer nofollow ugc"` dan `title` berisi alamat aslinya; kedalaman daftar/kutipan/`<div>`/`<details>` dan span inline (Markdown maupun HTML) dibatasi 6 (sisanya diratakan, teks tetap ada); urutan peristiwa atau tag yang aneh tidak pernah membuat panik (ada test noise acak yang menyisipkan tag HTML)
  - [x] Mode Terminal: preview shell 4 memakai `markdown::plain_text` (tanpa sintaks Markdown, satu baris, dipotong 240 karakter), `Page::Latest`; teks kosong "No entries yet."
  - [x] Teks UI hanya lewat `Strings` (grup `guestbook`; scanner teks tetap hijau; "www." diizinkan di scanner karena awalan URL, bukan teks UI). Dock: tombol "Guestbook" (`dock.guestbook`, `dock.guestbook_group`, `dock.open_guestbook_menu`); `meta.desktop_description` disesuaikan
  - [x] Dihapus: `chat.rs` (digantikan `guestbook.rs`), `gui/livechat.rs`, `logo-livechat.svg`, polling dan indikator "Connection lost", rendering `avatar_url`/`attachment_url`
  - [x] Terverifikasi di sandbox: frontend `cargo fmt --check`, `cargo clippy -p components -p markdown --all-targets -D warnings`, 146 test Rust, 27 test JS, 6 test Python; backend fmt, clippy workspace, 140 test Rust, 31 test node. Frontend dibangun dengan `api-types` lewat `path` ke backend lokal (tag `v0.4.0` belum ada di GitHub), jadi `Cargo.lock` di zip **masih entri git lama**: lihat langkah (6) di bawah
  - **Belum terverifikasi** (sandbox tidak punya target wasm32, `dx`, atau browser; gambar sungguhan juga belum pernah dimuat, dan `onerror`/`referrerpolicy` pada `img` baru lolos compile): build wasm32; tampilan jendela di browser (kelas Tailwind belum pernah dirender, jadi kemungkinan perlu sedikit tweak spasi/tinggi); toolbar editor di browser asli (`document::eval`, event `input` yang dikirim ulang ke Dioxus, posisi kursor setelah klik); perilaku `pulldown-cmark` pada Markdown nyata di browser (hanya dites lewat test unit); **ukuran wasm** yang bertambah karena `pulldown-cmark` (tidak bisa diukur di sini; cek hasil `dx bundle` dan, bila perlu, `wasm-opt`); aksesibilitas (pembaca layar, urutan fokus tab); Turso cloud dan Worker sungguhan untuk laporan/admin
  - [x] **Restrukturisasi workspace (2026-10-19):** `packages/*` diganti `crates/{components,markdown}` + `platform/{web,desktop,mobile}`. Engine Markdown (`lib.rs` + `html.rs`) dipisah jadi crate `markdown` tanpa Dioxus; `markdown_view.rs` tetap di `components` (butuh komponen dan `Strings`). Perilaku tidak berubah (142 test sama, 92 + 50). Path diperbarui di `Cargo.toml`, `ci.yml`, `deploy-page.yml`, `flake.nix`, `devenv.nix`, `.gitignore`, `tailwind.css` (`@import`/`@source`), dan dokumen. Tes pemindai teks UI ikut memindai `crates/markdown/src`. **Belum terverifikasi:** `dx bundle` web dengan path baru, `nix build` (fileset baru), `cargo check -p desktop -p mobile` (sandbox tanpa GTK), dan `Cargo.lock` (anggota `markdown` baru masuk saat kamu menjalankan `cargo update -p api-types`).
  - [x] **`align` pada `<img>` (2026-10-20):** `Inline::Image` punya field `align`; `left`/`right`/`center` (huruf besar/kecil dan spasi di sekitarnya tidak masalah) dibaca, nilai lain (`top`, `middle`, CSS) tetap inline. `style` tetap tidak pernah dibaca. Di view: `float-left`/`float-right` dengan margin, `block mx-auto` untuk center; paragraf memakai `flow-root` supaya gambar mengambang tidak keluar dari bloknya. Test baru memakai baris badge sosial (`<p align ="center">` + `<code><a><img></a></code>` + badge `align="left"` + counter `align="right"`): 7 gambar, urutan dan perataan terjaga. **Belum terverifikasi:** tampilan nyata di browser (apakah baris ikon + float kiri/kanan muat satu baris di lebar jendela Guestbook; kalau tidak muat, float turun ke bawah, sama seperti di GitHub). Snippet itu 1412 karakter dari batas 2000.
  - [x] **Tindakanmu D4**, berurutan: (1) push repo backend, tunggu CI hijau (termasuk job `wasm`, yang belum pernah menjalankan kode laporan/admin), beri tag **`v0.4.0`**; (2) job `migrate` jalan otomatis di tag itu, log harus "up to date (8 applied)"; (3) `openssl rand -hex 32` lalu `npx wrangler secret put ADMIN_TOKEN` (dari `crates/api`; simpan nilainya di password manager untuk `curl`); (4) `npx wrangler deploy`; (5) di repo frontend ini: `cargo update -p api-types`, commit `Cargo.lock` (juga menarik `pulldown-cmark` ke lockfile); (6) cek `cargo clippy -p components --all-targets -- -D warnings` dan `dx bundle` hijau di mesinmu, lalu tag `generation_N`
  - [x] **Uji manual D4:** buka Guestbook: entri seed muncul, terbaru di atas, Markdown-nya ter-render; ketik `**tebal**`, `# judul`, daftar, `- [ ] tugas`, tabel, blok kode, `> [!NOTE]`, URL telanjang, `<b>tebal</b>`, `<u>`, `<kbd>`, `<details><summary>x</summary>…</details>`, `<div align="center"><img src="https://…" width="120"></div>` (gambar muncul di tengah), `![x](https://…)`, gambar di dalam tautan (badge), `<img src="https://alamat-rusak/x.png">` (diganti teks alt), `<img src="http://…">` dan `<img src="javascript:alert(1)">` (tidak memuat apa pun), `<script>alert(1)</script>`, `<img src=x onerror=alert(1)>`, `<a href="javascript:alert(1)">x</a>` dan `[x](javascript:alert(1))` (tidak boleh jadi tautan atau menjalankan apa pun); di Network tab pastikan permintaan gambar tidak mengirim header `Referer`; tab Preview sama dengan hasil setelah dikirim; tiap tombol toolbar (termasuk Image) dengan dan tanpa seleksi (klik kedua pada bold melepas tanda); Ctrl+Enter mengirim, Enter biasa menambah baris; nama kosong / `bad url` / `javascript:x` / 2001 karakter / 41 baris menampilkan error tanpa request; kirim 6 entri dalam semenit: yang ke-6 → "Too many requests"; Refresh dari tab lain menampilkan entri baru, tanpa entri baru → "You are up to date."; Load older sampai habis ("That is the first entry."); Report: klik pertama meminta konfirmasi, klik kedua → "Reported. Thank you."; `curl` daftar laporan lalu `DELETE` entri itu, Refresh/muat ulang jendela, entri hilang; `curl` admin tanpa token → `401`; matikan Worker: pesan error + Try again; mode Terminal: preview berputar dan tanpa sintaks Markdown; ganti bahasa; layar sempit (ponsel): form dan toolbar tidak melebar
- [x] **D5** Jurnal GitHub (2026-10-21): kode selesai di kedua repo; tinggal tindakan dan uji manual Moch (di bawah). Dikerjakan dua langkah seperti D1:
  - [x] **D5a** Repo backend (tag **`v0.5.0`**): Cron Trigger `0 */6 * * *` mengambil statistik dari GitHub GraphQL (port Rust dari Worker JavaScript lama `github-journal-stats`) dan menyimpannya ke `journal_snapshots` (`source = 'github'`, 28 terbaru dipertahankan). `GET /api/v1/journal` membaca snapshot terakhir; `POST /api/v1/admin/journal/refresh` (curl, `ADMIN_TOKEN`, paling cepat sekali per menit) mengambil sekarang. **Akun, repositori sematan, dan stack ada di tabel** (`journal_profile`, `journal_pinned_repos`, `journal_stack`, migrasi `0009`; nilai awal = isi `CONFIG` Worker lama) supaya D6 bisa mengubahnya; nilainya masuk query GraphQL hanya sebagai variabel. Payload adalah data, bukan tampilan: tidak ada HTML dan tidak ada kalimat (`api_types::journal`), diperiksa lagi sebelum disimpan dan sebelum dikirim.
  - [x] **D5b** Repo ini: `components::api::journal()` (`GET /api/v1/journal`, lalu `Data::validate` lagi di klien; payload yang tidak lolos jadi error `Decode`). `journal/data.rs`, `journal.sample.json`, `include_str!`-nya, banner "offline snapshot", dan variable `STATS_API_URL` **dihapus** (menggantikan D5 Tahap 0): tidak ada lagi data contoh yang bisa tertukar dengan data asli; kalau API mati atau belum ada snapshot, halaman menulis galat. Hanya snapshot `source = "seed"` (dev lokal) yang diberi banner. `fetched_at` tampil di Hero sebagai "Updated 3 hours ago". Kalimat "Monthly Focus" disusun dari templat `Strings` (`focus_sentence`) dan `journal.activity_for(bahasa)`; kata yang ditonjolkan adalah `span` biasa. **Tidak ada lagi `dangerous_inner_html` untuk teks dari server** (grafik SVG tetap memakainya, tetapi hanya berisi angka, tanggal dari digit, dan teks `Strings`; ada catatan di `chart.rs`). Nama hari memakai `s.calendar.weekday`, warna bahasa hanya `#rrggbb` (selain itu warna cadangan), `stack` kosong menyembunyikan bagiannya (dulu ada daftar bawaan Linux/Neovim/Arch), deskripsi kosong menampilkan `journal.no_description`, bahasa kosong `journal.unknown_language`. 8 test baru (`api` 2, `journal::util` 6; 4 test `stats_url` dan 1 test sampel ikut hilang); 99 test `components` + 54 `markdown`.
  - [ ] **Tindakanmu (urut):**
    1. Repo backend: `cargo run -p migrate -- status` lalu pastikan `0009_journal_config.sql` akan jalan (job `migrate` di CI menjalankannya saat tag `v*`). Commit, push, **tag `v0.5.0`**
    2. Buat token GitHub baru (scope `read:user`), lalu `cd crates/api && npx wrangler secret put GITHUB_TOKEN`. Setelah jurnal jalan, cabut token Worker lama
    3. `npx wrangler deploy` (membaca `[triggers] crons` dari `wrangler.toml`; cek di dashboard Cloudflare, Worker `tquilla-api` > Settings > Trigger Events, bahwa cron `0 */6 * * *` muncul)
    4. Ambil data pertama tanpa menunggu cron: `curl -s -X POST -H "Authorization: Bearer $ADMIN_TOKEN" "$API/api/v1/admin/journal/refresh"` (harus `{"fetched_at":"…"}`), lalu `curl -s "$API/api/v1/journal"`
    5. Repo ini: `cargo update -p api-types` (tag sudah `v0.5.0` di `Cargo.toml`), commit `Cargo.lock`, push (sebelum langkah ini `Cargo.lock` masih menunjuk tag lama, jadi `--locked` di CI akan gagal), lalu deploy lewat tag `generation_*`. Hapus variable repo `STATS_API_URL` (tidak dipakai lagi) dan Worker lama `github-journal-stats` di Cloudflare kalau sudah yakin
  - [x] **Uji manual:** `/profile` diuji Moch (2026-10-21) dan berjalan dengan baik. Yang belum dicek: kegagalan API (Worker mati / `API_BASE_URL` salah) dan **waktu CPU cron pertama di dashboard Cloudflare: harus di bawah 10 ms** (plan gratis), kalau tidak snapshot tidak diperbarui
- [ ] **D6** Dashboard admin: konten dan operasi bisa diubah tanpa `curl` dan tanpa deploy ulang. Keputusan (2026-10-21): UI-nya **JavaScript biasa tanpa build step** (Preact + htm yang disalin ke repo) di **repo backend**, dilayani Worker yang sama di `/admin/` (satu origin dengan API, jadi CORS tidak dilonggarkan dan situs publik tidak membawa kode admin). Teks UI bahasa Indonesia biasa (bukan `Strings`). Perubahan skema (kolom baru) hanya lewat `migrations/`, bukan dashboard. Tiga tahap, semuanya di repo backend; repo ini tidak berubah selain README:
  - [x] **D6a** Fondasi (backend tag **`v0.6.0`**, migrasi `0010`): `/admin/` dilayani lewat Workers Static Assets (`web/`, `[assets]`, CSP di `web/_headers`); login dengan `ADMIN_TOKEN` (hanya di memori halaman, kunci otomatis 10 menit, tombol Kunci); pembatas percobaan gagal (10 per 15 menit per IP-hash, juga untuk token benar); riwayat aksi admin (`admin_audit`); Ringkasan (jumlah baris tiap tabel, status Guestbook dan journal, migrasi terakhir). Endpoint baru: `POST /admin/login`, `GET /admin/overview`, `GET /admin/audit`. Diuji: 208 test Rust, 86 test node, dan halaman dijalankan di jsdom. **Belum** teruji di browser/Worker sungguhan (lihat README backend, "Yang sudah dan belum terverifikasi")
  - [x] **D6b** (backend tag **`v0.7.0`**, tanpa migrasi baru) Tab **Journal**: refresh dengan hitung mundur `Retry-After`, riwayat snapshot, ganti akun GitHub, repo sematan dan stack (tambah, ubah, hapus dengan konfirmasi, geser urutan; batas 12 ditegakkan di SQL, nama kembar → `409`). Tab **Guestbook**: saringan Dilaporkan / Disembunyikan / Semua, teks entri lengkap sebagai teks biasa, sembunyikan, tampilkan lagi, tolak laporan, muat yang lebih lama. 11 endpoint admin baru; riwayat mencatat tiap aksi. Diuji: 233 test Rust, 106 test node, dan halaman dijalankan di jsdom dengan server palsu berkeadaan. **Belum** teruji di browser/Worker sungguhan. Perubahan Journal baru terlihat di situs setelah refresh berikutnya
  - [ ] **D6c** Editor data umum (daftar putih tabel, tambah / ubah / hapus baris dengan konfirmasi, tanpa SQL mentah), Embience dan radio, serta ekspor JSON/CSV
  - [x] Autentikasi admin lewat secret Cloudflare (jangan hardcode kredensial; password demo di `login.rs` hanya untuk animasi): sudah berlaku di D6a (`ADMIN_TOKEN`)
- [ ] **D7** Perbaiki data yang usang: URL stream radio mengandung token (`rj-tok`) yang bisa kedaluwarsa; Cron Trigger memvalidasi berkala dan menandai `is_active = false`
- [ ] **D8** Deploy dan operasi
  - [ ] Workflow `wrangler deploy` di repo backend (`CLOUDFLARE_API_TOKEN` sebagai secret Actions, token Turso lewat `wrangler secret`)
  - [ ] Backup database (dump terjadwal), health check, rotasi secret
  - [ ] Frontend: deploy Pages tidak berubah; `404.html` fallback tetap

---

## Lisensi

Lihat `LICENSE`.
