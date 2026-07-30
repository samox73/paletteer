# paletteer

Recolor PNG and JPEG wallpapers with the built-in `everforest-dark-medium` palette while preserving each pixel's source lightness. Outputs are written beside their inputs and never overwrite an existing file.

```sh
cargo install --path .
paletteer -t everforest-dark-medium wallpaper.jpg
paletteer -t everforest-dark-medium -f webp -q 85 wallpaper.jpg
paletteer -t everforest-dark-medium -f jpg -q 90 wallpaper.png
paletteer -t everforest-dark-medium images/
paletteer -t everforest-dark-medium -n 'wallpapers/**/*-forest.{png,jpg,jpeg}'
```

Directories include immediate PNG, JPG, and JPEG files only. Quote recursive globs for consistent shell behavior; unquoted globs work when the shell expands them.

`--format` / `-f` accepts `png` (the lossless default), `webp`, or `jpg`. `--quality` / `-q` accepts 1–100 for WebP and JPEG (default 80); PNG rejects it. JPEG does not support alpha, so it discards it.
