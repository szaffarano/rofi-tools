# Rofi Tools

![GitHub Release](https://img.shields.io/github/v/release/szaffarano/rofi-tools?sort=date&style=flat-square)
![GitHub License](https://img.shields.io/github/license/szaffarano/rofi-tools?style=flat-square)
[![pre-commit](https://img.shields.io/badge/pre--commit-enabled-brightgreen?logo=pre-commit&style=flat-square)](https://github.com/pre-commit/pre-commit)
![NixOS](https://img.shields.io/badge/NixOS-5277C3?logo=nixos&logoColor=fff&style=flat-square)

Note: Only tested with [rofi-wayland](https://github.com/lbonn/rofi), although
it should work with the [official version](https://github.com/davatorium/rofi).

## Installation

### Nix

#### Option 1: Using flakes (recommended)

If you have flakes enabled:

```bash
# Install directly
nix profile install github:szaffarano/rofi-tools

# Or run without installing
nix run github:szaffarano/rofi-tools
```

#### Option 2: Add to your NixOS configuration

Add to your `configuration.nix`:

```nix
{
  inputs.rofi-tools.url = "github:szaffarano/rofi-tools";

  outputs = { nixpkgs, rofi-tools, ... }: {
    # Your existing config...

    environment.systemPackages = with pkgs; [
      # Your other packages...
      rofi-tools.packages.${system}.default
    ];
  };
}
```

### Arch Linux

You can install this project using an [unofficial AUR
package](https://aur.archlinux.org/packages/rofi-tools-bin) (Thanks
[@a-priestley](https://github.com/a-priestley)):

```bash
yay -S rofi-tools
```

Alternatively, you can build from the AUR manually:

```bash
git clone https://aur.archlinux.org/rofi-tools-bin.git
cd rofi-tools-bin
makepkg -si
```

### Build from Source

If your distribution isn't listed above, you can build from source:

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Clone the repository
git clone https://github.com/szaffarano/rofi-tools.git
cd rofi-tools

# Build and install
cargo install --path .
```

Requirements:

- Rust 1.70 or newer
- [rofi](https://github.com/davatorium/rofi) or
  [rofi-wayland](https://github.com/lbonn/rofi) installed on your system
- [wl-clipboard](https://github.com/bugaevc/wl-clipboard)
- [cliphist](https://github.com/sentriz/cliphist) (for `rofi-cliphist`)
- [clipcat](https://github.com/xrelkd/clipcat) (for `rofi-clipcat`)
- `protobuf-compiler` (for `rofi-clipcat` only)

## Cliphist integration

A WIP integration with `cliphist` to show both text and images in a Rofi menu.
Requires [wl-clipboard](https://github.com/bugaevc/wl-clipboard) and of course
[cliphist](https://github.com/sentriz/cliphist).

Since I wanted to use different layouts/rofi configurations for texts and
images, neither the `script` mode nor a custom mode/plugin were valid options
because it's impossible to dynamically update the layout without re-launching
Rofi (more [info here](https://github.com/davatorium/rofi/issues/1356)).

### rofi-cliphist Usage

Just build the tool or download the [latest
release](https://github.com/szaffarano/rofi-tools/releases) and add the binary
to your path.

It's also exposed as a Nix flake. Add the input to your `flake.nix`

```nix
{
  inputs = {
    rofi-tools = {
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
rofi-cliphist
```

It will show by default all non-binary cliphist entries. You can switch between
text/image mode by using `Alt-t` / `Alt-i` and also delete entries using
`Alt-d`, `Alt-p` (delete previous) and `Alt-n` (delete next).

![Text Mode](./img/text-mode.png)

![Image Mode](./img/img-mode.png)

### rofi-cliphist Configuration

- Through the command line:

```bash
Usage: rofi-cliphist [OPTIONS]

Options:
  -v, --verbose                          Show verbose output
  -r, --rofi-path <ROFI_PATH>            Path to rofi executable [default: rofi]
  -c, --cliphist-path <CLIPHIST_PATH>    Path to cliphist executable [default: cliphist]
  -w, --clipboard-path <CLIPBOARD_PATH>  Path to wl-copy executable [default: wl-copy]
  -f, --config <FILE>                    Sets a custom config file
  -t, --theme <THEME>                    Sets a custom theme
  -h, --help                             Print help
  -V, --version                          Print version
```

- Through a configuration file, either on `$XDG_CONFIG_HOME/rofi-cliphist.toml`
or the path specified using the `-f/--config` flag

```toml
[rofi]
path = "rofi"

[cliphist]
path = "cliphist"

[clipboard]
path = "wl-copy"

[text_mode_config]
title = "Text"
shortcut = "Alt+t"
description = "Switch to text mode"

[image_mode_config]
title = "Image"
shortcut = "Alt+i"
description = "Switch to image mode!"

[delete_mode_config]
title = "Delete"
shortcut = "Alt+d"
description = "Delete entry"

[delete_previous_config]
title = "Delete previous"
shortcut = "Alt+p"
description = "Delete all entries before the selected one"

[delete_next_config]
title = "Delete next"
shortcut = "Alt+n"
description = "Delete all entries after the selected one"

[toggle_mode_config]
title = "Toggle"
shortcut = "Alt+m"
description = "Toggle text/image mode"
```

## Clipcat integration

A clipboard history picker backed by the
[clipcat](https://github.com/xrelkd/clipcat) daemon, providing clipboard
management with image preview support through a Rofi interface.

Unlike `rofi-cliphist`, which uses the `cliphist` command-line tool,
`rofi-clipcat` connects directly to the clipcat daemon via gRPC, offering a
more integrated experience with clipcat's features.

### rofi-clipcat Usage

Build the tool or download the [latest
release](https://github.com/szaffarano/rofi-tools/releases) and add the binary
to your path.

It's also exposed as a Nix flake:

```nix
{
  inputs = {
    rofi-tools = {
      url = "github:szaffarano/rofi-tools";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  }
}
```

Reference the package:

```nix
home.packages = [
  inputs.rofi-tools.packages.${pkgs.system}.rofi-clipcat
];
```

Then run:

```bash
rofi-clipcat
```

The tool displays clipboard history entries from the clipcat daemon. You can
switch between text/image modes using `Alt-t` / `Alt-i` and delete entries
using `Alt-d`, `Alt-p` (delete previous), and `Alt-n` (delete next), similar to
`rofi-cliphist`.

### rofi-clipcat Configuration

- Through the command line:

```bash
Usage: rofi-clipcat [OPTIONS]

Options:
  -v, --verbose                 Show verbose output
  -r, --rofi-path <ROFI_PATH>   Path to rofi executable [default: rofi]
  -e, --endpoint <ENDPOINT>     Clipcat gRPC endpoint [default: unix socket]
  -a, --access-token <TOKEN>    Clipcat access token
  -f, --config <FILE>           Sets a custom config file
  -t, --theme <THEME>           Sets a custom theme
  -h, --help                    Print help
  -V, --version                 Print version
```

- Through a configuration file at `$XDG_CONFIG_HOME/rofi-clipcat.toml` or the
  path specified using the `-f/--config` flag:

```toml
[rofi]
path = "rofi"

[clipcat]
endpoint = "unix:///run/user/1000/clipcat/grpc.sock"  # default socket path
access_token = ""  # optional

[text_mode_config]
title = "Text"
shortcut = "Alt+t"
description = "Switch to text mode"

[image_mode_config]
title = "Image"
shortcut = "Alt+i"
description = "Switch to image mode"

[delete_mode_config]
title = "Delete"
shortcut = "Alt+d"
description = "Delete entry"

[delete_previous_config]
title = "Delete previous"
shortcut = "Alt+p"
description = "Delete all entries before the selected one"

[delete_next_config]
title = "Delete next"
shortcut = "Alt+n"
description = "Delete all entries after the selected one"

[toggle_mode_config]
title = "Toggle"
shortcut = "Alt+m"
description = "Toggle text/image mode"
```

### Build Requirements

Building `rofi-clipcat` from source requires the Protocol Buffers compiler
(`protoc`) due to its gRPC dependencies:

```bash
# Debian/Ubuntu
sudo apt-get install protobuf-compiler

# Arch Linux
sudo pacman -S protobuf

# Fedora
sudo dnf install protobuf-compiler

# macOS
brew install protobuf
```

The Nix flake automatically handles this dependency.
