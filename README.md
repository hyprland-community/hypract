# Hypract [WIP]
KDE activities for Hyprland using Hyprland-rs

## Usage
> This cli tool replaces your workspace change commands so keep that in mind

- use `switch-workspace <workspace name>` to switch to that workspace
- use `switch-activity <activity name>` to switch to that activity

## Installation

### Cargo
To install just do `cargo install --git https://github.com/hyprland-community/hypract`
> I think

### Nix
To just run
```
nix run github:hyprland-community/hypract
```
Otherwise reference `the-flake-input.packages.${pkgs.system}.hypract`

#### Cachix
Binaries are pushed to `https://hyprland-community.cachix.org` with the key `hyprland-community.cachix.org-1:uhMZSrDGemVRhkoog1iYkDOUsyn8PwZrnlxci3B9dEg=`

## Anyrun
For anyrun details check [here](https://github.com/hyprland-community/hypract/tree/master/hypract-anyrun)
