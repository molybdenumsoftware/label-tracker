{
  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/nixos-unstable-small;
    flake-utils.url = "github:numtide/flake-utils";
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
      getExe
      pipe
      ;
  in
    forEachDefaultSystem = system: rec {
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
        program = pipe "sqlx-prepare" [util getExe];
      };

      apps.db-repl = {
        type = "app";
        program = pipe "db-repl" [util getExe];
      };

      checks =
        packages
        // {
          formatting = treefmtEval.config.build.check self;
        };

      formatter = treefmtEval.config.build.wrapper;
    };
    combine (pkgs: let
      treefmtEval = treefmt-nix.lib.evalModule pkgs ./treefmt.nix;
      util = bin: pkgs.writeShellScriptBin "util-${bin}" "cargo run --package util --bin ${bin}";
    in )
    // {
      nixosModule = import ./module.nix {inherit self;};
    };
}
