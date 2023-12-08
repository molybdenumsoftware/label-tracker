{
  rustPlatform,
  darwinBuildInputs,
}:
rustPlatform.buildRustPackage {
  name = "fetcher";
  cargoLock.lockFile = ./Cargo.lock;
  src = ./.;
  buildAndTestSubdir = "fetcher";
  buildInputs = darwinBuildInputs;
}
