{
  rustPlatform,
  system,
  lib,
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
      then []
      else [];
  }
