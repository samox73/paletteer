// Synthetic chart for eyeballing how the recolor pipeline treats smooth
// gradients. Each band is a known-smooth input, so any step or contour in the
// recolored output is the algorithm's, not a photo's. Run:
//
//     cargo run --example benchmark
//     paletteer --theme everforest-dark-medium benchmark.png
//
// then compare benchmark.png with the recolored result.
use image::RgbImage;
use palette::{FromColor, Oklch, Srgb};

const WIDTH: u32 = 720;
const BAND: u32 = 90;

fn main() {
    // Each band maps x in 0..1 to an Oklch color, stacked top to bottom.
    let bands: Vec<Box<dyn Fn(f32) -> Oklch>> = vec![
        Box::new(|t| Oklch::new(t, 0.0, 0.0)), // lightness ramp, neutral -> L remap + banding
        Box::new(|t| Oklch::new(0.70, 0.08, t * 360.0)), // hue sweep, light -> chroma contours
        Box::new(|t| Oklch::new(0.45, 0.08, t * 360.0)), // hue sweep, mid
        Box::new(|t| Oklch::new(t, 0.10, 150.0)), // lightness ramp at a fixed hue
        Box::new(|t| Oklch::new(0.60, t * 0.20, 30.0)), // chroma ramp -> where accents engage
    ];
    let height = BAND * bands.len() as u32;
    let mut image = RgbImage::new(WIDTH, height);
    for (band, make) in bands.iter().enumerate() {
        let top = band as u32 * BAND;
        for x in 0..WIDTH {
            let srgb = Srgb::from_color(make(x as f32 / (WIDTH - 1) as f32));
            let px = image::Rgb([
                (srgb.red.clamp(0.0, 1.0) * 255.0).round() as u8,
                (srgb.green.clamp(0.0, 1.0) * 255.0).round() as u8,
                (srgb.blue.clamp(0.0, 1.0) * 255.0).round() as u8,
            ]);
            for y in top..top + BAND {
                image.put_pixel(x, y, px);
            }
        }
    }
    image.save("benchmark.png").expect("write benchmark.png");
    println!("wrote benchmark.png ({WIDTH}x{height})");
}
