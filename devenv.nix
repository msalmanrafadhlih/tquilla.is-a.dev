{ templateInputs }:
{
  pkgs,
  lib,
  ...
}:
let
  system = pkgs.stdenv.hostPlatform.system;
  fenix = templateInputs.rust.inputs.fenix;
  rustToolChain = [
    # Android targets
    fenix.packages.${system}.targets.aarch64-linux-android.stable.rust-std
    fenix.packages.${system}.targets.x86_64-linux-android.stable.rust-std
    fenix.packages.${system}.targets.armv7-linux-androideabi.stable.rust-std
    fenix.packages.${system}.targets.i686-linux-android.stable.rust-std
    # Web target
    fenix.packages.${system}.targets.wasm32-unknown-unknown.stable.rust-std
  ];

  # Android SDK untuk `dx bundle/serve --platform android`.
  # Modul template `android` hanya memasang platform dan build-tools 35/36,
  # tetapi project Gradle buatan dx 0.7.x (AGP 8.7.0, compileSdk/targetSdk
  # default 34, tanpa buildToolsVersion) butuh `platforms;android-34` dan
  # `build-tools;34.0.0`. Kalau tidak ada, Gradle mencoba memasangnya sendiri
  # dan gagal karena SDK ada di /nix/store (read-only). Isi di bawah sama dengan
  # modul template + versi 34; ANDROID_HOME di-override lewat mkForce.
  # (Alternatif: tambahkan dua paket 34 itu di android/devenv.nix repo template.)
  androidSdk = templateInputs.android.inputs.android-nixpkgs.sdk.${system} (
    sdkPkgs: with sdkPkgs; [
      cmdline-tools-21-0
      platform-tools # adb
      build-tools-34-0-0
      build-tools-35-0-0
      build-tools-36-0-0
      platforms-android-34
      platforms-android-35
      platforms-android-36
      ndk-26-1-10909125 # sama dengan ndkVersion di modul template
    ]
  );
in
{
  imports = [
    templateInputs.rust.devenvModules.default
    templateInputs.android.devenvModules.default
  ];

  setupRust = {
    enable = true;
    toolchains = rustToolChain;
  };

  setupAndroid = {
    enable = true;
    backend = "android-nixpkgs"; # atau "devenv"
    emulator = false;
    device = true;
  };

  # ANDROID_SDK_ROOT, ANDROID_NDK_ROOT, dan NDK_HOME di modul template dihitung
  # dari config.env.ANDROID_HOME, jadi ikut berpindah ke SDK di atas.
  env.ANDROID_HOME = lib.mkForce "${androidSdk}/share/android-sdk";

  packages = with pkgs; [
    dioxus-cli # `dx`
    # HARUS v4: tailwind.css pakai sintaks v4 (@import "tailwindcss", @theme, ...).
    # Attr `tailwindcss` polos di nixpkgs = v3 -> "Failed to find 'tailwindcss'".
    tailwindcss_4
    git # dipakai `dx new`/`dx init` buat clone template

    pkg-config
    openssl
    xdotool
    binaryen

    # Runtime webview buat target desktop (dioxus-desktop pakai wry/tao,
    # dependency-nya sama persis dengan Tauri)
    webkitgtk_4_1
    gtk3
    libsoup_3
    librsvg
    at-spi2-atk
    glib-networking

    gdk-pixbuf
    cairo
    dbus

    gst_all_1.gstreamer
    gst_all_1.gst-plugins-base
    gst_all_1.gst-plugins-good
    gst_all_1.gst-plugins-bad
    gst_all_1.gst-libav
  ];

  env.LD_LIBRARY_PATH = lib.makeLibraryPath (
    with pkgs;
    [
      webkitgtk_4_1
      gtk3
      libsoup_3
      librsvg
      at-spi2-atk
      glib
      openssl
      xdotool
      binaryen

      gdk-pixbuf
      cairo
      dbus

      gst_all_1.gstreamer
      gst_all_1.gst-plugins-base
    ]
  );

  scripts = {
    dioxus-init.exec = "dx init";

    # Workspace: crates/{components,markdown} + platform/{web,desktop,mobile}.
    # `components` itu library UI bersama, `markdown` mesin parser README
    # (Rust murni, tanpa Dioxus); web/desktop/mobile cuma entry point per
    # platform. dx otomatis jalanin Tailwind (platform/<pkg>/tailwind.css ->
    # platform/<pkg>/assets/tailwind.css), jadi nggak perlu script manual.
    web-dev.exec = "dx serve --package web --platform web";
    web-build.exec = "dx bundle --package web --platform web --release";

    desktop-dev.exec = "dx serve --package desktop --platform desktop";
    desktop-build.exec = "dx bundle --package desktop --platform desktop --release";

    android-dev.exec = "dx serve --package mobile --platform android";
    android-build.exec = "dx bundle --package mobile --platform android --release";

    make-avd.exec = ''
      avdmanager create avd --force \
        --name dioxus-dev \
        --package 'system-images;android-34;google_apis_playstore;x86_64'
    '';

    # Cuma buat `nix build` (crane nggak lewat dx) atau debugging CSS.
    tailwind-build.exec = ''
      for pkg in web desktop mobile; do
        tailwindcss -i ./platform/$pkg/tailwind.css -o ./platform/$pkg/assets/tailwind.css --minify
      done
    '';
  };

  enterShell = ''
    _help() {
      echo "🧬 Dioxus Dev Shell Aktif (Desktop + Android + Web)"
      echo "rust targets : aarch64/x86_64/armv7/i686-linux-android, wasm32-unknown-unknown"

      echo ""
      echo "Panduan Inisialisasi Cepat:"
      echo "  1. Run: dioxus-init      (dx init, setup project Dioxus di folder ini)"
      echo ""
      echo "  Dev per platform:"
      echo "    desktop-dev / desktop-build"
      echo "    web-dev     / web-build   (dx compile Tailwind sendiri)"
      echo "    android-dev / android-build"
      echo ""
      echo "  tailwind-build cuma perlu dijalankan manual untuk 'nix build'"
      echo "  (target desktop) — dx serve/bundle sudah otomatis."
      echo ""
      echo "  Android emulator (opsional, [emulator = true]):"
      echo "     Run: make-avd   (bikin emulator AVD sekali saja)"
      echo "     Run: adb-device (cek serial number device)"
      echo "     ANDROID_SERIAL=<serial> android-dev  (untuk device spesifik)"
      echo ""
      if [ ! -f Cargo.toml ]; then
        echo ""
        echo "  Peringatan: Belum ada project Dioxus di folder ini!"
        echo "   Silakan run: dioxus-init"
      fi
      echo ""
      echo "  Kalau 'web-dev'/'web-build' error wasm-bindgen version mismatch:"
      echo "     https://wiki.nixos.org/wiki/Dioxus"
      echo ""
      echo "   ------------------------------------------------------------"
      echo "-------------------------------------------------------------------"
    }
    _help
  '';
}
