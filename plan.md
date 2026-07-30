# Paletteer prototype implementation plan

This document is intentionally explicit so a smaller implementation model can
build the prototype without inventing product or architecture decisions.

## Fixed decisions

- Project and binary name: `paletteer`.
- Language: Rust, stable toolchain, edition 2024.
- Built-in palette: `everforest-dark-medium`.
- Accepted inputs: PNG, JPEG, and directories containing PNG/JPEG files.
- Output: lossless PNG beside each input.
- Output name: `<input-stem>-<palette-name>.png`.
- `--normalize-name` / `-n` normalizes the complete output stem.
- Never overwrite an existing file.
- Preserve PNG alpha.
- A directory argument means its immediate files only, not recursive traversal.
  Recursive selection is expressed with a glob.

## Recoloring behavior

Do not reduce the output to the literal Everforest colors. That would replace
continuous brightness gradients with a few discrete brightness levels and
produce heavy color bands.

For every non-transparent pixel:

1. Convert its sRGB channels to Oklab.
2. Find the nearest Everforest color using squared Euclidean distance across
   Oklab `L`, `a`, and `b`. Do not use RGB distance.
3. Form a new Oklab color using:
   - `L` from the original pixel;
   - `a` and `b` from the selected Everforest color.
4. Convert the result back to sRGB.
5. Clamp out-of-gamut channels to the valid sRGB range.
6. Copy the original alpha channel unchanged.

This preserves the source's smooth lightness transitions while constraining
hue and chroma to Everforest-derived values. The output is therefore
Everforest-themed, but its pixels are shades derived from the palette rather
than exact palette entries. Minor clipping in unusually bright, saturated
pixels is acceptable for the prototype.

Use the core colors from the official dark-medium palette:

- backgrounds: `bg_dim`, `bg0` through `bg5`;
- foreground: `fg`;
- accents: `red`, `orange`, `yellow`, `green`, `aqua`, `blue`, `purple`;
- greys: `grey0`, `grey1`, `grey2`.

Exclude `bg_visual`, `bg_red`, `bg_yellow`, `bg_green`, `bg_blue`,
`bg_purple`, and the duplicate status-line colors. Copy hex values from the
official source and add a source URL comment above the constant:
<https://github.com/sainnhe/everforest/blob/master/palette.md>.

## CLI contract

```text
paletteer --theme <THEME> [--normalize-name] <INPUT>...
```

- `-t`, `--theme <THEME>` is required.
- The only accepted theme value is `everforest-dark-medium`.
- `-n`, `--normalize-name` takes no value.
- `<INPUT>...` requires one or more arguments.
- Each argument may be:
  - a PNG or JPEG file;
  - a directory;
  - an unexpanded glob containing `*`, `?`, or `[...]`.
- Accept multiple paths because Unix shells and Nushell commonly expand globs
  before starting the program.
- If a glob reaches the program unexpanded, expand it internally. This supports
  quoted patterns and shells that do not expand them.
- For consistent recursive behavior in every shell, document the quoted form:

```sh
paletteer -t everforest-dark-medium '**/*-forest.png'
```

- The unquoted form also works when the shell supports/enables recursive
  `**` expansion:

```sh
paletteer -t everforest-dark-medium **/*-forest.png
```

The program cannot recover an original pattern after a shell has partially
expanded it. In Bash, users must quote recursive globs or enable `globstar`.

Examples:

```sh
paletteer -t everforest-dark-medium wallpaper.jpg
paletteer -t everforest-dark-medium images/
paletteer -t everforest-dark-medium -n 'wallpapers/**/*-forest.{png,jpg,jpeg}'
```

## Output naming

Without `-n`, retain the input stem exactly and append the canonical palette
name:

```text
Misty Forest (Final).jpg
-> Misty Forest (Final)-everforest-dark-medium.png
```

With `-n`, normalize the complete stem after appending the palette:

1. Convert ASCII `A-Z` to `a-z`.
2. Keep only `a-z`, `0-9`, and `-`.
3. Replace each consecutive run of all other characters with one `-`.
4. Collapse consecutive `-` characters.
5. Remove leading and trailing `-`.
6. Append exactly `.png`.
7. Return an error if the normalized stem is empty.

Do not transliterate Unicode. It belongs to the disallowed run and becomes
`-`. A dot is allowed only as the separator before the generated `png`
extension; dots in the source stem become `-`.

Examples:

```text
Misty Forest (Final).JPG
-> misty-forest-final-everforest-dark-medium.png

foo.bar.png
-> foo-bar-everforest-dark-medium.png
```

Before decoding any image, collect all jobs and preflight them:

- remove duplicate input paths;
- reject unsupported files;
- reject globs with no matches;
- reject an empty directory with no supported images;
- reject two inputs that resolve to the same output path;
- reject every output path that already exists.

Only begin writing after the whole batch passes preflight. Write each image to
a sibling temporary file and rename it to the final output only after encoding
succeeds, so an encoder failure cannot leave a truncated final file.

## Dependencies

