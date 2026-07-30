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
#[command(name = "paletteer", about = "Recolor images with built-in palettes")]
struct Cli {
    #[arg(short, long, value_enum)]
    theme: Theme,
    #[arg(short = 'n', long)]
    normalize_name: bool,
    #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Png)]
    format: OutputFormat,
    #[arg(short = 'q', long, value_parser = clap::value_parser!(u8).range(1..=100))]
    quality: Option<u8>,
    #[arg(short = 'a', long, help = "Include palette accent colors")]
    accents: bool,
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

fn already_recolored(path: &Path) -> bool {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .is_some_and(|stem| {
            let stem = stem.rsplit_once("-mix-").map_or(stem, |(base, _mix)| base);
            THEME_NAMES.iter().any(|theme| {
                stem.ends_with(&format!("-{theme}")) || stem.ends_with(&format!("-{theme}-accent"))
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

fn output_path(
    input: &Path,
    normalize_name: bool,
    format: OutputFormat,
    theme: Theme,
    accents: bool,
    mix: f32,
) -> Result<PathBuf, String> {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("invalid input name: {}", input.display()))?;
    let accent_suffix = if accents { "-accent" } else { "" };
    let mix_suffix = if mix == 1.0 {
        String::new()
    } else {
        format!("-mix-{mix}")
    };
    let stem = format!("{stem}-{}{accent_suffix}{mix_suffix}", theme.name());
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

fn jobs(
    inputs: &[PathBuf],
    normalize_name: bool,
    format: OutputFormat,
    overwrite: bool,
    theme: Theme,
    accents: bool,
    mix: f32,
) -> Result<Vec<Job>, String> {
    let mut paths = Vec::new();
    for input in inputs {
        paths.extend(expand(input)?);
    }
    paths.sort();
    paths.dedup();
    paths.retain(|path| {
        if already_recolored(path) {
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
        let output = output_path(&input, normalize_name, format, theme, accents, mix)?;
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
    for job in jobs(
        &cli.input,
        cli.normalize_name,
        cli.format,
        cli.overwrite,
        cli.theme,
        cli.accents,
        cli.mix,
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
            cli.theme,
            cli.accents,
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
                Theme::EverforestDarkMedium,
                false,
                1.0,
            )
            .unwrap(),
            PathBuf::from("Misty Forest (Final)-everforest-dark-medium.png")
        );
        assert_eq!(
            output_path(
                Path::new("foo.bar.png"),
                true,
                OutputFormat::Webp,
                Theme::EverforestDarkMedium,
                false,
                1.0,
            )
            .unwrap(),
            PathBuf::from("foo-bar-everforest-dark-medium.webp")
        );
        assert_eq!(
            output_path(
                Path::new("foo.jpg"),
                false,
                OutputFormat::Jpg,
                Theme::EverforestDarkMedium,
                true,
                1.0,
            )
            .unwrap(),
            PathBuf::from("foo-everforest-dark-medium-accent.jpg")
        );
        assert_eq!(
            output_path(
                Path::new("foo.jpg"),
                false,
                OutputFormat::Jpg,
                Theme::EverforestDarkMedium,
                false,
                0.5,
            )
            .unwrap(),
            PathBuf::from("foo-everforest-dark-medium-mix-0.5.jpg")
        );
        assert_eq!(
            output_path(
                Path::new("foo.jpg"),
                true,
                OutputFormat::Jpg,
                Theme::EverforestDarkMedium,
                true,
                0.5,
            )
            .unwrap(),
            PathBuf::from("foo-everforest-dark-medium-accent-mix-0-5.jpg")
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
                Theme::EverforestDarkMedium,
                false,
                1.0,
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
                Theme::EverforestDarkMedium,
                false,
                1.0,
            )
            .is_err()
        );
        assert!(
            jobs(
                &[input],
                false,
                OutputFormat::Png,
                true,
                Theme::EverforestDarkMedium,
                false,
                1.0,
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
            Theme::EverforestDarkMedium,
            false,
            1.0,
        )
        .unwrap();
        assert_eq!(jobs.len(), 1);
        fs::remove_dir_all(directory).unwrap();
    }
}
