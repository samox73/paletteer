mod recolor;

use clap::{Parser, ValueEnum};
use glob::glob;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    process,
};

const DEFAULT_QUALITY: u8 = 80;
const DEFAULT_LAMBDA: f32 = 0.25;
const THEME_NAMES: &[&str] = &[
    "everforest-dark-medium",
    "catppuccin-mocha",
    "tokyo-night",
    "gruvbox-dark-medium",
    "nord",
    "dracula",
    "rose-pine-moon",
];

#[derive(Clone, Copy, ValueEnum)]
pub enum Theme {
    #[value(name = "everforest-dark-medium")]
    EverforestDarkMedium,
    #[value(name = "catppuccin-mocha")]
    CatppuccinMocha,
    #[value(name = "tokyo-night")]
    TokyoNight,
    #[value(name = "gruvbox-dark-medium")]
    GruvboxDarkMedium,
    Nord,
    Dracula,
    #[value(name = "rose-pine-moon")]
    RosePineMoon,
}

impl Theme {
    fn name(self) -> &'static str {
        match self {
            Self::EverforestDarkMedium => "everforest-dark-medium",
            Self::CatppuccinMocha => "catppuccin-mocha",
            Self::TokyoNight => "tokyo-night",
            Self::GruvboxDarkMedium => "gruvbox-dark-medium",
            Self::Nord => "nord",
            Self::Dracula => "dracula",
            Self::RosePineMoon => "rose-pine-moon",
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Png,
    Webp,
    Jpg,
}

impl OutputFormat {
    fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Webp => "webp",
            Self::Jpg => "jpg",
        }
    }

    fn supports_quality(self) -> bool {
        !matches!(self, Self::Png)
    }
}

#[derive(Parser)]
#[command(
    name = "paletteer",
    about = "Recolor images with built-in or custom palettes",
    group = clap::ArgGroup::new("theme-source")
        .required(true)
        .args(["theme", "theme_file"])
)]
struct Cli {
    #[arg(short, long, value_enum)]
    theme: Option<Theme>,
    #[arg(long, value_name = "FILE")]
    theme_file: Option<PathBuf>,
    #[arg(short = 'n', long)]
    normalize_name: bool,
    #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Png)]
    format: OutputFormat,
    #[arg(short = 'q', long, value_parser = clap::value_parser!(u8).range(1..=100))]
    quality: Option<u8>,
    #[arg(long, help = "Exclude the palette's accent colors")]
    neutral_only: bool,
    #[arg(long, help = "Keep only the palette name in the output suffix")]
    palette_name_only: bool,
    #[arg(
        long,
        default_value_t = DEFAULT_LAMBDA,
        value_parser = parse_lambda,
        help = "Weight palette lightness during color matching (0 to 1)"
    )]
    lambda: f32,
    #[arg(
        long,
        default_value_t = 1.0,
        value_parser = parse_mix,
        help = "Blend original and remapped colors (0 = original, 1 = fully remapped)"
    )]
    mix: f32,
    #[arg(short = 'o', long, help = "Replace existing output files")]
    overwrite: bool,
    #[arg(required = true)]
    input: Vec<PathBuf>,
}

struct SelectedPalette {
    name: String,
    colors: Vec<u32>,
}

struct CustomPalette {
    name: String,
    colors: Vec<u32>,
    accents: Vec<u32>,
}

fn parse_mix(value: &str) -> Result<f32, String> {
    let mix = value
        .parse::<f32>()
        .map_err(|_| format!("invalid mix value '{value}': expected a number from 0 to 1"))?;
    if mix.is_finite() && (0.0..=1.0).contains(&mix) {
        Ok(if mix == 0.0 { 0.0 } else { mix })
    } else {
        Err(format!(
            "invalid mix value '{value}': expected a number from 0 to 1"
        ))
    }
}

fn parse_lambda(value: &str) -> Result<f32, String> {
    let lambda = value
        .parse::<f32>()
        .map_err(|_| format!("invalid lambda '{value}': expected a number from 0 to 1"))?;
    if lambda.is_finite() && (0.0..=1.0).contains(&lambda) {
        Ok(if lambda == 0.0 { 0.0 } else { lambda })
    } else {
        Err(format!(
            "invalid lambda '{value}': expected a number from 0 to 1"
        ))
    }
}

