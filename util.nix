{
  rustPlatform,
  sqlx-cli,
}:
rustPlatform.buildRustPackage
{
  pname = "sqlx-prepare";
  version = "0.1.0";
  runtimeInputs = [sqlx-cli];
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;
  buildAndTestSubdir = "util";

  #<<< text = ''
  #<<<   cargo run --package util --bin
  #<<<   cargo sqlx prepare --workspace --database-url '<<<TODO>>>'
  #<<<   echo "hello, world"
  #<<< '';
}
