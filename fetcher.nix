{
  rustPlatform,
  system,
  lib,
  pkgs
}: let
  inherit
    (lib)
    hasSuffix
    ;
in
  rustPlatform.buildRustPackage {
    name = "fetcher";
    cargoLock.lockFile = ./Cargo.lock;
    src = ./.;
    buildAndTestSubdir = "fetcher";
    buildInputs =
      if hasSuffix "-darwin" system
      then [pkgs.darwin.apple_sdk.frameworks.SystemConfiguration]
      else [];
  }
