{rustPlatform, pkgs}:
pkgs.writeTextFile {
  name = "foo";
  text = "bar";
}
#<<< rustPlatform.buildRustPackage {
#<<<   pname = "fetcher";
#<<<
#<<<   cargoLock.lockFile = ./Cargo.lock;
#<<<
#<<<   src = ./fetcher;
#<<< }
