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

  packages = with pkgs; [
    dioxus-cli # `dx`
    tailwindcss # standalone CLI, tanpa Node/npm
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

    desktop-dev.exec = "dx serve --platform desktop";
    desktop-build.exec = "dx bundle --platform desktop";

    web-dev.exec = "dx serve --platform web";
    web-build.exec = "dx bundle --platform web";

    android-dev.exec = "dx serve --platform android";
    android-build.exec = "dx bundle --platform android";

    make-avd.exec = ''
      avdmanager create avd --force \
        --name dioxus-dev \
        --package 'system-images;android-34;google_apis_playstore;x86_64'
    '';

    # Sesuaikan path -i/-o dengan struktur project (cek assets/ & Dioxus.toml
    # setelah `dioxus-init`, path default template Dioxus biasanya assets/tailwind.css).
    tailwind-watch.exec = ''
      tailwindcss -i ./assets/tailwind.css -o ./assets/tailwind_output.css --watch
    '';
    tailwind-build.exec = ''
      tailwindcss -i ./assets/tailwind.css -o ./assets/tailwind_output.css --minify
    '';
  };

  enterShell = ''
    _help() {
      echo "🧬 Dioxus Dev Shell Aktif (Desktop + Android + Web)"
      echo "rust targets : aarch64/x86_64/armv7/i686-linux-android, wasm32-unknown-unknown"

      echo ""
      echo "Panduan Inisialisasi Cepat:"
      echo "  1. Run: dioxus-init      (dx init, setup project Dioxus di folder ini)"
      echo "  2. Run: tailwind-watch   (compile Tailwind, jalankan di terminal terpisah)"
      echo ""
      echo "  Dev per platform:"
      echo "    desktop-dev / desktop-build"
      echo "    web-dev     / web-build"
      echo "    android-dev / android-build"
      echo ""
      echo "  Android emulator (opsional, [emulator = true]):"
      echo "     Run: make-avd   (bikin emulator AVD sekali saja)"
      echo "     Run: adb-device (cek serial number device)"
      echo "     ANDROID_SERIAL=<serial> android-dev  (untuk device spesifik)"
      echo ""
      echo "  Kalau 'web-dev'/'web-build' error wasm-bindgen version mismatch:"
      echo "     https://wiki.nixos.org/wiki/Dioxus"

      if [ ! -f Cargo.toml ]; then
        echo ""
        echo "  Peringatan: Belum ada project Dioxus di folder ini!"
        echo "   Silakan run: dioxus-init"
      fi
    }
    _help
  '';
}
