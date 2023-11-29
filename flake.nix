{
  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/nixos-unstable-small;
    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    nixpkgs,
    treefmt-nix,
  }: let
    inherit
      (nixpkgs.lib)
      attrValues
      getExe'
      pipe
      ;

    systems = {"x86_64-linux" = {};};
    combine = fn:
      with builtins; let
        parts = mapAttrs (s: _: fn (nixpkgs.legacyPackages.${s})) systems;
        keys = foldl' (a: b: a // b) {} (attrValues parts);
      in
        mapAttrs (k: _: mapAttrs (s: _: parts.${s}.${k} or {}) systems) keys;
  in
    combine (pkgs: let
      treefmtEval = treefmt-nix.lib.evalModule pkgs ./treefmt.nix;
    in rec {
      packages = rec {
        label-tracker = pkgs.callPackage ./label-tracker.nix {};
        fetcher = pkgs.callPackage ./fetcher.nix {};
        api = pkgs.callPackage ./api.nix {};
      };

      devShells.default = pkgs.mkShell {
        inputsFrom = attrValues packages;
        packages = with pkgs; [rustfmt rust-analyzer clippy sqlx-cli];
        SQLX_OFFLINE = "true";
      };

      apps.sqlx-prepare = {
        type = "app";
        program =
          pipe {
            pname = "sqlx-prepare";
            version = "0.1.0";
            runtimeInputs = with pkgs; [sqlx-cli];
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            buildAndTestSubdir = "util";

            #<<< text = ''
            #<<<   cargo run --package util --bin
            #<<<   cargo sqlx prepare --workspace --database-url '<<<TODO>>>'
            #<<<   echo "hello, world"
            #<<< '';
          } [
            pkgs.rustPlatform.buildRustPackage
            (drv: getExe' drv "sqlx-prepare")
          ];
      };

      checks =
        packages
        // {
          formatting = treefmtEval.config.build.check self;
        };

      formatter = treefmtEval.config.build.wrapper;
    })
    // {
      nixosModule = import ./module.nix {inherit self;};
    };
}
