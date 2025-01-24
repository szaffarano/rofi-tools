# rofi-tools

Note: Only tested with [rofi-wayland](https://github.com/lbonn/rofi) although
it should work with the [official version](https://github.com/davatorium/rofi).

## Cliphist integration

A WIP integration with `cliphist` to show both text and images in a Rofi menu.
Requires [wl-clipboard](https://github.com/bugaevc/wl-clipboard) and of course
[cliphist](https://github.com/sentriz/cliphist).

Since I wanted to use different layouts/rofi configurations for texts and
images, neither the `script` mode nor a custom mode/plugin were valid options
because it's impossible to dynamically update the layout without re-launching
Rofi (more info [here](https://github.com/davatorium/rofi/issues/1356)).

![Text Mode](./img/text-mode.png)

![Image Mode](./img/img-mode.png)
