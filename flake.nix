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
      in
      {
        # Belum ada build package via Nix (dx bundle di-handle manual lewat devShell).
        # Kalau nanti butuh `nix build`, tinggal tambahkan `packages.default` di sini.
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
