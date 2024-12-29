{
  description = "bounty-cli - A CLI tool";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        inherit (nixpkgs) lib;
        
        # Use static builds for Docker
        pkgsStatic = import nixpkgs {
          inherit system overlays;
          crossSystem = {
            config = "x86_64-unknown-linux-musl";
            isStatic = true;
          };
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "x86_64-unknown-linux-musl" ];
        };

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
        ];

        buildInputs = with pkgs; [
          openssl
          libgit2
        ];

        # Native package for local development and running
        nativePackage = pkgs.rustPlatform.buildRustPackage {
          pname = "bounty-cli";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit buildInputs nativeBuildInputs;
        };

        # Static package for Docker
        dockerPackage = pkgsStatic.rustPlatform.buildRustPackage {
          pname = "bounty-cli";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgsStatic; [
            pkg-config
            pkgsStatic.stdenv.cc
          ];

          buildInputs = with pkgsStatic; [
            openssl.dev
            openssl.out
            libgit2.dev
            zlib.dev
          ];

          # Enable static linking
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          OPENSSL_STATIC = "1";
          OPENSSL_LIB_DIR = "${pkgsStatic.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgsStatic.openssl.dev}/include";
          OPENSSL_NO_VENDOR = "1";
          LIBGIT2_SYS_USE_PKG_CONFIG = "1";
          PKG_CONFIG_ALL_STATIC = "1";
          PKG_CONFIG_PATH = "${pkgsStatic.openssl.dev}/lib/pkgconfig:${pkgsStatic.libgit2.dev}/lib/pkgconfig";
          
          # Force static linking
          NIX_LDFLAGS = "-L${pkgsStatic.openssl.out}/lib -lssl -lcrypto";
          
          # Use musl target
          RUSTFLAGS = "-C target-feature=+crt-static";
        };

      in
      {
        # Development shell
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };

        # Packages
        packages = {
          default = nativePackage;
          docker = pkgs.dockerTools.buildLayeredImage {
            name = "bounty-cli";
            tag = "latest";
            
            # Use a minimal base
            fromImage = pkgs.dockerTools.buildImage {
              name = "scratch";
              tag = "latest";
            };

            contents = [
              (pkgs.runCommand "minimal-runtime" {} ''
                mkdir -p $out/etc/ssl/certs
                cp ${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt $out/etc/ssl/certs/
              '')
              dockerPackage
            ];

            config = {
              Cmd = [ "/bin/bounty-cli" ];
              Env = [
                "SSL_CERT_FILE=/etc/ssl/certs/ca-bundle.crt"
              ];
            };
          };
        };

        # Apps
        apps.default = flake-utils.lib.mkApp {
          drv = nativePackage;
        };

        # Optional: Overlay to make the package available in other flakes
        overlays.default = final: prev: {
          bounty-cli = nativePackage;
        };
      }
    );
} 