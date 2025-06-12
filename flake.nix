{
  description = "Flake to manage the trmt builds.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = {
    self,
    nixpkgs,
    flake-parts,
  } @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} (top @ {
      config,
      withSystem,
      moduleWithSystem,
      ...
    }: {
      flake = {
        homeManagerModules = rec {
          trmt = import ./nix/modules/home-manager.nix self;
          default = trmt;
        };
      };
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      perSystem = {
        config,
        pkgs,
        ...
      }: {
        packages = rec {
          trmt = pkgs.callPackage ./nix/package.nix {};
          default = trmt;
        };
      };
    });
}
