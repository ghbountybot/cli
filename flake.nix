{
  # This is the main configuration file for building and developing the bounty Rust project
  # using the Nix package manager. It handles dependencies, building, and Docker image creation.
  description = "bounty - A CLI tool";

  # External dependencies needed to build the project
  inputs = {
    # nixpkgs: The main Nix package repository containing thousands of packages
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    # rust-overlay: Provides Rust toolchain management
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, ... }:
    let
      forAllSystems = function:
        nixpkgs.lib.genAttrs [
          "x86_64-linux"
          "aarch64-linux"
          "x86_64-darwin"
          "aarch64-darwin"
        ] (system:
          function (import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          }));

      # Configure the Rust toolchain with necessary components and target platforms
      mkRustToolchain = pkgs:
        pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets =
            [ "x86_64-unknown-linux-musl" "aarch64-unknown-linux-musl" ];
        };

    in {
      # Development shells for each system
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          buildInputs = with pkgs; [ openssl libgit2 ];
          nativeBuildInputs = with pkgs; [ (mkRustToolchain pkgs) pkg-config ];
          RUST_SRC_PATH =
            "${mkRustToolchain pkgs}/lib/rustlib/src/rust/library";
        };
      });

      # Packages for each system
      packages = forAllSystems (pkgs: {
        default = pkgs.rustPlatform.buildRustPackage {
          pname = "bounty";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          buildInputs = with pkgs; [ openssl libgit2 ];
          nativeBuildInputs = with pkgs; [ (mkRustToolchain pkgs) pkg-config ];
        };
      });

      # Default app for each system
      apps = forAllSystems (pkgs: {
        default = {
          type = "app";
          program = "${self.packages.${pkgs.system}.default}/bin/bounty";
        };
      });

      # Make the package available to other Nix flakes
      overlays.default = final: prev: {
        bounty = self.packages.${prev.system}.default;
      };
    };
}
