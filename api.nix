{
  rustPlatform,
  postgresql,
  buildInputs,
}:
rustPlatform.buildRustPackage {
  name = "api";
  cargoLock.lockFile = ./Cargo.lock;
  src = ./.;
  buildAndTestSubdir = "api";
  checkInputs = [postgresql];
  inherit buildInputs;
}