fn parse_colors(document: &toml::Table, field: &str, required: bool) -> Result<Vec<u32>, String> {
    let Some(values) = document.get(field) else {
        return if required {
            Err(format!("missing required '{field}' array"))
        } else {
            Ok(Vec::new())
        };
    };
    let values = values
        .as_array()
        .ok_or_else(|| format!("'{field}' must be an array of hex color strings"))?;
    if required && values.is_empty() {
        return Err(format!("'{field}' must contain at least one color"));
    }
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let value = value
                .as_str()
                .ok_or_else(|| format!("'{field}[{index}]' must be a hex color string"))?;
            let Some(hex) = value.strip_prefix('#').filter(|hex| hex.len() == 6) else {
                return Err(format!(
                    "invalid color '{value}' in '{field}[{index}]': expected #RRGGBB"
                ));
            };
            u32::from_str_radix(hex, 16).map_err(|_| {
                format!("invalid color '{value}' in '{field}[{index}]': expected #RRGGBB")
            })
        })
        .collect()
}

fn parse_custom_palette(text: &str) -> Result<CustomPalette, String> {
    let document = text
        .parse::<toml::Table>()
        .map_err(|error| error.to_string())?;
    if let Some(field) = document
        .keys()
        .find(|field| !matches!(field.as_str(), "name" | "colors" | "accents"))
    {
        return Err(format!("unknown palette field '{field}'"));
    }
    let name = document
        .get("name")
        .and_then(toml::Value::as_str)
        .ok_or_else(|| "missing required string 'name'".to_owned())?;
    if normalized(name).ok().as_deref() != Some(name) {
        return Err("'name' must contain only a-z, 0-9, and single hyphens".to_owned());
    }
    Ok(CustomPalette {
        name: name.to_owned(),
        colors: parse_colors(&document, "colors", true)?,
        accents: parse_colors(&document, "accents", false)?,
    })
}

fn select_palette(cli: &Cli) -> Result<SelectedPalette, String> {
    if let Some(path) = &cli.theme_file {
        let text =
            fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let mut palette =
            parse_custom_palette(&text).map_err(|error| format!("{}: {error}", path.display()))?;
        if !cli.neutral_only {
            palette.colors.extend(palette.accents);
        }
        return Ok(SelectedPalette {
            name: palette.name,
            colors: palette.colors,
        });
    }
    let theme = cli.theme.expect("clap requires a theme or palette");
    Ok(SelectedPalette {
        name: theme.name().to_owned(),
        colors: recolor::built_in_colors(theme, cli.neutral_only),
    })
}

struct Job {
    input: PathBuf,
    output: PathBuf,
}

fn supported(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase())
            .as_deref(),
        Some("png" | "jpg" | "jpeg" | "webp")
    )
}

fn already_recolored(path: &Path, palette_name: &str) -> bool {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .is_some_and(|stem| {
            let stem = stem.rsplit_once("-mix-").map_or(stem, |(base, _mix)| base);
            let stem = stem
                .rsplit_once("-lambda-")
                .map_or(stem, |(base, _lambda)| base);
            THEME_NAMES
                .iter()
                .copied()
                .chain(std::iter::once(palette_name))
                .any(|theme| {
                    stem.ends_with(&format!("-{theme}"))
                        || stem.ends_with(&format!("-{theme}-accent"))
                        || stem.ends_with(&format!("-{theme}-neutral"))
                })
        })
}

fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KiB", "MiB", "GiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn normalized(stem: &str) -> Result<String, String> {
    let mut output = String::new();
    let mut dash = false;
    for c in stem.bytes() {
        if c.is_ascii_alphanumeric() {
            output.push(c.to_ascii_lowercase() as char);
            dash = false;
        } else if !dash {
            output.push('-');
            dash = true;
        }
    }
    let output = output.trim_matches('-').to_owned();
    if output.is_empty() {
        Err(format!("normalized output stem is empty: {stem}"))
    } else {
        Ok(output)
    }
}

