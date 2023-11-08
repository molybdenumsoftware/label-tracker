{
  rustPlatform,
  pkg-config,
  openssl,
  lib,
  clippy,
}:
rustPlatform.buildRustPackage {
  pname = "label-tracker";
  version = "0.1.1";

  src = lib.cleanSource ./.;

  nativeBuildInputs = [pkg-config clippy];
  buildInputs = [openssl];

  RUSTFLAGS = "--deny warnings";

  cargoLock.lockFile = ./Cargo.lock;

  preBuild = "cargo clippy";
}
