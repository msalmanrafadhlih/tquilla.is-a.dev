{
  description = "Dioxus (Desktop + Android + Web) — Dev Template";

  inputs = {
    systems.url = "github:nix-systems/default";

    rust.url = "github:msalmanrafadhlih/nixos-development-templates/main?dir=rust";
    android.url = "github:msalmanrafadhlih/nixos-development-templates/main?dir=android";

    flake-utils = {
      follows = "rust/flake-utils";
      inputs.systems.follows = "systems";
    };

    nixpkgs.follows = "rust/nixpkgs";
    devenv.follows = "rust/devenv";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      ...
    }@inputs:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config = {
            android_sdk.accept_license = true;
            allowUnfree = true;
          };
        };
        inherit (pkgs) lib;

        crane = inputs.rust.inputs.crane;
        craneLib = crane.mkLib pkgs;

        # Runtime libs buat target desktop (dioxus-desktop pakai wry/tao,
        # dependency-nya sama persis dengan Tauri).
        desktopRuntimeLibs = with pkgs; [
          webkitgtk_4_1
          gtk3
          libsoup_3
          librsvg
          at-spi2-atk
          xdotool
          binaryen
          glib-networking
          gdk-pixbuf
          cairo
          dbus
          gst_all_1.gstreamer
          gst_all_1.gst-plugins-base
        ];

        src = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            ./Cargo.toml
            ./Cargo.lock
            (lib.fileset.maybeMissing ./Dioxus.toml)
            ./src
            ./assets
          ];
        };

        inherit (craneLib.crateNameFromCargoToml { inherit src; }) pname version;

        commonArgs = {
          inherit src pname version;
          strictDeps = true;
          nativeBuildInputs = with pkgs; [
            pkg-config
            makeWrapper
            tailwindcss
          ];
          buildInputs = with pkgs; [ openssl ] ++ desktopRuntimeLibs;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        dioxusDesktop = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;

            # `asset!()` di Dioxus butuh file CSS-nya udah ada pas compile time,
            # jadi Tailwind di-generate dulu sebelum `cargo build`.
            # Sesuaikan path -i/-o dan nama feature "desktop" di bawah dengan
            # struktur project & Cargo.toml kamu (hasil `dx init`/`dx new`).
            preBuild = ''
              tailwindcss -i ./assets/tailwind.css -o ./assets/tailwind_output.css --minify
            '';

            postInstall = ''
              wrapProgram "$out/bin/${pname}" \
                --set LD_LIBRARY_PATH ${lib.makeLibraryPath desktopRuntimeLibs}
            '';
          }
        );
      in
      {
        # Cuma target desktop yang di-build lewat `nix build`.
        #
        # Web sengaja nggak dibikinin package Nix: `dx bundle --platform web`
        # jalanin pipeline wasm-bindgen + optimasi asset yang gak gampang
        # direplikasi hermetic pakai crane. Pakai `web-build` di devShell aja.
        #
        # Android juga sengaja nggak dibikinin: butuh signing key, Gradle,
        # dan NDK toolchain yang gak praktis dibikin hermetic di sandbox Nix.
        # Pakai `android-build` di devShell.
        packages.default = dioxusDesktop;

        checks = {
          app = dioxusDesktop;
          clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- -D warnings";
            }
          );
        };

        devShells = {
          default = inputs.devenv.lib.mkShell {
            inherit inputs pkgs;
            modules = [
              (import ./devenv.nix { templateInputs = inputs; })
            ];
          };
        };
      }
    )
    // {
      devenvModules.default = import ./devenv.nix { templateInputs = inputs; };
    };
}
