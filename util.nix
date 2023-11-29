{
  rustPlatform,
  sqlx-cli,
  lib,
}:

let
  inherit (lib)
    fileset;

  sourceFiles = fileset.unions [ #<<<
    ./util
    ./Cargo.lock
    ./Cargo.toml
  ];
in

fileset.trace sourceFiles

rustPlatform.buildRustPackage
{
  pname = "sqlx-prepare";
  version = "0.1.0";
  runtimeInputs = [sqlx-cli];
  #<<< src = ./.;
  src = fileset.toSource {
    root = ./.;
    fileset = fileset.unions [
      ./util
      ./Cargo.lock
      ./Cargo.toml
    ];
  };
  cargoLock.lockFile = ./Cargo.lock;
  buildAndTestSubdir = "util";

  preBuild = ''
    echo "HIYAAAAAAAAAAAA";
    ls
    cat Cargo.toml
  '';

  #<<< text = ''
  #<<<   cargo run --package util --bin
  #<<<   cargo sqlx prepare --workspace --database-url '<<<TODO>>>'
  #<<<   echo "hello, world"
  #<<< '';
}
