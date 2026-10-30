{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?ref=nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    wrappers = {
      url = "github:lassulus/wrappers";
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
            self',
            ...
          }:
          let

            devState = "$(pwd)/.devState";
            pgPort = 5432;
            pgUser = "app";
            pgPW = "app";
            pgDB = "app";

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
                      depends_on.postgres.condition = "process_healthy";
                    };

                    postgres =
                      let
                        PGDATA = "${devState}/postgres/db";

                        command = pkgs.writeShellApplication {
                          name = "postgress-process";
                          runtimeInputs = [
                            pkgs.postgresql
                          ];
                          text = ''
                            PGDATA="${PGDATA}"
                            export PGDATA
                            echo "PGDATA: $PGDATA"
                            echo "PGHOST: $PGHOST"


                            copyConfig() {
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
                            CREATE ROLE "${pgUser}" WITH LOGIN PASSWORD '${pgPW}';
                            CREATE DATABASE "${pgDB}" OWNER "${pgUser}";
                            EOF
                              pg_ctl -D "$PGDATA" -m fast -w stop
                            else
                              copyConfig
                            fi
                            exec postgres -D "$PGDATA"
                          '';
                        };

                        check = pkgs.writeShellApplication {
                          name = "postgres-check";
                          runtimeInputs = [
                            pkgs.postgresql
                          ];
                          text = ''
                            PGDATA="${PGDATA}"
                            export PGDATA

                            if (shopt -s nullopt dotglob; f=("$PGHOST"/*); ((''${#f[@]}))); then
                              pg_isready -d template1 && \
                              psql -c "SELECT 1" template1 > /dev/null 2>&1
                            else
                              echo "Waiting for PostgreSQL initialization to complete..." 2>&1
                              exit 1
                            fi
                          '';
                        };
                      in
                      {
                        command = lib.getExe command;
                        readiness_probe = {
                          exec.command = lib.getExe check;
                          initial_delay_seconds = 2;
                          timeout_seconds = 4;
                          success_threshold = 1;
                          failure_threshold = 5;
                        };
                        # SIGINT requests a fast shutdown: https://www.postgresql.org/docs/current/server-shutdown.html
                        shutdown.signal = 2;
                      };
                  };
                };
              }).wrapper;

            rustPlatform = pkgs.makeRustPlatform {
              cargo = self'.packages.rust-overte;
              rustc = self'.packages.rust-overte;
            };
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
              overte-directory =
                pkgs.callPackage
                  (
                    {
                      version,
                      rustPlatform,
                      pnpm,
                      pnpmConfigHook,
                      fetchPnpmDeps,
                    }:
                    rustPlatform.buildRustPackage (finalAttrs: {
                      pname = "overte-directory";
                      inherit version;
                      nativeBuildInputs = [
                        pnpm
                        pnpmConfigHook
                      ];

                      src = ./.;
                      cargoLock = {
                        lockFile = ./Cargo.lock;
                      };
                      pnpmDeps = fetchPnpmDeps {
                        inherit (finalAttrs) pname version src;
                        fetcherVersion = 4;
                        hash = "sha256-Bgz0kprmiv05201qcu/q/TrCcKGaGUAqWh/MyCicAJE=";
                      };
                    })
                  )
                  {
                    version = inputs.self.shortRev or "dirty";
                    inherit rustPlatform;
                  };
              default = self'.packages.overte-directory;
              inherit pcWrapped;
              inherit (pkgs) rust-overte;
            };
            treefmt = {
              programs.sql-formatter.enable = true;
              programs.rustfmt.enable = true;
              programs.nixfmt.enable = true;
            };
            devShells.default = pkgs.mkShell {
              packages = [
                pcWrapped
                pkgs.sqlx-cli
              ];
              inputsFrom = [ self'.packages.overte-directory ];

              shellHook = ''
                export PGHOST="${devState}/postgres/socket"
                mkdir -p "$PGHOST"
                export DATABASE_URL="postgresql://${pgUser}:${pgPW}@$(${lib.getExe pkgs.python311} -c "import urllib.parse, sys; print(urllib.parse.quote(sys.argv[1], safe='''))" "$PGHOST")/${pgDB}"

                echo "Adminer url: http://127.0.0.1:8100/?pgsql=$(${lib.getExe pkgs.python311} -c "import urllib.parse, sys; print(urllib.parse.quote(sys.argv[1], safe='''))" "$PGHOST")&username=${pgUser}&db=${pgDB}"
              '';
            };
          };
      }
    );
}
