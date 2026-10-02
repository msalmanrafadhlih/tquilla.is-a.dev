# My Portofolio — Dioxus multi-platform

```
packages/
├── components/   # library UI bersama (Route, Home, Journal, NixOS desktop sim, assets, data)
├── web/          # entry point web (wasm)       — index.html, window.open
├── desktop/      # entry point desktop (webview) — buka link di browser sistem
└── mobile/       # entry point Android/iOS      — TODO: buka link via intent
```

Platform package cuma: `dioxus::launch`, load `assets/tailwind.css` miliknya,
dan `use_context_provider(|| PlatformServices { .. })`. Semua UI ada di `components`.

## Jalankan

Pakai `-p`/`--package` (jangan `cargo check --workspace`: fitur `web`, `desktop`,
`mobile` Dioxus tidak boleh nyala bersamaan).

```sh
dx serve --package web     --platform web
dx serve --package desktop --platform desktop
dx serve --package mobile  --platform android
```

## Tailwind (v4)

- Tiap platform punya `tailwind.css` di root crate-nya; `dx` otomatis menjalankan
  Tailwind dan menulis `assets/tailwind.css` (di-gitignore).
- Theme bersama ada di `packages/components/theme.css`.
- Class Tailwind yang dipakai di `components/src/**` otomatis ter-scan lewat `@source`.
- Pastikan `tailwindcss_4` (bukan `tailwindcss` = v3) ada di PATH — lihat `dx doctor`.

## Menambah logika khusus platform

1. Tambah field/fungsi di `PlatformServices` (`components/src/platform.rs`).
2. Isi implementasinya di `web/src/main.rs`, `desktop/src/main.rs`, `mobile/src/main.rs`.
3. Panggil lewat `use_platform()` dari komponen.
