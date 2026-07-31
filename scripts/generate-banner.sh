#!/usr/bin/env bash
# Build banner.png: crop DSC_1657, recolor it with all 7 built-in palettes,
# arrange original + palettes as 8 angular octants around the center, and
# overlay the title. Needs ImageMagick (magick) plus the paletteer binary.
set -euo pipefail

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_dir=$(CDPATH= cd -- "$script_dir/.." && pwd)
cd "$repo_dir"

src=assets/DSC_1657.JPG
out=banner.png
W=1500 H=500 CX=750 CY=250 R=6000            # R >> image diagonal, so wedges clip exactly
font=${BANNER_FONT:-DejaVu-Sans-Bold}
pointsize=${BANNER_POINTSIZE:-190}
border_color=${BANNER_BORDER:-#BB9AF7}         # Tokyo Night violet, fits the banner's cast
border_width=${BANNER_BORDER_WIDTH:-6}
themes=(everforest-dark-medium catppuccin-mocha tokyo-night gruvbox-dark-medium nord dracula rose-pine-moon)

test -f "$src"
cargo build --release
paletteer="$repo_dir/target/release/paletteer"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# Crop to fill 1500x500 (source is small, so upscale then gently sharpen).
magick "$src" -resize "${W}x${H}^" -gravity center -extent "${W}x${H}" \
    -unsharp 0x1 "$tmp/base.png"

# One recolor per palette; imgs[0] stays the untouched original -> 8 octants.
imgs=("$tmp/base.png")
for theme in "${themes[@]}"; do
    "$paletteer" --theme "$theme" --format png --lambda 0.5 --mix 1 \
        --palette-name-only --overwrite "$tmp/base.png"
    imgs+=("$tmp/base-$theme.png")
done

# Composite each image through its 45-degree wedge mask.
cp "${imgs[0]}" "$out"
for i in "${!imgs[@]}"; do
    # Equal 45-degree wedges. R stays well below the rasterizer's coordinate
    # limits; scaling the rays by W/H here overflows it and corrupts the masks.
    coords=$(awk -v cx="$CX" -v cy="$CY" -v r="$R" -v i="$i" \
        'BEGIN{pi=atan2(0,-1);a0=i*45*pi/180;a1=(i+1)*45*pi/180;
               printf "%.1f,%.1f %.1f,%.1f",cx+r*cos(a0),cy+r*sin(a0),cx+r*cos(a1),cy+r*sin(a1)}')
    magick -size "${W}x${H}" xc:black -fill white -draw "polygon $CX,$CY $coords" "$tmp/mask.png"
    magick "$out" "${imgs[$i]}" "$tmp/mask.png" -compose Over -composite "$out"
done

# Centered title: black outline pass, then clean white fill pass.
magick "$out" -gravity center -font "$font" -pointsize "$pointsize" \
    -stroke black -strokewidth 12 -fill white -annotate 0 Paletteer \
    -stroke none -fill white -annotate 0 Paletteer \
    "$out"

# Thin violet frame drawn inside the canvas so dimensions stay ${W}x${H}.
half=$((border_width / 2))
magick "$out" -fill none -stroke "$border_color" -strokewidth "$border_width" \
    -draw "rectangle $half,$half $((W - 1 - half)),$((H - 1 - half))" \
    "$out"

echo "wrote $out (${W}x${H})"
