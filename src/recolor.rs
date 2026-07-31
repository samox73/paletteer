use crate::{OutputFormat, Theme};
use image::{ExtendedColorType, ImageEncoder, ImageReader, RgbaImage, codecs::jpeg::JpegEncoder};
use palette::{FromColor, Oklab, Srgb};
use rayon::prelude::*;
use std::{
    fs,
    io::{self, Write},
    path::Path,
    time::Instant,
};

pub struct Conversion {
    pub duration: std::time::Duration,
    pub width: u32,
    pub height: u32,
}

struct ThemePalette {
    neutral: &'static [u32],
    accents: &'static [u32],
}

// https://github.com/sainnhe/everforest/blob/master/palette.md
const EVERFOREST: ThemePalette = ThemePalette {
    neutral: &[
        0x232A2E, 0x2D353B, 0x343F44, 0x3D484D, 0x475258, 0x4F585E, 0x56635F, 0xD3C6AA, 0x7A8478,
        0x859289, 0x9DA9A0,
    ],
    accents: &[
        0xE67E80, 0xE69875, 0xDBBC7F, 0xA7C080, 0x83C092, 0x7FBBB3, 0xD699B6,
    ],
};
// https://github.com/catppuccin/palette/blob/main/palette.json
const CATPPUCCIN: ThemePalette = ThemePalette {
    neutral: &[
        0x1E1E2E, 0x181825, 0x11111B, 0x313244, 0x45475A, 0x585B70, 0x6C7086, 0x7F849C, 0x9399B2,
        0xCDD6F4, 0xBAC2DE, 0xA6ADC8,
    ],
    accents: &[
        0xF5E0DC, 0xF2CDCD, 0xF5C2E7, 0xCBA6F7, 0xF38BA8, 0xEBA0AC, 0xFAB387, 0xF9E2AF, 0xA6E3A1,
        0x94E2D5, 0x89DCEB, 0x74C7EC, 0x89B4FA, 0xB4BEFE,
    ],
};
// https://github.com/folke/tokyonight.nvim/blob/main/lua/tokyonight/colors/night.lua
const TOKYO_NIGHT: ThemePalette = ThemePalette {
    neutral: &[
        0x16161E, 0x1A1B26, 0x292E42, 0x3B4261, 0x414868, 0x545C7E, 0x565F89, 0x737AA2, 0xA9B1D6,
        0xC0CAF5,
    ],
    accents: &[
        0xF7768E, 0x9ECE6A, 0xE0AF68, 0x7AA2F7, 0xBB9AF7, 0x7DCFFF, 0xFF9E64, 0xDB4B4B, 0x73DACA,
    ],
};
// https://github.com/morhetz/gruvbox/blob/master/colors/gruvbox.vim
const GRUVBOX: ThemePalette = ThemePalette {
    neutral: &[
        0x1D2021, 0x282828, 0x32302F, 0x3C3836, 0x504945, 0x665C54, 0x7C6F64, 0x928374, 0xA89984,
        0xBDAE93, 0xD5C4A1, 0xEBDBB2, 0xFBF1C7,
    ],
    accents: &[
        0xFB4934, 0xB8BB26, 0xFABD2F, 0x83A598, 0xD3869B, 0x8EC07C, 0xFE8019,
    ],
};
// https://www.nordtheme.com/docs/colors-and-palettes
const NORD: ThemePalette = ThemePalette {
    neutral: &[
        0x2E3440, 0x3B4252, 0x434C5E, 0x4C566A, 0xD8DEE9, 0xE5E9F0, 0xECEFF4,
    ],
    accents: &[
        0x8FBCBB, 0x88C0D0, 0x81A1C1, 0x5E81AC, 0xBF616A, 0xD08770, 0xEBCB8B, 0xA3BE8C, 0xB48EAD,
    ],
};
// https://draculatheme.com/contribute
const DRACULA: ThemePalette = ThemePalette {
    neutral: &[0x282A36, 0x44475A, 0x6272A4, 0xF8F8F2],
    accents: &[
        0x8BE9FD, 0x50FA7B, 0xFFB86C, 0xFF79C6, 0xBD93F9, 0xFF5555, 0xF1FA8C,
    ],
};
// https://github.com/rose-pine/rose-pine-palette
const ROSE_PINE: ThemePalette = ThemePalette {
    neutral: &[
        0x232136, 0x2A273F, 0x393552, 0x6E6A86, 0x908CAA, 0xE0DEF4, 0x2A283E, 0x44415A, 0x56526E,
    ],
    accents: &[0xEB6F92, 0xF6C177, 0xEA9A97, 0x3E8FB0, 0x9CCFD8, 0xC4A7E7],
};

