{
  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/nixos-unstable-small;
    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, treefmt-nix }:
    let
      systems = { "x86_64-linux" = {}; };
      combine = fn: with builtins;
        let
          parts = mapAttrs (s: _: fn (nixpkgs.legacyPackages.${s})) systems;
          keys = foldl' (a: b: a // b) {} (attrValues parts);
        in
          mapAttrs (k: _: mapAttrs (s: _: parts.${s}.${k} or {}) systems) keys;
    in
      combine (pkgs: rec {
        packages = rec {
          label-tracker = pkgs.callPackage ./default.nix {};
          default = label-tracker;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ packages.default ];
          packages = with pkgs; [ rustfmt rust-analyzer clippy ];
        };

        checks.build = packages.label-tracker;
        checks.formatting =
          let
            treefmtEval = treefmt-nix.lib.evalModule pkgs ./treefmt.nix;
          in treefmtEval.config.build.check self;
      }) // {
        nixosModule = import ./module.nix { inherit self; };
      };
}
