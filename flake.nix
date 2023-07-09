{
  description = "Hypract devshell";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    nci = {
      url = "github:yusdacra/nix-cargo-integration";
    };
    flake-parts = {
      inputs = {
        nixpkgs-lib.follows = "nixpkgs";
      };
    };
  };
  outputs = inputs @ {
    flake-parts,
    nci,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        nci.flakeModule
      ];
      systems = ["x86_64-linux" "aarch64-linux"];
      perSystem = {
        pkgs,
        config,
        ...
      }: let
        mainCrateOutputs = config.nci.outputs.hypract;
        crate = let
          cmake-stuff = rec {
            RUSTFLAGS = "-C target-cpu=native";
            RUSTDOCFLAGS = RUSTFLAGS;
            overrideAttrs = old: {
              nativeBuildInputs = (old.nativeBuildInputs or []) ++ (with pkgs; [cmake pkg-config]);
            };
          };
        in {
          export = true;

          depsOverrides.cmake-stuff = cmake-stuff;
          overrides.cmake-stuff = cmake-stuff;
        };
      in {
        nci.projects.hypract.relPath = "";
        nci.crates.hypract = crate;
        nci.crates.hypract-anyrun = crate;
        devShells.default = mainCrateOutputs.devShell;
        packages.default = mainCrateOutputs.packages.release;
        packages.hypract = mainCrateOutputs.packages.release;
        packages.hypract-anyrun = config.nci.outputs.hypract-anyrun.packages.release;
      };
    };
}
