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

      # Function to create static builds for different CPU architectures
      mkStaticPackage = pkgs: arch:
        let
          pkgsStatic = import nixpkgs {
            system = pkgs.system;
            overlays = [ (import rust-overlay) ];
            crossSystem = {
              config = "${arch}-unknown-linux-musl";
              isStatic = true;
            };
          };
        in pkgsStatic.rustPlatform.buildRustPackage {
          pname = "bounty";
          version = "0.1.0";
          src = ./.;

          cargoLock = { lockFile = ./Cargo.lock; };

          nativeBuildInputs = with pkgsStatic; [ pkg-config stdenv.cc ];

          buildInputs = with pkgsStatic; [
            openssl.dev
            openssl.out
            libgit2.dev
            zlib.dev
          ];

          CARGO_BUILD_TARGET = "${arch}-unknown-linux-musl";
          OPENSSL_STATIC = "1";
          OPENSSL_LIB_DIR = "${pkgsStatic.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgsStatic.openssl.dev}/include";
          OPENSSL_NO_VENDOR = "1";
          LIBGIT2_SYS_USE_PKG_CONFIG = "1";
          PKG_CONFIG_ALL_STATIC = "1";
          PKG_CONFIG_PATH =
            "${pkgsStatic.openssl.dev}/lib/pkgconfig:${pkgsStatic.libgit2.dev}/lib/pkgconfig";

          NIX_LDFLAGS = "-L${pkgsStatic.openssl.out}/lib -lssl -lcrypto";
          RUSTFLAGS = "-C target-feature=+crt-static -C link-arg=-static";

          stripAllList = [ "bin" ];
        };

      # Create minimal versions of the binaries
      mkStaticBinary = pkgs: pkg:
        pkgs.runCommand "bounty-static" { } ''
          mkdir -p $out/bin
          cp ${pkg}/bin/bounty $out/bin/
        '';

      # Function to create Docker images for different architectures
      mkDockerImage = pkgs: arch: binary:
        pkgs.dockerTools.buildLayeredImage {
          name = "bounty";
          tag = "latest";

          fromImage = pkgs.dockerTools.buildImage {
            name = "scratch";
            tag = "latest";
          };

          contents = [
            (pkgs.runCommand "ssl-certs" { } ''
              mkdir -p $out/etc/ssl/certs
              cp ${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt $out/etc/ssl/certs/
            '')
            binary
          ];

          config = {
            Entrypoint = [ "/bin/bounty" ];
            Cmd = [ ];
            Env = [ "SSL_CERT_FILE=/etc/ssl/certs/ca-bundle.crt" ];
            Architecture = if arch == "arm64" then "arm64v8" else arch;
          };
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

        # Static builds and Docker images
        docker-amd64 = mkDockerImage pkgs "amd64"
          (mkStaticBinary pkgs (mkStaticPackage pkgs "x86_64"));
        docker-arm64 = mkDockerImage pkgs "arm64"
          (mkStaticBinary pkgs (mkStaticPackage pkgs "aarch64"));
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
