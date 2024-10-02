use std::{
    collections::HashSet,
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

mod common;
use common::{CONVERTED_PNG_MAPTECH_TEST_KAP_4_DEPTH, TEST_KAP_TO_PNG};
use image::{codecs::png::PngEncoder, GenericImageView, ImageEncoder};
use libbsb::{
    image::raw::header::{GeneralParameters, ImageHeader},
    BitMap, ColorPalette, Depth, KapImageFile,
};
use mktemp::Temp;

#[test]
fn create_png_from_reserialized_bsb() -> anyhow::Result<()> {
    let bsb = KapImageFile::from_file(&PathBuf::from_str(TEST_KAP_TO_PNG)?)?;
    println!("Read BSB from {TEST_KAP_TO_PNG}");
    let tmp_kap = Temp::new_file()?;
    bsb.into_file(&tmp_kap)?;
    let bsb_2 = KapImageFile::from_file(tmp_kap)?;

    let as_rgb = bsb_2.as_palette_iter(ColorPalette::Rgb)?;

    let output = File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(Temp::new_file()?)?;

    let encoder = PngEncoder::new(output);
    encoder.write_image(
        as_rgb.flatten().collect::<Vec<_>>().as_slice(),
        u32::from(bsb_2.width()),
        u32::from(bsb_2.height()),
        image::ExtendedColorType::Rgb8,
    )?;
    Ok(())
}

#[test]
fn create_bsb_from_converted_png() -> anyhow::Result<()> {
    let img = image::open(CONVERTED_PNG_MAPTECH_TEST_KAP_4_DEPTH).expect("Failed to open image");
    let unique_colors = img
        .pixels()
        .map(|(_, _, rgba)| (rgba[0], rgba[1], rgba[2]))
        .collect::<HashSet<_>>();
    let (width, height) = (
        img.width().try_into().expect("width is too big"),
        img.height().try_into().expect("height is too big"),
    );
    debug_assert!(unique_colors.len() <= 127);

    let header = ImageHeader::builder()
        .ifm(Depth::Four)
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
            bitmap.set_pixel(x as u16, y as u16, (index + 1) as u8)
        } else {
            eprintln!("Unable to find pos for pixel");
        }
    }
    let bsb = KapImageFile::new(header, bitmap)?;
    bsb.into_file(Temp::new_file()?)?;
    Ok(())
}

#[test]
fn recreate_png_from_converted_png() -> anyhow::Result<()> {
    let img = image::open(CONVERTED_PNG_MAPTECH_TEST_KAP_4_DEPTH).expect("Failed to open image");
    let unique_colors = img
        .pixels()
        .map(|(_, _, rgba)| (rgba[0], rgba[1], rgba[2]))
        .collect::<HashSet<_>>();
    let (width, height) = (
        img.width().try_into().expect("width is too big"),
        img.height().try_into().expect("height is too big"),
    );
    debug_assert!(unique_colors.len() <= 127);

    let header = ImageHeader::builder()
        .ifm(Depth::Four)
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
            bitmap.set_pixel(x as u16, y as u16, (index + 1) as u8)
        } else {
            eprintln!("Unable to find pos for pixel");
        }
    }
    let tmp_kap = Temp::new_file()?;
    let bsb = KapImageFile::new(header, bitmap)?;
    bsb.into_file(&tmp_kap)?;

    // now load it again
    let bsb = KapImageFile::from_file(&tmp_kap)?;

    let as_rgb = bsb.as_palette_iter(ColorPalette::Rgb)?;
    let tmp_png = Temp::new_file()?;
    let output = File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&tmp_png)?;

    let encoder = PngEncoder::new(output);
    encoder.write_image(
        as_rgb.flatten().collect::<Vec<_>>().as_slice(),
        u32::from(bsb.width()),
        u32::from(bsb.height()),
        image::ExtendedColorType::Rgb8,
    )?;

    // assert hashes
    let hash_1 = sha256::try_digest(Path::new(CONVERTED_PNG_MAPTECH_TEST_KAP_4_DEPTH)).unwrap();
    let hash_2 = sha256::try_digest(tmp_png).unwrap();
    assert_eq!(hash_1, hash_2);
    Ok(())
}
