# Supported palettes

Paletteer supports these palettes:

| Name | Official source |
| --- | --- |
| `everforest-dark-medium` | [Everforest](https://github.com/sainnhe/everforest/blob/master/palette.md) |
| `catppuccin-mocha` | [Catppuccin Mocha](https://github.com/catppuccin/palette/blob/main/palette.json) |
| `tokyo-night` | [Tokyo Night](https://github.com/folke/tokyonight.nvim/blob/main/lua/tokyonight/colors/night.lua) |
| `gruvbox-dark-medium` | [Gruvbox Dark Medium](https://github.com/morhetz/gruvbox/blob/master/colors/gruvbox.vim) |
| `nord` | [Nord](https://www.nordtheme.com/docs/colors-and-palettes) |
| `dracula` | [Dracula](https://draculatheme.com/contribute) |
| `rose-pine-moon` | [Rosé Pine Moon](https://github.com/rose-pine/rose-pine-palette) |

The color names and values below come from the [official Everforest palette](https://github.com/sainnhe/everforest/blob/master/palette.md). Paletteer uses these colors to select hue and chroma; it preserves the source image's lightness.

Neutral and accent colors are used by default. `--neutral-only` excludes the
accent colors.

## Custom palette files

Custom palettes use TOML:

```toml
name = "forest-dusk"
colors = ["#1b2428", "#556b62", "#d8c9aa"]
accents = ["#e67e80", "#a7c080", "#7fbbb3"]
```

`name` must contain only `a-z`, `0-9`, and single hyphens. `colors` is a
non-empty array of `#RRGGBB` strings. `accents` is optional and included by
default; `--neutral-only` excludes it.
