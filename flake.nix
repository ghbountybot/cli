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
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
        ];

        buildInputs = with pkgs; [
          openssl
          libgit2
        ];

        # Define the package separately so we can reuse it in the Docker image
        package = pkgs.rustPlatform.buildRustPackage {
          pname = "bounty-cli";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit buildInputs nativeBuildInputs;
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
          default = package;
          docker = pkgs.dockerTools.buildImage {
            name = "bounty-cli";
            tag = "latest";
            
            copyToRoot = pkgs.buildEnv {
              name = "image-root";
              paths = [ 
                package 
                pkgs.cacert
              ];
              pathsToLink = [ "/bin" "/etc/ssl" ];
            };

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
          drv = package;
        };

        # Optional: Overlay to make the package available in other flakes
        overlays.default = final: prev: {
          bounty-cli = package;
        };
      }
    );
} 