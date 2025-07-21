{
  description = "Rustimenator - A REST API for managing tags, tasks, and timed events";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
        };

        nativeBuildInputs = with pkgs; [ 
          rustToolchain
          pkg-config 
        ];

        buildInputs = with pkgs; [ 
          sqlite 
        ];

        rustimenatorPkg = pkgs.rustPlatform.buildRustPackage {
          pname = "rustimenator";
          version = "0.1.0";

          src = pkgs.lib.cleanSource ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit nativeBuildInputs buildInputs;

          prePatch = ''
            cp -r ${./migrations} migrations
          '';

          preBuild = ''
            export DATABASE_URL="sqlite:./temp.db"
            ${pkgs.sqlx-cli}/bin/sqlx database create
            ${pkgs.sqlx-cli}/bin/sqlx migrate run
          '';

          postInstall = ''
            mkdir -p $out/share/rustimenator
            cp -r migrations $out/share/rustimenator/
          '';
        };
      in
      {
        packages = {
          default = rustimenatorPkg;

          dockerImage = pkgs.dockerTools.buildLayeredImage {
            name = "rustimenator";
            tag = "0.1.0";
            contents = [ rustimenatorPkg pkgs.sqlx-cli pkgs.sqlite pkgs.bash ];

            config = {
              Cmd = [ "/entrypoint.sh" ];
              ExposedPorts = { "8080/tcp" = {}; };
              Volumes = { "/data" = {}; };
            };
          };
        };

        packages.entrypoint = pkgs.writeShellScriptBin "entrypoint.sh" ''
          #!/usr/bin/env bash
          set -e

          # Set database path (inside mounted volume)
          export DATABASE_URL="sqlite:/data/rustimenator.db"

          # Create DB if not exists and run migrations
          if [ ! -f "/data/rustimenator.db" ]; then
            sqlx database create
          fi
          sqlx migrate run --source /share/rustimenator/migrations

          # Start the app
          exec rustimenator
        '';

        devShells.default = pkgs.mkShell {
          inherit buildInputs;
          nativeBuildInputs = nativeBuildInputs ++ (with pkgs; [
            sqlx-cli
            cargo-watch
            rust-analyzer
          ]);

          shellHook = ''
            export DATABASE_URL="sqlite:./rustimenator.db"
            echo "Rustimenator development environment"
            echo "DATABASE_URL=$DATABASE_URL"
          '';
        };
      });
}

