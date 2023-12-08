{
  rustPlatform,
  pkgs,
  postgresql,
}:
rustPlatform.buildRustPackage {
  name = "api";
  cargoLock.lockFile = ./Cargo.lock;
  src = ./.;
  buildAndTestSubdir = "api";
  checkInputs = [postgresql];
}
