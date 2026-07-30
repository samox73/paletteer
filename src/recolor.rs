use crate::OutputFormat;
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

// https://github.com/sainnhe/everforest/blob/master/palette.md
const EVERFOREST: &[(u8, u8, u8)] = &[
    (0x23, 0x2A, 0x2E),
    (0x2D, 0x35, 0x3B),
    (0x34, 0x3F, 0x44),
    (0x3D, 0x48, 0x4D),
    (0x47, 0x52, 0x58),
    (0x4F, 0x58, 0x5E),
    (0x56, 0x63, 0x5F),
    (0xD3, 0xC6, 0xAA),
    (0xE6, 0x7E, 0x80),
    (0xE6, 0x98, 0x75),
    (0xDB, 0xBC, 0x7F),
    (0xA7, 0xC0, 0x80),
    (0x83, 0xC0, 0x92),
    (0x7F, 0xBB, 0xB3),
    (0xD6, 0x99, 0xB6),
    (0x7A, 0x84, 0x78),
    (0x85, 0x92, 0x89),
    (0x9D, 0xA9, 0xA0),
];

fn palette_labs(accents: bool) -> Vec<Oklab> {
    EVERFOREST
        .iter()
        .enumerate()
        .filter(|(index, _)| accents || !(8..15).contains(index))
        .map(|(_, &(r, g, b))| {
            Oklab::from_color(Srgb::new(
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
            ))
        })
        .collect()
}

fn nearest(color: Oklab, colors: &[Oklab]) -> Oklab {
    *colors
        .iter()
        .min_by(|a, b| {
            let distance = |x: &Oklab| {
                (color.l - x.l).powi(2) + (color.a - x.a).powi(2) + (color.b - x.b).powi(2)
            };
            distance(a).total_cmp(&distance(b))
        })
        .expect("palette is not empty")
}

pub fn recolor(image: &mut RgbaImage, accents: bool) {
    let colors = palette_labs(accents);
    image.as_mut().par_chunks_exact_mut(4).for_each(|pixel| {
        if pixel[3] == 0 {
            return;
        }
        let original = Oklab::from_color(Srgb::new(
            pixel[0] as f32 / 255.0,
            pixel[1] as f32 / 255.0,
            pixel[2] as f32 / 255.0,
        ));
        let selected = nearest(original, &colors);
        let rgb: Srgb = Srgb::from_color(Oklab::new(original.l, selected.a, selected.b));
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
    accents: bool,
) -> Result<Conversion, String> {
    let started = Instant::now();
    let mut image = ImageReader::open(input)
        .map_err(|e| format!("{}: {e}", input.display()))?
        .decode()
        .map_err(|e| format!("{}: {e}", input.display()))?
        .to_rgba8();
    recolor(&mut image, accents);
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
    fn nearest_selects_expected_color() {
        let colors = [Oklab::new(0.1, 0.0, 0.0), Oklab::new(0.8, 0.2, -0.1)];
        assert_eq!(nearest(Oklab::new(0.75, 0.18, -0.08), &colors), colors[1]);
    }

    #[test]
    fn accents_are_opt_in() {
        assert_eq!(palette_labs(false).len(), 11);
        assert_eq!(palette_labs(true).len(), EVERFOREST.len());
    }

    #[test]
    fn recolor_keeps_alpha_and_gradient_lightness() {
        let mut image = RgbaImage::new(256, 1);
        for (x, _, pixel) in image.enumerate_pixels_mut() {
            *pixel = image::Rgba([x as u8, x as u8, x as u8, x as u8]);
        }
        recolor(&mut image, false);
        assert!(
            image
                .pixels()
                .map(|p| [p[0], p[1], p[2]])
                .collect::<std::collections::HashSet<_>>()
                .len()
                > EVERFOREST.len()
        );
        for (x, _, pixel) in image.enumerate_pixels() {
            assert_eq!(pixel[3], x as u8);
        }
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
        for (format, extension) in [(OutputFormat::Webp, "webp"), (OutputFormat::Jpg, "jpg")] {
            let output = directory.join(format!("output.{extension}"));
            encode_image(&input, &output, format, 80, false).unwrap();
            assert_eq!(image::open(output).unwrap().dimensions(), (1, 1));
        }
        std::fs::remove_dir_all(directory).unwrap();
    }
}