fn palette(theme: Theme) -> &'static ThemePalette {
    match theme {
        Theme::EverforestDarkMedium => &EVERFOREST,
        Theme::CatppuccinMocha => &CATPPUCCIN,
        Theme::TokyoNight => &TOKYO_NIGHT,
        Theme::GruvboxDarkMedium => &GRUVBOX,
        Theme::Nord => &NORD,
        Theme::Dracula => &DRACULA,
        Theme::RosePineMoon => &ROSE_PINE,
    }
}

pub fn built_in_colors(theme: Theme, neutral_only: bool) -> Vec<u32> {
    let palette = palette(theme);
    palette
        .neutral
        .iter()
        .chain(if neutral_only { &[] } else { palette.accents })
        .copied()
        .collect()
}

fn palette_labs(colors: &[u32]) -> Vec<Oklab> {
    colors
        .iter()
        .map(|&color| {
            Oklab::from_color(Srgb::new(
                ((color >> 16) & 0xFF) as f32 / 255.0,
                ((color >> 8) & 0xFF) as f32 / 255.0,
                (color & 0xFF) as f32 / 255.0,
            ))
        })
        .collect()
}

fn distance(a: Oklab, b: Oklab, lambda: f32) -> f32 {
    lambda * (a.l - b.l).powi(2) + (a.a - b.a).powi(2) + (a.b - b.b).powi(2)
}

fn nearest_two(color: Oklab, colors: &[Oklab], lambda: f32) -> (Oklab, Oklab) {
    let mut nearest = [(f32::INFINITY, colors[0]); 2];
    for &candidate in colors {
        let candidate = (distance(color, candidate, lambda), candidate);
        if candidate.0 < nearest[0].0 {
            nearest[1] = nearest[0];
            nearest[0] = candidate;
        } else if candidate.0 < nearest[1].0 {
            nearest[1] = candidate;
        }
    }
    (nearest[0].1, nearest[1].1)
}

// Interpolate between the two nearest palette colors by the pixel's projection
// onto the segment between them. Continuous across Voronoi boundaries, so smooth
// chroma gradients stay smooth instead of snapping to a hard contour.
fn blended(color: Oklab, colors: &[Oklab], lambda: f32) -> Oklab {
    let (first, second) = nearest_two(color, colors, lambda);
    let denominator = distance(first, second, lambda);
    if denominator == 0.0 {
        return first;
    }
    let amount = ((lambda * (color.l - first.l) * (second.l - first.l)
        + (color.a - first.a) * (second.a - first.a)
        + (color.b - first.b) * (second.b - first.b))
        / denominator)
        .clamp(0.0, 1.0);
    Oklab::new(
        first.l + amount * (second.l - first.l),
        first.a + amount * (second.a - first.a),
        first.b + amount * (second.b - first.b),
    )
}

