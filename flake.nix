{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?ref=nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    wrappers = {
      url = "git+file:///home/trr/dev/RTUnreal/wrappers";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay.url = "github:oxalica/rust-overlay";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } (
      { inputs, ... }:
      {
        imports = [
          inputs.treefmt-nix.flakeModule
        ];
        systems = [
          "x86_64-linux"
          "aarch64-linux"
          "aarch64-darwin"
        ];

        perSystem =
          {
            pkgs,
            lib,
            system,
            ...
          }:
          let

            devState = "$(pwd)/.devState";
            pgPort = 5432;

            postgres-process = pkgs.writeShellApplication {
              name = "postgress-process";
              runtimeInputs = [
                pkgs.postgresql
              ];
              text = ''
                PGDATA="${devState}/postgres/db"
                export PGDATA
                echo "PGDATA: $PGDATA"
                echo "PGHOST: $PGHOST"


                copyConfig() {
                  # TODO: this should be run every startup
                  cat > "$PGDATA/postgresql.conf" <<EOF
                port = ${toString pgPort}
                listen_addresses = '''
                unix_socket_directories='$PGHOST'
                EOF
                }
                if [[ ! -d "$PGDATA" ]]; then
                  echo ""
                  echo "Database missing, regenerating"
                  echo ""

                  mkdir -p "$PGDATA"
                  initdb --locale=C --encoding=UTF8

                  copyConfig
                  pg_ctl -D "$PGDATA" -w start

                  psql --dbname postgres <<'EOF'
                    CREATE ROLE "app" WITH LOGIN PASSWORD 'app';
                    CREATE DATABASE "app" OWNER "app";
                EOF
                  pg_ctl -D "$PGDATA" -m fast -w stop
                else
                  copyConfig
                fi
                exec postgres -D "$PGDATA"
              '';
            };

            pcWrapped =
              (inputs.wrappers.wrapperModules.process-compose.apply {
                inherit pkgs;
                processConfig = {
                  is_strict = true;
                  shell = {
                    shell_command = lib.getExe pkgs.bash;
                    shell_argument = "-c";
                  };
                  environment = [
                    "PGHOST=\${PGHOST:-}"
                  ];

                  processes = {
                    adminer = {
                      command = "exec ${lib.getExe pkgs.php} -S 127.0.0.1:8100 -t ${pkgs.adminer} ${pkgs.adminer}/adminer.php";
                    };

                    postgres = {
                      command = "${lib.getExe postgres-process}";
                    };
                  };
                };
              }).wrapper;
          in
          {
            _module.args.pkgs = import inputs.nixpkgs {
              inherit system;

              overlays = [
                inputs.rust-overlay.overlays.default
                (self: _super: {
                  rust-overte = self.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
                })
              ];
              config = { };
            };
            packages = {
              /*
                overte-metaverse = pkgs.callPackage ./nix/overte.nix {
                  inherit (self'.packages)
                    glad
                    scribe
                    gif_creator
                    artery-font-format
                    cgltf
                    etc2comp
                    draco
                    glm
                    ;
                  libv8 = pkgs.nodejs_22.libv8;
                };
                default = self'.packages.overte-metaverse;
              */
              inherit pcWrapped;
            };
            treefmt = {
              programs.sql-formatter.enable = true;
              programs.rustfmt.enable = true;
              programs.nixfmt.enable = true;
            };
            devShells.default = pkgs.mkShell {
              packages = [
                pcWrapped
                pkgs.rust-overte
                pkgs.sqlx-cli
                pkgs.pnpm
              ];

              shellHook = ''
                export PGHOST="${devState}/postgres-socket"
                mkdir -p "$PGHOST"
                export DATABASE_URL="postgresql://app:app@$(${lib.getExe pkgs.python311} -c "import urllib.parse, sys; print(urllib.parse.quote(sys.argv[1], safe='''))" "$PGHOST")/app"

                echo "Adminer url: http://127.0.0.1:8100/?pgsql=$(${lib.getExe pkgs.python311} -c "import urllib.parse, sys; print(urllib.parse.quote(sys.argv[1], safe='''))" "$PGHOST")"
              '';
            };
          };
      }
    );
}
