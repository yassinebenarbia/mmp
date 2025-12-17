{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      utils,
      naersk,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        naersk-lib = pkgs.callPackage naersk { };
        rust_overlay = import (
          builtins.fetchTarball {
            url = "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
            sha256 = "0yml2nxjrldj11zp8ca8rmcq9qwc37j9ap0nkjqj6qykkbwxspzc";
          }
        );
        pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };
        rustVersion = "1.91.0";
        rust = pkgs.rust-bin.stable.${rustVersion}.default.override {
          extensions = [
            "rust-src" # for rust-analyzer
            "rust-analyzer"
            "rustc"
            "cargo"
          ];
        };
      in
      {
        defaultPackage = naersk-lib.buildPackage {
          pname = cargoToml.package.name;
          version = cargoToml.package.version;
          edition = cargoToml.package.edition;
          src = ./.;
        };
        devShell =
          with pkgs;
          mkShell {
            buildInputs = [
              rust
              pre-commit
              rustPackages.clippy
            ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
      }
    );
}
