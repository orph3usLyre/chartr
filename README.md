# chartr
A naval chart manipulation library written in rust.

<!-- cargo-rdme start -->

# libbsb


This library provides datatypes and i/o functionality for the `MapTech` BSB/KAP file format, an
older format used for naval raster navigational charts (RNC).

It aims to provide a minimal, low-level API to build upon. Since it remains unclear what guarantees the BSB file
should provide (the header section, in particular), the responsibility of creating "valid" files is placed on the users of this crate.
I've tried to emphasise this where possible, by placing these types in `raw` modules.

### Limitations

The [`Maptech BSB File Format Test Dataset Instructions`](https://legacy.iho.int/mtg_docs/com_wg/HSSC/HSSC3/S-64_Edition_2.0.0/RNC_Test_Data_Sets/BSB_TDS/BSB_Test_Dataset_Instructions_for_RNC.pdf) specifies that the BSB format contains three types of files:
- A documentation file
- An image file
- An update patch file

This library currently **only** supports the BSB image file.

While the BSB/KAP image file supposedly supports a image depth of `1`, I haven't found any examples
  to test this functionality yet.


Comments inside BSB/KAP are currently ignored, since it remains unclear how they should be
  handled.

Please open an issue on GitHub if you would like to see support for unimplemented features. PRs
are welcome!

### Usage

The primary use case for this library is to allow conversions from and to the BSB/KAP image file format.

#### Converting a BSB/KAP image file to an image

```rust
use image::{codecs::png::PngEncoder, ImageEncoder};
use libbsb::{KapImageFile,  ColorPalette};

fn main() -> anyhow::Result<()> {
    // read a BSB/KAP file
    let bsb =
    KapImageFile::from_path("../test_assets/12221_1_MapTech_testing_origin.kap")?;

    // convert the pixel indices into RGB colors
    // (this test file has several palettes to play around with)
    let as_rgb: Vec<_> = bsb.as_palette_iter(ColorPalette::Rgb)?.flatten().collect();

    let output = std::fs::File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open("kap_to_png_example.png")?;

    // here we use the `image` crate to encode the RGBs as a PNG file
    let encoder = PngEncoder::new(output);
    encoder.write_image(
        &as_rgb,
        bsb.width() as u32,
        bsb.height() as u32,
        image::ExtendedColorType::Rgb8,
    )?;
    Ok(())
}
```

#### Converting an image file to a BSB/KAP file

Converting an image file into a BSB/KAP file is slightly more involved, since BSB files store
the color palette as part of the header, and can hold multiple palettes. The first step is to
reduce the number of colors in a given image to 127 or below (7-bit pixel depth).

In the following example, the image used is a chart with 15 colors, which matches the 4-bit
pixel depth BSB file. For images with more than 127 colors,
consider a color quantization technique such as median cut, k-means clustering, or octree quantization.

```rust
use image::GenericImageView;
use libbsb::{KapImageFile, image::raw::header::{ImageHeader, GeneralParameters}, Depth};
use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    let img = image::open("../test_assets/converted_png_8_depth_saint_malo.png")
        .expect("Failed to open image");

    // BSB/KAP files use `u16`s to define their heigh/width
    let (width, height) = (
        img.width().try_into().expect("width is too big"),
        img.height().try_into().expect("height is too big"),
    );

    
  // create a buffer for the raster data
  let mut raster_data = vec![0; width as usize * height as usize];
  // create a hashmap that represents our palette, from RGB -> index
  let mut map = HashMap::new();
    for (x, y, p) in img.pixels() {
        let index = map.len();
        let i = u8::try_from(*map.entry((p[0], p[1], p[2])).or_insert(index))
            .expect("too many colors for BSB/KAP file");
        // Since BSB/KAP files use 1, 4, or 7 pixel depth, it cannot support
        // more than (2^7 - 1 = 127) colors
        debug_assert!(i <= 127);

        // BSB indexes start from 1
         raster_data[y as usize * width as usize + x as usize] = (index + 1) as u8;
    }
    // we sort our palette so that pixel indices start from 1
    let mut palette = map.into_iter().collect::<Vec<_>>();
    palette.sort_by_key(|(_, i)| *i);

    let header = ImageHeader::builder()
        .ifm(Depth::Seven)
        .general_parameters(
            GeneralParameters::builder()
                .chart_name("test chart".to_owned())
                .image_width_height((width, height))
                .build(),
        )
        .rgb(
            palette.into_iter()
                .map(|(rgb, _)| rgb)
                .collect(),
        )
        .build();

    // create and save the new BSB/KAP file
    let bsb = KapImageFile::new(header, raster_data)?;
    bsb.into_file("kap_from_png_example.kap")?;
    Ok(())
}
```

### History

It is frustratingly hard to find a formal specification for the `MapTech` BSB/KAP file format. Since the early
2000s, multiple projects have implemented libraries for read/write operations for the KAP format; examples
include [libbsb](https://libbsb.sourceforge.net/) ([also mirrored on github](https://github.com/nohal/libbsb)), [imgkap](https://github.com/nohal/imgkap),
and the [bsb module of GDAL](https://github.com/OSGeo/gdal/tree/master/frmts/bsb). I have taken
inspiration from all of these for this crate. A particular valuable resource has been the `BSB_Test_Dataset_Instructions_for_RNC.pdf`, taken from the legacy
International Hydrographic Organization [found here](https://legacy.iho.int/mtg_docs/com_wg/HSSC/HSSC3/S-64_Edition_2.0.0/RNC_Test_Data_Sets/BSB_TDS/BSB_Test_Dataset_Instructions_for_RNC.pdf).

#### Unstable API

This crate is still very much a work-in-progress. Expect breaking changes between minor
releases until `v1.0`. All the Raw types in this library carry the `#[non_exhaustive]`
attribute and implement the builder pattern. To avoid breaking changes between versions, use
the `Builder` version of the types where possible and set specific fields sparingly. (If
needed, see the [`bon`] crate for information on the builder pattern.)

### License

Dual-licensed under Apache 2.0 and MIT terms.

<!-- cargo-rdme end -->
