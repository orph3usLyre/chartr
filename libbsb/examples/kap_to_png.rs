/// Demonstrates how to create a png file from a [`KapImageFile`]
/// using the [`image`] crate
///
use image::{codecs::png::PngEncoder, ImageEncoder};
use libbsb::{ColorPalette, KapImageFile};
use std::fs::File;

fn main() -> anyhow::Result<()> {
    let bsb = KapImageFile::from_file("../test_assets/12221_1_MapTech_testing_origin.kap")?;

    let as_rgb: Vec<_> = bsb.as_palette_iter(ColorPalette::Rgb)?.flatten().collect();

    let output = File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open("kap_to_png_example.png")?;

    let encoder = PngEncoder::new(output);
    encoder.write_image(
        &as_rgb,
        bsb.width() as u32,
        bsb.height() as u32,
        image::ExtendedColorType::Rgb8,
    )?;
    Ok(())
}
