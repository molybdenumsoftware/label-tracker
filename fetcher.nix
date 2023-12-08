{
  rustPlatform,
  buildInputs,
}:
rustPlatform.buildRustPackage {
  name = "fetcher";
  cargoLock.lockFile = ./Cargo.lock;
  src = ./.;
  buildAndTestSubdir = "fetcher";
  inherit buildInputs;
}
