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
      getExe
      pipe
      ;

    systems = {"x86_64-linux" = {}; "aarch64-darwin" = {};};
    combine = fn:
      with builtins; let
        parts = mapAttrs (s: _: fn (nixpkgs.legacyPackages.${s})) systems;
        keys = foldl' (a: b: a // b) {} (attrValues parts);
      in
        mapAttrs (k: _: mapAttrs (s: _: parts.${s}.${k} or {}) systems) keys;
  in
    combine (pkgs: let
      treefmtEval = treefmt-nix.lib.evalModule pkgs ./treefmt.nix;
      util = bin: pkgs.writeShellScriptBin "util-${bin}" "cargo run --package util --bin ${bin}";
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
    })
    // {
      nixosModule = import ./module.nix {inherit self;};
    };
}
