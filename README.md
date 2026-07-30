# paletteer

Recolor PNG and JPEG wallpapers with the built-in `everforest-dark-medium` palette while preserving each pixel's source lightness. Outputs are written beside their inputs and never overwrite an existing file.

See [PALETTES.md](PALETTES.md) for the supported palette and its colors.

## Examples

[![Original DSC 1255](examples-400/DSC_1255.JPG)](examples/DSC_1255.JPG) → [![Everforest DSC 1255](examples-400/dsc-1255-everforest-dark-medium.jpg)](examples/dsc-1255-everforest-dark-medium.jpg)
[![Original DSC 1261](examples-400/DSC_1261.JPG)](examples/DSC_1261.JPG) → [![Everforest DSC 1261](examples-400/dsc-1261-everforest-dark-medium.jpg)](examples/dsc-1261-everforest-dark-medium.jpg)
[![Original DSC 1275](examples-400/DSC_1275.JPG)](examples/DSC_1275.JPG) → [![Everforest DSC 1275](examples-400/dsc-1275-everforest-dark-medium.jpg)](examples/dsc-1275-everforest-dark-medium.jpg)
[![Original DSC 1321](examples-400/DSC_1321.JPG)](examples/DSC_1321.JPG) → [![Everforest DSC 1321](examples-400/dsc-1321-everforest-dark-medium.jpg)](examples/dsc-1321-everforest-dark-medium.jpg)
[![Original DSC 1340](examples-400/DSC_1340.JPG)](examples/DSC_1340.JPG) → [![Everforest DSC 1340](examples-400/dsc-1340-everforest-dark-medium.jpg)](examples/dsc-1340-everforest-dark-medium.jpg)
[![Original DSC 1344](examples-400/DSC_1344.JPG)](examples/DSC_1344.JPG) → [![Everforest DSC 1344](examples-400/dsc-1344-everforest-dark-medium.jpg)](examples/dsc-1344-everforest-dark-medium.jpg)
[![Original DSC 1383](examples-400/DSC_1383.JPG)](examples/DSC_1383.JPG) → [![Everforest DSC 1383](examples-400/dsc-1383-everforest-dark-medium.jpg)](examples/dsc-1383-everforest-dark-medium.jpg)
[![Original DSC 1484](examples-400/DSC_1484.JPG)](examples/DSC_1484.JPG) → [![Everforest DSC 1484](examples-400/dsc-1484-everforest-dark-medium.jpg)](examples/dsc-1484-everforest-dark-medium.jpg)
[![Original DSC 1571](examples-400/DSC_1571.JPG)](examples/DSC_1571.JPG) → [![Everforest DSC 1571](examples-400/dsc-1571-everforest-dark-medium.jpg)](examples/dsc-1571-everforest-dark-medium.jpg)
[![Original DSC 1629](examples-400/DSC_1629.JPG)](examples/DSC_1629.JPG) → [![Everforest DSC 1629](examples-400/dsc-1629-everforest-dark-medium.jpg)](examples/dsc-1629-everforest-dark-medium.jpg)
[![Original DSC 1657](examples-400/DSC_1657.JPG)](examples/DSC_1657.JPG) → [![Everforest DSC 1657](examples-400/dsc-1657-everforest-dark-medium.jpg)](examples/dsc-1657-everforest-dark-medium.jpg)
[![Original DSC 1719](examples-400/DSC_1719.JPG)](examples/DSC_1719.JPG) → [![Everforest DSC 1719](examples-400/dsc-1719-everforest-dark-medium.jpg)](examples/dsc-1719-everforest-dark-medium.jpg)
[![Original DSC 1806](examples-400/DSC_1806.JPG)](examples/DSC_1806.JPG) → [![Everforest DSC 1806](examples-400/dsc-1806-everforest-dark-medium.jpg)](examples/dsc-1806-everforest-dark-medium.jpg)
[![Original DSC 1812](examples-400/DSC_1812.JPG)](examples/DSC_1812.JPG) → [![Everforest DSC 1812](examples-400/dsc-1812-everforest-dark-medium.jpg)](examples/dsc-1812-everforest-dark-medium.jpg)
[![Original DSC 1915](examples-400/DSC_1915.JPG)](examples/DSC_1915.JPG) → [![Everforest DSC 1915](examples-400/dsc-1915-everforest-dark-medium.jpg)](examples/dsc-1915-everforest-dark-medium.jpg)
[![Original DSC 2080](examples-400/DSC_2080.JPG)](examples/DSC_2080.JPG) → [![Everforest DSC 2080](examples-400/dsc-2080-everforest-dark-medium.jpg)](examples/dsc-2080-everforest-dark-medium.jpg)
[![Original DSC 2311](examples-400/DSC_2311.JPG)](examples/DSC_2311.JPG) → [![Everforest DSC 2311](examples-400/dsc-2311-everforest-dark-medium.jpg)](examples/dsc-2311-everforest-dark-medium.jpg)
[![Original DSC 2316](examples-400/DSC_2316.JPG)](examples/DSC_2316.JPG) → [![Everforest DSC 2316](examples-400/dsc-2316-everforest-dark-medium.jpg)](examples/dsc-2316-everforest-dark-medium.jpg)
[![Original DSC 2357](examples-400/DSC_2357.JPG)](examples/DSC_2357.JPG) → [![Everforest DSC 2357](examples-400/dsc-2357-everforest-dark-medium.jpg)](examples/dsc-2357-everforest-dark-medium.jpg)

```sh
cargo install --path .
paletteer -t everforest-dark-medium wallpaper.jpg
paletteer -t everforest-dark-medium -f webp -q 85 wallpaper.jpg
paletteer -t everforest-dark-medium -f jpg -q 90 wallpaper.png
paletteer -t everforest-dark-medium --accents wallpaper.jpg
paletteer -t everforest-dark-medium --overwrite wallpaper.jpg
paletteer -t everforest-dark-medium images/
paletteer -t everforest-dark-medium -n 'wallpapers/**/*-forest.{png,jpg,jpeg}'
```

Directories include immediate PNG, JPG, and JPEG files only. Quote recursive globs for consistent shell behavior; unquoted globs work when the shell expands them.

`--format` / `-f` accepts `png` (the lossless default), `webp`, or `jpg`. `--quality` / `-q` accepts 1–100 for WebP and JPEG (default 80); PNG rejects it. JPEG does not support alpha, so it discards it.

Accent colors are disabled by default to avoid shifting natural highlights to unrelated hues. Use `--accents` / `-a` to include them.

Existing outputs are rejected by default. Use `--overwrite` / `-o` to replace them after successful encoding.

Files whose names already end in `-everforest-dark-medium` are skipped, preventing accidental repeat recoloring when processing directories or globs.
