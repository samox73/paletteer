# paletteer

Recolor PNG and JPEG wallpapers with the built-in `everforest-dark-medium` palette while preserving each pixel's source lightness. Outputs are written beside their inputs and never overwrite an existing file.

See [PALETTES.md](PALETTES.md) for the supported palette and its colors.

## Examples

<table>
<tr><td><img src="examples-400/DSC_1255.JPG" width="400" alt="Original DSC 1255"></td><td valign="middle">→</td><td><img src="examples-400/dsc-1255-everforest-dark-medium.jpg" width="400" alt="Everforest DSC 1255"></td></tr>
<tr><td><img src="examples-400/DSC_1261.JPG" width="400" alt="Original DSC 1261"></td><td valign="middle">→</td><td><img src="examples-400/dsc-1261-everforest-dark-medium.jpg" width="400" alt="Everforest DSC 1261"></td></tr>
<tr><td><img src="examples-400/DSC_1275.JPG" width="400" alt="Original DSC 1275"></td><td valign="middle">→</td><td><img src="examples-400/dsc-1275-everforest-dark-medium.jpg" width="400" alt="Everforest DSC 1275"></td></tr>
<tr><td><img src="examples-400/DSC_1321.JPG" width="400" alt="Original DSC 1321"></td><td valign="middle">→</td><td><img src="examples-400/dsc-1321-everforest-dark-medium.jpg" width="400" alt="Everforest DSC 1321"></td></tr>
<tr><td><img src="examples-400/DSC_1340.JPG" width="400" alt="Original DSC 1340"></td><td valign="middle">→</td><td><img src="examples-400/dsc-1340-everforest-dark-medium.jpg" width="400" alt="Everforest DSC 1340"></td></tr>
<tr><td><img src="examples-400/DSC_1344.JPG" width="400" alt="Original DSC 1344"></td><td valign="middle">→</td><td><img src="examples-400/dsc-1344-everforest-dark-medium.jpg" width="400" alt="Everforest DSC 1344"></td></tr>
<tr><td><img src="examples-400/DSC_1383.JPG" width="400" alt="Original DSC 1383"></td><td valign="middle">→</td><td><img src="examples-400/dsc-1383-everforest-dark-medium.jpg" width="400" alt="Everforest DSC 1383"></td></tr>
<tr><td><img src="examples-400/DSC_1484.JPG" width="400" alt="Original DSC 1484"></td><td valign="middle">→</td><td><img src="examples-400/dsc-1484-everforest-dark-medium.jpg" width="400" alt="Everforest DSC 1484"></td></tr>
<tr><td><img src="examples-400/DSC_1571.JPG" width="400" alt="Original DSC 1571"></td><td valign="middle">→</td><td><img src="examples-400/dsc-1571-everforest-dark-medium.jpg" width="400" alt="Everforest DSC 1571"></td></tr>
<tr><td><img src="examples-400/DSC_1629.JPG" width="400" alt="Original DSC 1629"></td><td valign="middle">→</td><td><img src="examples-400/dsc-1629-everforest-dark-medium.jpg" width="400" alt="Everforest DSC 1629"></td></tr>
<tr><td><img src="examples-400/DSC_1657.JPG" width="400" alt="Original DSC 1657"></td><td valign="middle">→</td><td><img src="examples-400/dsc-1657-everforest-dark-medium.jpg" width="400" alt="Everforest DSC 1657"></td></tr>
<tr><td><img src="examples-400/DSC_1719.JPG" width="400" alt="Original DSC 1719"></td><td valign="middle">→</td><td><img src="examples-400/dsc-1719-everforest-dark-medium.jpg" width="400" alt="Everforest DSC 1719"></td></tr>
<tr><td><img src="examples-400/DSC_1806.JPG" width="400" alt="Original DSC 1806"></td><td valign="middle">→</td><td><img src="examples-400/dsc-1806-everforest-dark-medium.jpg" width="400" alt="Everforest DSC 1806"></td></tr>
<tr><td><img src="examples-400/DSC_1812.JPG" width="400" alt="Original DSC 1812"></td><td valign="middle">→</td><td><img src="examples-400/dsc-1812-everforest-dark-medium.jpg" width="400" alt="Everforest DSC 1812"></td></tr>
<tr><td><img src="examples-400/DSC_1915.JPG" width="400" alt="Original DSC 1915"></td><td valign="middle">→</td><td><img src="examples-400/dsc-1915-everforest-dark-medium.jpg" width="400" alt="Everforest DSC 1915"></td></tr>
<tr><td><img src="examples-400/DSC_2080.JPG" width="400" alt="Original DSC 2080"></td><td valign="middle">→</td><td><img src="examples-400/dsc-2080-everforest-dark-medium.jpg" width="400" alt="Everforest DSC 2080"></td></tr>
<tr><td><img src="examples-400/DSC_2311.JPG" width="400" alt="Original DSC 2311"></td><td valign="middle">→</td><td><img src="examples-400/dsc-2311-everforest-dark-medium.jpg" width="400" alt="Everforest DSC 2311"></td></tr>
<tr><td><img src="examples-400/DSC_2316.JPG" width="400" alt="Original DSC 2316"></td><td valign="middle">→</td><td><img src="examples-400/dsc-2316-everforest-dark-medium.jpg" width="400" alt="Everforest DSC 2316"></td></tr>
<tr><td><img src="examples-400/DSC_2357.JPG" width="400" alt="Original DSC 2357"></td><td valign="middle">→</td><td><img src="examples-400/dsc-2357-everforest-dark-medium.jpg" width="400" alt="Everforest DSC 2357"></td></tr>
</table>

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
