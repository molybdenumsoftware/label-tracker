{
  rustPlatform,
  pkgs,
}:
rustPlatform.buildRustPackage {
  name = "fetcher";
  cargoLock.lockFile = ./Cargo.lock;
  src = ./.;
  buildAndTestSubdir = "fetcher";
  buildInputs = with pkgs; [iconv.dev];
}
