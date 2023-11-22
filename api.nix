{
  rustPlatform,
  pkgs,
}:
rustPlatform.buildRustPackage {
  name = "api";
  cargoLock.lockFile = ./Cargo.lock;
  src = ./.;
  buildAndTestSubdir = "api";
}
