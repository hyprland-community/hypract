# Hypract anyrun plugin
This simple anyrun plugin lets you change workspaces and activities from anyrun.

## Configuration
Wherever the anyrun configuration directory is in a file named `hypract.ron`,
you can configure 2 options `prefix` and `max_entries`. `prefix` is used
to check when to actually do the plugin, the default is `:ha`. And `max_entries`
is pretty self explanatory.

## Screenshot
> Ignore the lack of icons, it's a bug with my icon pack 😛

![screenshot](https://i.imgur.com/l2vv7mC.png)

## Installation

### Cargo

1. Clone the repo: `git clone https://github.com/hyprland-community/hypract` or `gh repo clone hyprland-community/hypract`
2. CD to it: `cd hypract`
3. Build it: `cargo build --release --package=hypract-anyrun`
4. Copy the built binary somewhere and reference it in your configuration (binary should be in `target/release/libhypract_anyrun.so`)

### Nix

### Home Manager
If you use Home Manager you can reference the flake in your anyrun hm configuration.
(package is at `the-flake-input.packages.${pkgs.system}.hypract-anyrun`)

### Manual
Build with `nix build github:hyprland-community/hypract#hypract-anyrun` and reference in anyrun configuration
