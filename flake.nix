{
  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/nixos-unstable-small;
  };

  outputs = { self, nixpkgs }:
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
      }) // {
        nixosModule = import ./module.nix { inherit self; };
      };
}
