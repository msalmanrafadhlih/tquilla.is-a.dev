# desktify-lab

Sekarang saya lihat isi folder `bruh/` (hasil `dx new bruh`):Setelah saya bongkar isi zip-nya, jadi jelas kelihatan bedanya. File ini sebenarnya berisi **dua hasil scaffolding yang digabung dalam satu workspace**: bagian root (workspace `desktify-lab`) itu hasil `dx init`, dan folder `bruh/` itu hasil `dx new bruh` yang dijalankan belakangan di dalam workspace yang sama. Berikut rinciannya.

## 1. `dx init` → struktur multi-package (workspace) di root

Root `Cargo.toml` isinya workspace, bukan satu package tunggal:

```toml
[workspace]
members = ["bruh", "packages/desktop", "packages/mobile", "packages/ui", "packages/web"]
resolver = "2"

[workspace.dependencies.dioxus]
version = "0.7.1"
[workspace.dependencies.ui]
path = "packages/ui"
```

Ini menghasilkan 4 crate terpisah di `packages/`:
- **`packages/ui`** — crate library murni (`lib.rs` cuma `mod hero; pub use hero::Hero;`), tempat komponen UI yang dipakai bersama.
- **`packages/web`, `packages/desktop`, `packages/mobile`** — masing-masing crate binary kecil, `Cargo.toml`-nya cuma punya dependency `dioxus` + `ui` (via `workspace = true`), dan `main.rs`-nya nyaris identik: import `ui::Hero`, load asset platform-spesifik, lalu `dioxus::launch(App)`.

Tidak ada `Dioxus.toml` sama sekali di root atau di `packages/*` — konfigurasi platform murni diatur lewat **Cargo feature flags** (`default = ["web"]`, `web = ["dioxus/web"]`, dst di tiap `Cargo.toml`).

**Intinya:** `dx init` di sini menghasilkan pola *"satu UI crate, banyak entrypoint binary per platform"* — cocok untuk aplikasi yang benar-benar mau di-ship terpisah ke web/desktop/mobile tapi berbagi komponen lewat crate `ui`.

## 2. `dx new bruh` → single-crate app klasik

Folder `bruh/` adalah project mandiri (lalu ditambahkan manual ke `members` workspace). Bedanya cukup mencolok:

| Aspek | `dx init` (root) | `dx new bruh` |
|---|---|---|
| Struktur | Workspace, 4 crate terpisah | 1 crate tunggal |
| Config platform | Cargo `[features]` saja | **`Dioxus.toml`** + `[features]` |
| Kode UI | Terpisah di crate `ui`, di-share via path dependency | Dalam crate yang sama, di `src/components/` |
| Assets | Per-platform (`packages/web/assets`, `packages/desktop/assets`, dst) | Satu folder `assets/` untuk semua platform |
| Styling | CSS biasa (`main.css`) | CSS biasa **+ Tailwind otomatis** (`tailwind.css`) |
| Jalankan platform lain | `cd packages/desktop && dx serve` (crate berbeda) | `dx serve --platform desktop` (flag saja) |
| Komentar kode | Minim | Banyak komentar penjelasan (khas template pemula) |

Beberapa bukti konkret dari file:

**`bruh/Cargo.toml`** — satu package, semua platform jadi feature dari crate yang sama:
```toml
[package]
name = "bruh"
...
[dependencies]
dioxus = { version = "0.7.1", features = [] }
[features]
default = ["web"]
web = ["dioxus/web"]
desktop = ["dioxus/desktop"]
mobile = ["dioxus/mobile"]
```

**`bruh/Dioxus.toml`** — file konfigurasi khusus CLI Dioxus yang cuma ada di sini:
```toml
[application]
[web.app]
title = "bruh"
[web.resource]
style = []
script = []
```

**`bruh/src/main.rs`** — komponen `Hero` diimpor dari modul lokal (`mod components;`), bukan dari crate eksternal, dan ada tambahan `TAILWIND_CSS` yang tidak muncul di project hasil `dx init`:
```rust
use components::Hero;
mod components;
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
```

Ini juga terlihat dari `bruh/README.md` yang menyebut fitur "Automatic Tailwind (Dioxus 0.7+)" — otomatis kalau ada file `tailwind.css` di sebelah `Cargo.toml`, sesuatu yang memang jadi bagian dari template default `dx new`.

## 3. Kenapa keduanya bisa hidup berdampingan di satu workspace?

Karena struktur workspace Cargo cukup fleksibel: `dx init` bikin skeleton workspace kosong (atau minimal) → lalu perintah `dx new bruh` dipakai untuk generate satu app baru dengan template default Dioxus (yang sebetulnya didesain untuk berdiri sendiri) → app itu kemudian didaftarkan sebagai `members` tambahan di `Cargo.toml` root. Itu sebabnya `bruh` punya `Dioxus.toml` dan `.gitignore` sendiri (khas project mandiri), sementara `packages/*` tidak.

## Kesimpulan praktis

- **`dx init`** cocok kalau dari awal kamu memang berencana ship ke banyak platform dan mau berbagi komponen lewat satu crate `ui` — strukturnya lebih "enterprise/monorepo".
- **`dx new <nama>`** cocok untuk mulai cepat satu aplikasi, semua platform diatur lewat feature flag di crate yang sama, plus dapat bonus setup Tailwind otomatis — lebih pas untuk pemula atau prototipe cepat.

