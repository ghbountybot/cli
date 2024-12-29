{
  # This is the main configuration file for building and developing the bounty-cli Rust project
  # using the Nix package manager. It handles dependencies, building, and Docker image creation.
  description = "bounty-cli - A CLI tool";

  # External dependencies needed to build the project
  inputs = {
    # nixpkgs: The main Nix package repository containing thousands of packages
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    
    # rust-overlay: Provides Rust toolchain management
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    
    # flake-utils: Helper functions for creating Nix flakes
    flake-utils.url = "github:numtide/flake-utils";
  };

  # The main build configuration for the project
  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        # Set up the basic Nix environment with Rust support
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        inherit (nixpkgs) lib;

        # Configure the Rust toolchain with necessary components and target platforms
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "x86_64-unknown-linux-musl" "aarch64-unknown-linux-musl" ];
        };

        # Tools needed during the build process
        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
        ];

        # Libraries needed by the project
        buildInputs = with pkgs; [
          openssl  # For HTTPS support
          libgit2  # For Git operations
        ];

        # Build configuration for local development
        nativePackage = pkgs.rustPlatform.buildRustPackage {
          pname = "bounty-cli";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit buildInputs nativeBuildInputs;
        };

        # Function to create static builds for different CPU architectures (x86_64, ARM64)
        # Static builds are self-contained and don't depend on system libraries
        mkStaticPackage = arch: let
          pkgsStatic = import nixpkgs {
            inherit system overlays;
            crossSystem = {
              config = "${arch}-unknown-linux-musl";
              isStatic = true;
            };
          };
        in pkgsStatic.rustPlatform.buildRustPackage {
          pname = "bounty-cli";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgsStatic; [
            pkg-config
            stdenv.cc
          ];

          buildInputs = with pkgsStatic; [
            openssl.dev
            openssl.out
            libgit2.dev
            zlib.dev
          ];

          # Environment variables for static linking
          CARGO_BUILD_TARGET = "${arch}-unknown-linux-musl";
          OPENSSL_STATIC = "1";
          OPENSSL_LIB_DIR = "${pkgsStatic.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgsStatic.openssl.dev}/include";
          OPENSSL_NO_VENDOR = "1";
          LIBGIT2_SYS_USE_PKG_CONFIG = "1";
          PKG_CONFIG_ALL_STATIC = "1";
          PKG_CONFIG_PATH = "${pkgsStatic.openssl.dev}/lib/pkgconfig:${pkgsStatic.libgit2.dev}/lib/pkgconfig";
          
          # Force static linking of all dependencies
          NIX_LDFLAGS = "-L${pkgsStatic.openssl.out}/lib -lssl -lcrypto";
          RUSTFLAGS = "-C target-feature=+crt-static -C link-arg=-static";
          
          # Strip debug symbols to reduce binary size
          stripAllList = [ "bin" ];
        };

        # Create packages for different CPU architectures
        x86_64Package = mkStaticPackage "x86_64";  # For Intel/AMD processors
        aarch64Package = mkStaticPackage "aarch64"; # For ARM processors (e.g., Apple M1/M2)

        # Create minimal versions of the binaries (just the executable, no extra files)
        mkStaticBinary = pkg: pkgs.runCommand "bounty-cli-static" {} ''
          mkdir -p $out/bin
          cp ${pkg}/bin/bounty-cli $out/bin/
        '';

        x86_64Binary = mkStaticBinary x86_64Package;
        aarch64Binary = mkStaticBinary aarch64Package;

        # Function to create Docker images for different architectures
        mkDockerImage = arch: binary: pkgs.dockerTools.buildLayeredImage {
          name = "bounty-cli";
          tag = "latest";
          
          # Start from an empty base image
          fromImage = pkgs.dockerTools.buildImage {
            name = "scratch";
            tag = "latest";
          };

          # Include SSL certificates for HTTPS support
          contents = [
            (pkgs.runCommand "ssl-certs" {} ''
              mkdir -p $out/etc/ssl/certs
              cp ${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt $out/etc/ssl/certs/
            '')
            binary
          ];

          # Docker image configuration
          config = {
            Entrypoint = [ "/bin/bounty-cli" ];  # Command to run when container starts
            Cmd = [ ];
            Env = [
              "SSL_CERT_FILE=/etc/ssl/certs/ca-bundle.crt"  # Enable SSL certificate verification
            ];
            Architecture = if arch == "arm64" then "arm64v8" else arch;
          };
        };

      in
      {
        # Development environment configuration
        # Run: nix develop
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };

        # Build targets
        # Run: nix build .#<target>
        packages = {
          default = nativePackage;                    # Regular build
          docker-amd64 = mkDockerImage "amd64" x86_64Binary;   # Docker image for Intel/AMD
          docker-arm64 = mkDockerImage "arm64" aarch64Binary;  # Docker image for ARM
        };

        # Run the program directly
        # Run: nix run
        apps.default = flake-utils.lib.mkApp {
          drv = nativePackage;
        };

        # Make the package available to other Nix flakes
        overlays.default = final: prev: {
          bounty-cli = nativePackage;
        };
      }
    );
} 