#[allow(clippy::too_many_arguments)]
fn output_path(
    input: &Path,
    normalize_name: bool,
    format: OutputFormat,
    palette_name: &str,
    neutral_only: bool,
    lambda: f32,
    mix: f32,
    palette_name_only: bool,
) -> Result<PathBuf, String> {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("invalid input name: {}", input.display()))?;
    let neutral_suffix = if neutral_only && !palette_name_only {
        "-neutral"
    } else {
        ""
    };
    let lambda_suffix = if lambda == DEFAULT_LAMBDA || palette_name_only {
        String::new()
    } else {
        format!("-lambda-{lambda}")
    };
    let mix_suffix = if mix == 1.0 || palette_name_only {
        String::new()
    } else {
        format!("-mix-{mix}")
    };
    let stem = format!("{stem}-{palette_name}{neutral_suffix}{lambda_suffix}{mix_suffix}");
    let name = if normalize_name {
        normalized(&stem)?
    } else {
        format!("{stem}")
    };
    Ok(input.with_file_name(format!("{name}.{}", format.extension())))
}

fn expand(input: &Path) -> Result<Vec<PathBuf>, String> {
    let text = input.to_string_lossy();
    if text.contains(['*', '?', '[']) {
        let paths: Result<Vec<_>, _> = glob(&text).map_err(|e| e.to_string())?.collect();
        let paths = paths.map_err(|e| e.to_string())?;
        if paths.is_empty() {
            return Err(format!("glob has no matches: {}", input.display()));
        }
        return Ok(paths);
    }
    if input.is_dir() {
        let mut paths: Vec<_> = fs::read_dir(input)
            .map_err(|e| format!("{}: {e}", input.display()))?
            .map(|entry| entry.map(|e| e.path()).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        paths.retain(|path| path.is_file() && supported(path));
        if paths.is_empty() {
            return Err(format!(
                "directory has no supported images: {}",
                input.display()
            ));
        }
        return Ok(paths);
    }
    Ok(vec![input.to_path_buf()])
}

#[allow(clippy::too_many_arguments)]
fn jobs(
    inputs: &[PathBuf],
    normalize_name: bool,
    format: OutputFormat,
    overwrite: bool,
    palette_name: &str,
    neutral_only: bool,
    lambda: f32,
    mix: f32,
    palette_name_only: bool,
) -> Result<Vec<Job>, String> {
    let mut paths = Vec::new();
    for input in inputs {
        paths.extend(expand(input)?);
    }
    paths.sort();
    paths.dedup();
    paths.retain(|path| {
        if already_recolored(path, palette_name) {
            eprintln!(
                "skipped {}: filename already has a paletteer suffix",
                path.display()
            );
            false
        } else {
            true
        }
    });
    let mut outputs = HashSet::new();
    let mut jobs = Vec::new();
    for input in paths {
        if !input.is_file() {
            return Err(format!("not a file: {}", input.display()));
        }
        if !supported(&input) {
            return Err(format!("unsupported file: {}", input.display()));
        }
        let output = output_path(
            &input,
            normalize_name,
            format,
            palette_name,
            neutral_only,
            lambda,
            mix,
            palette_name_only,
        )?;
        if !outputs.insert(output.clone()) {
            return Err(format!(
                "inputs resolve to the same output: {}",
                output.display()
            ));
        }
        if output.exists() && !overwrite {
            return Err(format!("output already exists: {}", output.display()));
        }
        jobs.push(Job { input, output });
    }
    Ok(jobs)
}

fn run(cli: Cli) -> Result<(), String> {
    if cli.quality.is_some() && !cli.format.supports_quality() {
        return Err("--quality is only valid with --format webp or jpg".to_owned());
    }
    let quality = cli.quality.unwrap_or(DEFAULT_QUALITY);
    let palette = select_palette(&cli)?;
    for job in jobs(
        &cli.input,
        cli.normalize_name,
        cli.format,
        cli.overwrite,
        &palette.name,
        cli.neutral_only,
        cli.lambda,
        cli.mix,
        cli.palette_name_only,
    )? {
        let input_size = fs::metadata(&job.input)
            .map_err(|e| format!("{}: {e}", job.input.display()))?
            .len();
        let temp = job.output.with_file_name(format!(
            ".{}.{}.tmp",
            job.output.file_name().unwrap().to_string_lossy(),
            process::id()
        ));
        let conversion = match recolor::encode_image(
            &job.input,
            &temp,
            cli.format,
            quality,
            &palette.colors,
            cli.lambda,
            cli.mix,
        ) {
            Ok(conversion) => conversion,
            Err(error) => {
                let _ = fs::remove_file(&temp);
                return Err(error);
            }
        };
        fs::rename(&temp, &job.output).map_err(|e| format!("{}: {e}", job.output.display()))?;
        let output_size = fs::metadata(&job.output)
            .map_err(|e| format!("{}: {e}", job.output.display()))?
            .len();
        let change = (output_size as f64 / input_size as f64 - 1.0) * 100.0;
        println!(
            "{} -> {} | {}x{} | {} -> {} ({change:+.0}%) | {:.2?}",
            job.input.display(),
            job.output.display(),
            conversion.width,
            conversion.height,
            human_size(input_size),
            human_size(output_size),
            conversion.duration,
        );
    }
    Ok(())
}

fn main() {
    if let Err(error) = run(Cli::parse()) {
        eprintln!("paletteer: {error}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalization_table() {
        assert_eq!(
            normalized("Misty Forest (Final)").unwrap(),
            "misty-forest-final"
        );
        assert_eq!(normalized("foo...BAR___baz").unwrap(), "foo-bar-baz");
        assert_eq!(
            normalized("é").unwrap_err(),
            "normalized output stem is empty: é"
        );
    }

    #[test]
    fn formats_file_sizes() {
        assert_eq!(human_size(999), "999 B");
        assert_eq!(human_size(1024), "1.0 KiB");
        assert_eq!(human_size(1_572_864), "1.5 MiB");
    }

    #[test]
    fn supports_input_formats() {
        for extension in ["png", "jpg", "jpeg", "webp", "WEBP"] {
            assert!(supported(Path::new(&format!("image.{extension}"))));
        }
        assert!(!supported(Path::new("image.gif")));
    }

    #[test]
    fn output_names() {
        assert_eq!(
            output_path(
                Path::new("Misty Forest (Final).jpg"),
                false,
                OutputFormat::Png,
                "everforest-dark-medium",
                false,
                DEFAULT_LAMBDA,
                1.0,
                false,
            )
            .unwrap(),
            PathBuf::from("Misty Forest (Final)-everforest-dark-medium.png")
        );
        assert_eq!(
            output_path(
                Path::new("foo.bar.png"),
                true,
                OutputFormat::Webp,
                "everforest-dark-medium",
                false,
                DEFAULT_LAMBDA,
                1.0,
                false,
            )
            .unwrap(),
            PathBuf::from("foo-bar-everforest-dark-medium.webp")
        );
        assert_eq!(
            output_path(
                Path::new("foo.jpg"),
                false,
                OutputFormat::Jpg,
                "everforest-dark-medium",
                true,
                DEFAULT_LAMBDA,
                1.0,
                false,
            )
            .unwrap(),
            PathBuf::from("foo-everforest-dark-medium-neutral.jpg")
        );
        assert_eq!(
            output_path(
                Path::new("foo.jpg"),
                false,
                OutputFormat::Jpg,
                "everforest-dark-medium",
                false,
                DEFAULT_LAMBDA,
                0.5,
                false,
            )
            .unwrap(),
            PathBuf::from("foo-everforest-dark-medium-mix-0.5.jpg")
        );
        assert_eq!(
            output_path(
                Path::new("foo.jpg"),
                true,
                OutputFormat::Jpg,
                "everforest-dark-medium",
                true,
                DEFAULT_LAMBDA,
                0.5,
                false,
            )
            .unwrap(),
            PathBuf::from("foo-everforest-dark-medium-neutral-mix-0-5.jpg")
        );
        assert_eq!(
            output_path(
                Path::new("foo.jpg"),
                false,
                OutputFormat::Jpg,
                "everforest-dark-medium",
                false,
                1.0,
                1.0,
                false,
            )
            .unwrap(),
            PathBuf::from("foo-everforest-dark-medium-lambda-1.jpg")
        );
        assert_eq!(
            output_path(
                Path::new("foo.jpg"),
                false,
                OutputFormat::Jpg,
                "everforest-dark-medium",
                true,
                1.0,
                0.5,
                true,
            )
            .unwrap(),
            PathBuf::from("foo-everforest-dark-medium.jpg")
        );
    }

    #[test]
    fn mix_validation() {
        assert_eq!(parse_mix("0").unwrap(), 0.0);
        assert_eq!(parse_mix("0.5").unwrap(), 0.5);
        assert_eq!(parse_mix("1").unwrap(), 1.0);
        for invalid in ["-0.1", "1.1", "NaN", "inf", "nope"] {
            assert!(parse_mix(invalid).is_err());
        }
    }

    #[test]
    fn lambda_validation() {
        assert_eq!(parse_lambda("0").unwrap(), 0.0);
        assert_eq!(parse_lambda("0.25").unwrap(), DEFAULT_LAMBDA);
        assert_eq!(parse_lambda("1").unwrap(), 1.0);
        for invalid in ["-0.1", "1.1", "NaN", "inf", "nope"] {
            assert!(parse_lambda(invalid).is_err());
        }
    }

    #[test]
    fn custom_palette_parsing() {
        let palette = parse_custom_palette(
            r##"
name = "forest-dusk"
colors = ["#112233", "#AABBCC"]
accents = ["#ff8800"]
"##,
        )
        .unwrap();
        assert_eq!(palette.name, "forest-dusk");
        assert_eq!(palette.colors, [0x112233, 0xAABBCC]);
        assert_eq!(palette.accents, [0xFF8800]);

        for invalid in [
            "name = 'Bad Name'\ncolors = ['#112233']",
            "name = 'empty'\ncolors = []",
            "name = 'bad-color'\ncolors = ['#12345']",
            "name = 'typo'\ncolours = ['#112233']",
        ] {
            assert!(parse_custom_palette(invalid).is_err());
        }
    }

    #[test]
    fn palette_source_is_required_and_exclusive() {
        assert!(Cli::try_parse_from(["paletteer", "image.jpg"]).is_err());
        assert!(
            Cli::try_parse_from([
                "paletteer",
                "--theme",
                "nord",
                "--theme-file",
                "custom.toml",
                "image.jpg",
            ])
            .is_err()
        );
        assert!(
            Cli::try_parse_from(["paletteer", "--theme", "nord", "image.jpg"])
                .unwrap()
                .theme_file
                .is_none()
        );
    }

    #[test]
    fn duplicate_outputs_are_rejected() {
        let directory = std::env::temp_dir().join(format!("paletteer-test-{}", process::id()));
        fs::create_dir_all(&directory).unwrap();
        let jpg = directory.join("same.jpg");
        let png = directory.join("same.png");
        fs::File::create(&jpg).unwrap();
        fs::File::create(&png).unwrap();
        assert!(
            jobs(
                &[jpg, png],
                false,
                OutputFormat::Png,
                false,
                "everforest-dark-medium",
                false,
                DEFAULT_LAMBDA,
                1.0,
                false,
            )
            .is_err()
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn overwrite_allows_an_existing_output() {
        let directory =
            std::env::temp_dir().join(format!("paletteer-overwrite-test-{}", process::id()));
        fs::create_dir_all(&directory).unwrap();
        let input = directory.join("image.jpg");
        fs::File::create(&input).unwrap();
        fs::File::create(directory.join("image-everforest-dark-medium.png")).unwrap();
        assert!(
            jobs(
                &[input.clone()],
                false,
                OutputFormat::Png,
                false,
                "everforest-dark-medium",
                false,
                DEFAULT_LAMBDA,
                1.0,
                false,
            )
            .is_err()
        );
        assert!(
            jobs(
                &[input],
                false,
                OutputFormat::Png,
                true,
                "everforest-dark-medium",
                false,
                DEFAULT_LAMBDA,
                1.0,
                false,
            )
            .is_ok()
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn prior_outputs_are_skipped() {
        let directory = std::env::temp_dir().join(format!("paletteer-skip-test-{}", process::id()));
        fs::create_dir_all(&directory).unwrap();
        let original = directory.join("mountain.jpg");
        let prior_output = directory.join("mountain-everforest-dark-medium-mix-0-5.jpg");
        fs::File::create(&original).unwrap();
        fs::File::create(&prior_output).unwrap();
        let jobs = jobs(
            &[original, prior_output],
            false,
            OutputFormat::Webp,
            false,
            "everforest-dark-medium",
            false,
            DEFAULT_LAMBDA,
            1.0,
            false,
        )
        .unwrap();
        assert_eq!(jobs.len(), 1);
        fs::remove_dir_all(directory).unwrap();
    }
}
