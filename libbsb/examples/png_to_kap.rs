/// Demonstrates how to create a [`KapImageFile`] from a png file
/// using the [`image`] crate
///
///
use anyhow::Context;
use image::GenericImageView;
use itertools::Itertools;
use libbsb::{
    image::raw::header::{GeneralParameters, ImageHeader},
    Depth, KapImageFile,
};
use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    let img = image::open("../test_assets/converted_png_8_depth_saint_malo.png")
        .expect("Failed to open image");
    let (width, height) = (
        img.width().try_into().expect("width is too big"),
        img.height().try_into().expect("height is too big"),
    );

    let mut raster_data = Vec::with_capacity((width * height) as usize);
    // Create a map to map RGB values to palette indexes
    let mut map: HashMap<(u8, u8, u8), usize> = HashMap::new();
    for (x, y, rgb) in img.pixels() {
        let index = map.len();
        let i = u8::try_from(*map.entry((rgb[0], rgb[1], rgb[2])).or_insert(index))
            .context("too many colors for BSB/KAP file")?;
        // Since KAP/BSB files use 1, 4, or 7 pixel depth, it cannot support
        // more than (2^7 - 1 = 127) colors
        debug_assert!(i <= 127);

        // BSB indexes start from 1
        raster_data.insert((x * y) as usize, (index + 1) as u8);
    }

    let header = ImageHeader::builder()
        .ifm(Depth::Seven)
        .general_parameters(
            GeneralParameters::builder()
                .chart_name("test chart".to_owned())
                .image_width_height((width, height))
                .build(),
        )
        .rgb(
            map.into_iter()
                .sorted_by_key(|(_, i)| *i)
                .map(|(rgb, _)| rgb)
                .collect(),
        )
        .build();
    let bsb = KapImageFile::new(header, raster_data)?;
    bsb.into_file("kap_from_png_example.kap")?;
    Ok(())
}