pub fn recolor(image: &mut RgbaImage, colors: &[u32], lambda: f32, mix: f32) {
    if mix == 0.0 {
        return;
    }
    let colors = palette_labs(colors);
    // ponytail: exact extrema match the requested range; use percentiles if outliers flatten contrast.
    let (source_min, source_max) = image
        .as_raw()
        .par_chunks_exact(4)
        .filter(|pixel| pixel[3] != 0)
        .map(|pixel| {
            Oklab::from_color(Srgb::new(
                pixel[0] as f32 / 255.0,
                pixel[1] as f32 / 255.0,
                pixel[2] as f32 / 255.0,
            ))
            .l
        })
        .map(|lightness| (lightness, lightness))
        .reduce(
            || (f32::INFINITY, f32::NEG_INFINITY),
            |a, b| (a.0.min(b.0), a.1.max(b.1)),
        );
    if !source_min.is_finite() {
        return;
    }
    let (theme_min, theme_max) = colors
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |range, color| {
            (range.0.min(color.l), range.1.max(color.l))
        });
    let (scale, offset) = if source_max > source_min {
        let scale = (theme_max - theme_min) / (source_max - source_min);
        (scale, theme_min - source_min * scale)
    } else {
        (0.0, (theme_min + theme_max) / 2.0)
    };
    image.as_mut().par_chunks_exact_mut(4).for_each(|pixel| {
        if pixel[3] == 0 {
            return;
        }
        let original = Oklab::from_color(Srgb::new(
            pixel[0] as f32 / 255.0,
            pixel[1] as f32 / 255.0,
            pixel[2] as f32 / 255.0,
        ));
        let theme_lightness = original.l * scale + offset;
        let selected = blended(
            Oklab::new(theme_lightness, original.a, original.b),
            &colors,
            lambda,
        );
        let remapped = Oklab::new(
            original.l + mix * (theme_lightness - original.l),
            original.a + mix * (selected.a - original.a),
            original.b + mix * (selected.b - original.b),
        );
        let rgb: Srgb = Srgb::from_color(remapped);
        let (r, g, b) = rgb.into_components();
        pixel[0] = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
        pixel[1] = (g.clamp(0.0, 1.0) * 255.0).round() as u8;
        pixel[2] = (b.clamp(0.0, 1.0) * 255.0).round() as u8;
    });
}

