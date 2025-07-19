{
  description = "Rustimenator - A REST API for managing tags, tasks, and timed events";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
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
        ] ++ lib.optionals stdenv.isDarwin [
          darwin.apple_sdk.frameworks.SystemConfiguration
        ];
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
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

          # Include migrations in the output
          postInstall = ''
            mkdir -p $out/share/rustimenator
            cp -r migrations $out/share/rustimenator/
          '';

        };

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