Keep the dependency list to these crates:

- `clap` with `derive`: argument parsing and generated help.
- `image` with default features disabled and only `png` and `jpeg` enabled:
  decoding and PNG encoding.
- `palette`: sRGB/Oklab conversion.
- `glob`: expansion of patterns that reach the binary unexpanded.

Use the Rust standard library for paths, directory enumeration, errors,
deduplication, temporary sibling names, and file renaming. Do not add
`anyhow`, `thiserror`, `walkdir`, `rayon`, `serde`, a logging framework, or a
test helper crate.

## Minimal file layout

```text
Cargo.toml
src/
  main.rs
  recolor.rs
README.md
```

- `main.rs`: CLI types, input/glob/directory expansion, output naming,
  preflight, error printing, and job execution.
- `recolor.rs`: Everforest constants, Oklab nearest-color mapping, image
  decoding, PNG encoding, and pixel-level tests.
- `README.md`: installation, CLI examples, supported formats, shell glob note,
  non-overwrite behavior, and the brightness-preserving algorithm in one short
  section.

Do not create a library crate, traits, palette registry, configuration module,
command subcommands, or one-file-per-type structure.

## Implementation sequence

### 1. Scaffold the binary

Create `Cargo.toml` and `src/main.rs`. Define the `clap` arguments exactly as
specified. Confirm:

```sh
cargo run -- --help
cargo run -- 2>/dev/null
```

The second command must fail because theme and input are required.

### 2. Implement recoloring

In `src/recolor.rs`:

- store the selected Everforest colors as `&'static [Srgb<f32>]` or the
  simplest equivalent supported by the chosen `palette` version;
- precompute their Oklab representations once per image, not per pixel;
- decode an input to RGBA8;
- iterate pixels in place;
- skip color conversion for fully transparent pixels and preserve all alpha;
- find the nearest palette entry without square roots;
- replace `L` as described above;
- save as PNG.

Do not add dithering, quantization, clustering, lookup tables, SIMD, or
parallel processing.

### 3. Implement input collection and naming

In `main.rs`:

- expand each positional argument;
- enumerate immediate children for directory arguments;
- keep `.png`, `.jpg`, and `.jpeg` case-insensitively;
- sort discovered paths for deterministic processing order;
- deduplicate paths;
- generate and preflight outputs;
- process sequentially;
- print one line per successful output path.

An explicitly supplied unsupported file is an error. Unsupported files found
while scanning a directory are ignored.

### 4. Handle failures safely

- Exit nonzero on every invalid argument, discovery, decoding, encoding, or
  filesystem error.
- Include the relevant path in error messages.
- Stop at the first processing failure.
- Save to a unique sibling name such as
  `.<output-file-name>.<process-id>.tmp`, then rename.
- Remove that temporary file if encoding fails.
- Do not remove any successfully completed earlier outputs if a later job
  fails.

### 5. Add focused tests

Use built-in Rust tests only.

Required tests:

- normalization table covering spaces, uppercase, repeated separators, dots,
  Unicode, and an empty normalized stem;
- output naming for JPEG and PNG sources;
- nearest-color selection with a few synthetic Oklab values;
- a generated grayscale gradient produces more distinct output colors than
  the number of literal palette entries, proving that source lightness is
  preserved instead of quantized;
- RGBA recoloring preserves alpha values;
- duplicate output detection for `same.jpg` plus `same.png`.

Tests must generate pixels and temporary paths at runtime. Do not commit binary
image fixtures or add a temporary-directory dependency.

### 6. Document and verify

Write the concise `README.md`, then run:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -- -t everforest-dark-medium --normalize-name <a-small-test-image>
```

Inspect the generated PNG manually for obvious hue errors, preserved gradients,
and the expected name. Do not commit the generated test output.

## Explicitly out of scope

Do not implement any of the following in this prototype:

- YAML, TOML, JSON, or other external palette files;
- a generic palette schema or parser prepared for future formats;
- exact-palette quantization;
- dithering;
- recursive directory traversal;
- output format selection or JPEG output;
- in-place overwrite or `--force`;
- custom output directories;
- metadata, EXIF orientation, ICC profile, or animation preservation;
- GIF, WebP, TIFF, SVG, HEIF, or RAW support;
- multithreading, GPU processing, SIMD, or performance caches;
- progress bars, colored logs, shell completions, configuration files, or
  environment variables.

Add external palette loading only after the built-in algorithm and CLI behavior
have been validated on real wallpapers. At that point, choose exactly one
markup format based on actual palette files users want to load; do not build
both YAML and TOML preemptively.

## Completion criteria

The prototype is complete when:

- all verification commands pass;
- PNG and JPEG files can be selected individually, through a directory, or
  through expanded/unexpanded globs;
- every output is a sibling PNG named with the palette suffix;
- `-n` produces only `[a-z0-9-]+.png`;
- brightness gradients remain gradual;
- alpha is preserved;
- no existing output is overwritten;
- failures return a nonzero exit code with a path-specific message.