pub fn encode_image(
    input: &Path,
    temporary: &Path,
    format: OutputFormat,
    quality: u8,
    colors: &[u32],
    lambda: f32,
    mix: f32,
) -> Result<Conversion, String> {
    let started = Instant::now();
    let mut image = ImageReader::open(input)
        .map_err(|e| format!("{}: {e}", input.display()))?
        .decode()
        .map_err(|e| format!("{}: {e}", input.display()))?
        .to_rgba8();
    recolor(&mut image, colors, lambda, mix);
    let (width, height) = image.dimensions();
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temporary)
        .map_err(|e| format!("{}: {e}", temporary.display()))?;
    let mut writer = io::BufWriter::new(file);
    match format {
        OutputFormat::Png => image
            .write_to(&mut writer, image::ImageFormat::Png)
            .map_err(|e| format!("{}: {e}", temporary.display()))?,
        OutputFormat::Webp => writer
            .write_all(
                &webp::Encoder::from_rgba(image.as_raw(), image.width(), image.height())
                    .encode(quality as f32),
            )
            .map_err(|e| format!("{}: {e}", temporary.display()))?,
        OutputFormat::Jpg => {
            let rgb = image::DynamicImage::ImageRgba8(image).to_rgb8();
            JpegEncoder::new_with_quality(&mut writer, quality)
                .write_image(
                    rgb.as_raw(),
                    rgb.width(),
                    rgb.height(),
                    ExtendedColorType::Rgb8,
                )
                .map_err(|e| format!("{}: {e}", temporary.display()))?;
        }
    }
    writer
        .flush()
        .map_err(|e| format!("{}: {e}", temporary.display()))?;
    Ok(Conversion {
        duration: started.elapsed(),
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::GenericImageView;

    #[test]
    fn lambda_controls_lightness_weight() {
        let colors = [Oklab::new(0.1, 0.18, -0.08), Oklab::new(0.75, 0.0, 0.0)];
        let source = Oklab::new(0.75, 0.18, -0.08);
        // At lambda 0 lightness is ignored, so the exact-chroma match wins outright.
        assert_eq!(nearest_two(source, &colors, 0.0).0, colors[0]);
        // At lambda 1 lightness dominates, so the matching-lightness color wins.
        assert_eq!(nearest_two(source, &colors, 1.0).0, colors[1]);
    }

    #[test]
    fn blended_interpolates_between_neighbors() {
        let colors = [Oklab::new(0.0, 0.0, 0.0), Oklab::new(0.0, 1.0, 0.0)];
        let midpoint = blended(Oklab::new(0.0, 0.5, 0.0), &colors, 0.25);
        assert!((midpoint.a - 0.5).abs() < 1e-6);
        // Endpoints stay put instead of averaging toward the other color.
        assert_eq!(blended(Oklab::new(0.0, 0.0, 0.0), &colors, 0.25), colors[0]);
    }

    #[test]
    fn all_colors_are_used_by_default() {
        assert_eq!(
            built_in_colors(Theme::EverforestDarkMedium, false).len(),
            EVERFOREST.neutral.len() + EVERFOREST.accents.len()
        );
        assert_eq!(
            built_in_colors(Theme::EverforestDarkMedium, true).len(),
            EVERFOREST.neutral.len()
        );
    }

    #[test]
    fn every_theme_has_neutral_colors() {
        for theme in [
            Theme::EverforestDarkMedium,
            Theme::CatppuccinMocha,
            Theme::TokyoNight,
            Theme::GruvboxDarkMedium,
            Theme::Nord,
            Theme::Dracula,
            Theme::RosePineMoon,
        ] {
            assert!(!built_in_colors(theme, false).is_empty());
        }
    }

    #[test]
    fn recolor_maps_continuous_theme_lightness_and_keeps_alpha() {
        let mut image = RgbaImage::new(256, 1);
        for (x, _, pixel) in image.enumerate_pixels_mut() {
            *pixel = image::Rgba([x as u8, x as u8, x as u8, (x % 254 + 1) as u8]);
        }
        let colors = [0x202020, 0xC0C0C0];
        recolor(&mut image, &colors, 0.25, 1.0);
        assert!(
            image
                .pixels()
                .map(|p| [p[0], p[1], p[2]])
                .collect::<std::collections::HashSet<_>>()
                .len()
                > colors.len()
        );
        assert_eq!(&image.get_pixel(0, 0).0[..3], &[0x20, 0x20, 0x20]);
        assert_eq!(&image.get_pixel(255, 0).0[..3], &[0xC0, 0xC0, 0xC0]);
        for (x, _, pixel) in image.enumerate_pixels() {
            assert_eq!(pixel[3], (x % 254 + 1) as u8);
        }
    }

    #[test]
    fn mix_endpoints_and_midpoint() {
        let original = RgbaImage::from_pixel(1, 1, image::Rgba([220, 40, 30, 128]));
        let colors = built_in_colors(Theme::EverforestDarkMedium, false);

        let mut unchanged = original.clone();
        recolor(&mut unchanged, &colors, 0.25, 0.0);
        assert_eq!(unchanged, original);

        let mut midpoint = original.clone();
        recolor(&mut midpoint, &colors, 0.25, 0.5);

        let mut remapped = original.clone();
        recolor(&mut remapped, &colors, 0.25, 1.0);

        assert_ne!(midpoint, original);
        assert_ne!(midpoint, remapped);
        assert_eq!(midpoint.get_pixel(0, 0)[3], 128);
        assert_eq!(remapped.get_pixel(0, 0)[3], 128);
    }

    #[test]
    fn encoded_formats_decode() {
        let directory =
            std::env::temp_dir().join(format!("paletteer-recolor-test-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let input = directory.join("input.png");
        RgbaImage::from_pixel(1, 1, image::Rgba([10, 20, 30, 128]))
            .save(&input)
            .unwrap();
        let colors = built_in_colors(Theme::EverforestDarkMedium, false);
        for (format, extension) in [(OutputFormat::Webp, "webp"), (OutputFormat::Jpg, "jpg")] {
            let output = directory.join(format!("output.{extension}"));
            encode_image(&input, &output, format, 80, &colors, 0.25, 1.0).unwrap();
            assert_eq!(image::open(output).unwrap().dimensions(), (1, 1));
        }
        std::fs::remove_dir_all(directory).unwrap();
    }
}
