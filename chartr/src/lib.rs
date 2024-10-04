use std::{collections::HashSet, fs::File, path::Path};

use anyhow::Result;
use image::{codecs::png::PngEncoder, GenericImageView, ImageEncoder};
use libbsb::{
    image::{
        raw::header::{GeneralParameters, ImageHeader},
        BitMap,
    },
    ColorPalette, KapImageFile,
};
use tracing::{debug, info, instrument};

#[instrument]
pub fn kap_to_image(bsb_file: &Path, output_name: &Path) -> Result<()> {
    let bsb = KapImageFile::from_file(bsb_file)?;
    debug!("Read bsb from file");

    let as_rgb: Vec<_> = bsb.as_palette_iter(ColorPalette::Rgb)?.flatten().collect();
    debug!("Length of RGB bsb data: {}", as_rgb.len());

    let output = File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(output_name)?;

    info!("Writing applied palatte image to {}", output_name.display());
    let encoder = PngEncoder::new(output);
    encoder.write_image(
        &as_rgb,
        bsb.width() as u32,
        bsb.height() as u32,
        image::ExtendedColorType::Rgb8,
    )?;
    info!(
        "Successfully wrote palatte image to {}",
        output_name.display()
    );
    Ok(())
}

#[instrument]
pub fn image_to_kap(image_file: &Path, output_name: &Path) -> Result<()> {
    let img = image::open(image_file).expect("Failed to open image");
    let unique_colors = img
        .pixels()
        .map(|(_, _, rgba)| (rgba[0], rgba[1], rgba[2]))
        .collect::<HashSet<_>>();
    debug!("Read {} unique colors from image", unique_colors.len());

    debug_assert!(unique_colors.len() <= 127);

    // The 3 depth types a BSB file can hold
    let depth = match unique_colors.len() {
        1 => 1,
        n if n <= 15 => 4,
        n if n <= 127 => 7,
        _ => unreachable!(),
    }
    .try_into()
    .unwrap();

    let (width, height) = (
        img.width().try_into().expect("width is too big"),
        img.height().try_into().expect("height is too big"),
    );

    let header = ImageHeader::builder()
        .ifm(depth)
        .general_parameters(
            GeneralParameters::builder()
                .chart_name("test chart".to_owned())
                .image_width_height((width, height))
                .build(),
        )
        .rgb(unique_colors.into_iter().collect())
        .build();

    let rgbs = header.rgb.as_ref().unwrap();
    let mut bitmap = BitMap::empty(width, height);
    for (x, y, p) in img.pixels() {
        if let Some(index) = rgbs.iter().position(|rgb| rgb.eq(&(p[0], p[1], p[2]))) {
            // BSB indexes start from 1
            bitmap.set_pixel_index(x as u16, y as u16, (index + 1) as u8)
        } else {
            eprintln!("Unable to find pos for pixel");
        }
    }
    let bsb = KapImageFile::new(header, bitmap)?;
    bsb.into_file("test_assets/bsb_from_png.kap")?;
    Ok(())
}
