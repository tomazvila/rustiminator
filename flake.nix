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
        
        rustToolchain = pkgs.rust-bin.stable."1.87.0".default.override {
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
          
          src = ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          
          inherit nativeBuildInputs buildInputs;
          
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

        # NixOS module for the service
        nixosModules.default = { config, lib, pkgs, ... }: {
          options.services.rustimenator = {
            enable = lib.mkEnableOption "Rustimenator service";
            
            port = lib.mkOption {
              type = lib.types.port;
              default = 3000;
              description = "Port to listen on";
            };
            
            openFirewall = lib.mkOption {
              type = lib.types.bool;
              default = false;
              description = "Open firewall port";
            };
            
            dataDir = lib.mkOption {
              type = lib.types.str;
              default = "/var/lib/rustimenator";
              description = "Directory to store database and state";
            };
          };
          
          config = lib.mkIf config.services.rustimenator.enable {
            systemd.services.rustimenator = {
              description = "Rustimenator REST API Service";
              after = [ "network.target" ];
              wantedBy = [ "multi-user.target" ];
              
              serviceConfig = {
                Type = "simple";
                DynamicUser = true;
                StateDirectory = "rustimenator";
                WorkingDirectory = config.services.rustimenator.dataDir;
                
                Environment = [
                  "DATABASE_URL=sqlite://${config.services.rustimenator.dataDir}/rustimenator.db"
                ];
                
                ExecStart = "${self.packages.${pkgs.system}.default}/bin/rustimenator";
                
                # Security hardening
                PrivateTmp = true;
                ProtectSystem = "strict";
                ProtectHome = true;
                ReadWritePaths = config.services.rustimenator.dataDir;
                
                Restart = "on-failure";
                RestartSec = "5s";
              };
            };
            
            networking.firewall.allowedTCPPorts = 
              lib.optional config.services.rustimenator.openFirewall config.services.rustimenator.port;
          };
        };
      });

}
