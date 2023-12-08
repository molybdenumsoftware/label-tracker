{
  rustPlatform,
  pkgs,
}:
rustPlatform.buildRustPackage {
  name = "fetcher";
  cargoLock.lockFile = ./Cargo.lock;
  src = ./.;
  buildAndTestSubdir = "fetcher";
  nativeBuildInputs = with pkgs; [iconv.dev];
}
