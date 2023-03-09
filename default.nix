{ rustPlatform
, pkg-config
, openssl
, lib
}:

rustPlatform.buildRustPackage {
  pname = "label-tracker";
  version = "0.1.1";

  src = lib.cleanSource ./.;

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ openssl ];

  cargoLock.lockFile = ./Cargo.lock;
}
