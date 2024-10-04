/// Demonstrates how to create a [`KapImageFile`] from a png file
/// using the [`image`] crate
///
///
use anyhow::Context;
use image::GenericImageView;
use itertools::Itertools;
use libbsb::{
    image::{
        raw::header::{GeneralParameters, ImageHeader},
        BitMap,
    },
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

    let mut bitmap = BitMap::empty(width, height);
    let mut map: HashMap<(u8, u8, u8), usize> = HashMap::new();
    for (x, y, p) in img.pixels() {
        let index = map.len();
        let i = u8::try_from(*map.entry((p[0], p[1], p[2])).or_insert(index))
            .context("too many colors for BSB/KAP file")?;
        // Since KAP/BSB files use 1, 4, or 7 pixel depth, it cannot support
        // more than (2^7 - 1 = 127) colors
        debug_assert!(i <= 127);

        // BSB indexes start from 1
        bitmap.set_pixel_index(x as u16, y as u16, i + 1)
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
    let bsb = KapImageFile::new(header, bitmap)?;
    bsb.into_file("kap_from_png_example.kap")?;
    Ok(())
}
