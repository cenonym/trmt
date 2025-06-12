{
  lib,
  rustPlatform,
}: let
  cargoToml = lib.importTOML ../Cargo.toml;
in
  rustPlatform.buildRustPackage (finalAttrs: {
    pname = "trmt";
    version = cargoToml.package.version;
    src = ../.;
    cargoLock.lockFile = ../Cargo.lock;
  })
