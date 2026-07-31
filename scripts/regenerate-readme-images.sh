#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_dir=$(CDPATH= cd -- "$script_dir/.." && pwd)
cd "$repo_dir"

test -f Cargo.toml
test -f examples-400/DSC_1657.JPG

cargo build --release
paletteer="$repo_dir/target/release/paletteer"

for input in examples-400/DSC_*.JPG; do
    test -f "$input"
    "$paletteer" --theme everforest-dark-medium --format jpg --quality 80 --normalize-name \
        --lambda 1 --mix 1 --neutral-only --palette-name-only \
        --overwrite "$input"
done

for theme in \
    everforest-dark-medium \
    catppuccin-mocha \
    tokyo-night \
    gruvbox-dark-medium \
    nord \
    dracula \
    rose-pine-moon
do
    "$paletteer" --theme "$theme" --format jpg --quality 80 --normalize-name \
        --lambda 1 --mix 1 --palette-name-only \
        --overwrite examples-400/DSC_1657.JPG
done

echo "README images regenerated."
