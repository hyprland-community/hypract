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
        crateName = "hypract";
        crateOutputs = config.nci.outputs.${crateName};
      in {
        nci.projects.${crateName}.relPath = "";
        nci.crates.${crateName} = {
          export = true;
          overrides.cmake-stuff.overrideAttrs = old: {
            nativeBuildInputs = (old.nativeBuildInputs or []) ++ (with pkgs; [cmake pkg-config]);
          };
        };
        devShells.default = crateOutputs.devShell;
        packages.default = crateOutputs.packages.release;
      };
    };
}
