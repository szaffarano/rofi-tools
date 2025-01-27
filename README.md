# Rofi Tools

![GitHub Release](https://img.shields.io/github/v/release/szaffarano/rofi-tools?sort=date&style=flat-square)
![GitHub License](https://img.shields.io/github/license/szaffarano/rofi-tools?style=flat-square)
[![pre-commit](https://img.shields.io/badge/pre--commit-enabled-brightgreen?logo=pre-commit&style=flat-square)](https://github.com/pre-commit/pre-commit)
![NixOS](https://img.shields.io/badge/NixOS-5277C3?logo=nixos&logoColor=fff&style=flat-square)

Note: Only tested with [rofi-wayland](https://github.com/lbonn/rofi), although
it should work with the [official version](https://github.com/davatorium/rofi).

## Cliphist integration

A WIP integration with `cliphist` to show both text and images in a Rofi menu.
Requires [wl-clipboard](https://github.com/bugaevc/wl-clipboard) and of course
[cliphist](https://github.com/sentriz/cliphist).

Since I wanted to use different layouts/rofi configurations for texts and
images, neither the `script` mode nor a custom mode/plugin were valid options
because it's impossible to dynamically update the layout without re-launching
Rofi (more info [here](https://github.com/davatorium/rofi/issues/1356)).

### Usage

Just build the tool or download the [latest
release](https://github.com/szaffarano/rofi-tools/releases) and add the binary
to your path.

It's also exposed as a Nix flake. Add the input to your `flake.nix`

```nix
{
  inputs = {
    wofi-tools = {
      url = "github:szaffarano/rofi-tools";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  }
}
```

And reference the package as usual, e.g.,

```nix
home.packages = [
  inputs.rofi-tools.packages.${pkgs.system}.rofi-cliphist
];
```

And finally call it

```bash
path/to/cliphist-rofi
```

It will show by default all non-binary cliphist entries. You can switch between
text/image mode by using `Alt-t` / `Alt-i` and also delete entries using
`Alt-d`.

![Text Mode](./img/text-mode.png)

![Image Mode](./img/img-mode.png)
