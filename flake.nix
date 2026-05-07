{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?ref=nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    devenv.url = "github:cachix/devenv";
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
          inputs.devenv.flakeModule
          inputs.treefmt-nix.flakeModule
        ];
        systems = [
          "x86_64-linux"
          "aarch64-linux"
          "aarch64-darwin"
        ];

        perSystem =
          { pkgs, lib, ... }:
          {
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
            };
            treefmt = {
              programs.sql-formatter.enable = true;
              programs.rustfmt.enable = true;
              programs.nixfmt.enable = true;
            };
            devenv.shells.default = {
              packages = [ pkgs.sqlx-cli ];
              services.postgres = {
                enable = true;
                initialDatabases = [
                  {
                    name = "app";
                    user = "app";
                    pass = "app";
                  }
                ];
              };
              # http://127.0.0.1:8080/?pgsql=%2Frun%2Fuser%2F1000%2Fdevenv-613fd34%2Fpostgres
              services.adminer.enable = true;
              languages.rust = {
                enable = true;
                toolchainFile = ./rust-toolchain.toml;
              };

              enterShell = ''
                export DATABASE_URL="postgresql://app:app@$(${lib.getExe pkgs.python311} -c "import urllib.parse, sys; print(urllib.parse.quote(sys.argv[1], safe='''))" "$PGHOST")/app"
              '';
            };
          };
      }
    );
}
