mod recolor;

use clap::{Parser, ValueEnum};
use glob::glob;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    process,
};

const THEME: &str = "everforest-dark-medium";
const DEFAULT_QUALITY: u8 = 80;

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
    about = "Recolor images with an Everforest palette"
)]
struct Cli {
    #[arg(short, long, value_parser = [THEME])]
    theme: String,
    #[arg(short = 'n', long)]
    normalize_name: bool,
    #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Png)]
    format: OutputFormat,
    #[arg(short = 'q', long, value_parser = clap::value_parser!(u8).range(1..=100))]
    quality: Option<u8>,
    #[arg(required = true)]
    input: Vec<PathBuf>,
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
        Some("png" | "jpg" | "jpeg")
    )
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
) -> Result<PathBuf, String> {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("invalid input name: {}", input.display()))?;
    let stem = format!("{stem}-{THEME}");
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
) -> Result<Vec<Job>, String> {
    let mut paths = Vec::new();
    for input in inputs {
        paths.extend(expand(input)?);
    }
    paths.sort();
    paths.dedup();
    let mut outputs = HashSet::new();
    let mut jobs = Vec::new();
    for input in paths {
        if !input.is_file() {
            return Err(format!("not a file: {}", input.display()));
        }
        if !supported(&input) {
            return Err(format!("unsupported file: {}", input.display()));
        }
        let output = output_path(&input, normalize_name, format)?;
        if !outputs.insert(output.clone()) {
            return Err(format!(
                "inputs resolve to the same output: {}",
                output.display()
            ));
        }
        if output.exists() {
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
    for job in jobs(&cli.input, cli.normalize_name, cli.format)? {
        let temp = job.output.with_file_name(format!(
            ".{}.{}.tmp",
            job.output.file_name().unwrap().to_string_lossy(),
            process::id()
        ));
        if let Err(error) = recolor::encode_image(&job.input, &temp, cli.format, quality) {
            let _ = fs::remove_file(&temp);
            return Err(error);
        }
        fs::rename(&temp, &job.output).map_err(|e| format!("{}: {e}", job.output.display()))?;
        println!("{}", job.output.display());
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
    fn output_names() {
        assert_eq!(
            output_path(
                Path::new("Misty Forest (Final).jpg"),
                false,
                OutputFormat::Png
            )
            .unwrap(),
            PathBuf::from("Misty Forest (Final)-everforest-dark-medium.png")
        );
        assert_eq!(
            output_path(Path::new("foo.bar.png"), true, OutputFormat::Webp).unwrap(),
            PathBuf::from("foo-bar-everforest-dark-medium.webp")
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
        assert!(jobs(&[jpg, png], false, OutputFormat::Png).is_err());
        fs::remove_dir_all(directory).unwrap();
    }
}
