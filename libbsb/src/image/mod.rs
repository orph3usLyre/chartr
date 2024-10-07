#![allow(clippy::module_name_repetitions)]

pub(crate) mod bitmap;
pub(crate) mod compress;
pub(crate) mod decompress;
pub(crate) mod header;
pub(crate) mod index;

/// Module containing raw header types
///
/// Types in this module are considered "unchecked", the responsibility
/// of upholding validity is on the user
pub mod raw {
    /// Contains types related to BSB/KAP image file headers
    pub mod header {
        pub use crate::image::header::{
            AdditionalParameters, ChartEditionParameters, DetailedParameters, GeneralParameters,
            ImageHeader, NTMRecord, Polynomial, Ref,
        };
    }
}

use crate::{error::Error, CTRL_Z};
use anyhow::{ensure, Context, Result};
use bitmap::BitMap;
use compress::compress_bsb_row;
use decompress::{BsbDecompressor, Decompressor};
use header::ImageHeader;
use index::read_index;
use std::{
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write},
    path::Path,
};
use tracing::{debug, info, trace};

/// A typed representation of a BSB/KAP image file
#[derive(Debug, PartialEq, PartialOrd)]
pub struct KapImageFile {
    header: ImageHeader,
    bitmap: BitMap,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
/// The different palettes a BSB/KAP image file can contain
pub enum ColorPalette {
    /// Default color palette (RGB)
    Rgb,
    /// Day color palette (DAY)
    Day,
    /// Dusk color palette (DSK)
    Dsk,
    /// Night color palette (NGT)
    Ngt,
    /// Night red color palette (NGR)
    Ngr,
    /// Gray color palette (NGR)
    Gry,
    /// Optional color palette (PRC)
    Prc,
    /// Optional gray color palette (PRG)
    Prg,
}

#[derive(Default, Debug, Eq, PartialEq, PartialOrd, Ord, Copy, Clone)]
/// Image depth
/// BSB/KAP image files only support 1, 4, and 7 pixel depth
pub enum Depth {
    /// Image depth 1: 1 bit is used to represent the depth of the image
    // TODO: use builder in header parse to remove default
    #[default]
    One,
    /// Image depth 4: 4 bits are used to represent the depth of the image
    Four,
    /// Image depth 7: 7 bits are used to represent the depth of the image
    Seven,
}

impl KapImageFile {
    // /// Creates a new [`KapImageFile`]
    // ///
    // /// # Errors
    // /// This function errors if the width and height of the image header don't match
    // /// the width and height of the bitmap
    // ///
    // pub fn new(header: ImageHeader, bitmap: BitMap) -> Result<Self, Error> {
    //     if header.general_parameters.image_width_height != (bitmap.width(), bitmap.height()) {
    //         return Err(Error::MismatchWidthHeight {
    //             header: header.general_parameters.image_width_height,
    //             raster_length: (bitmap.width(), bitmap.height()),
    //         });
    //     }
    //     Ok(Self { header, bitmap })
    // }

    /// Creates a new [`KapImageFile`]
    ///
    /// # Errors
    /// This function errors if the width and height of the image header don't match
    /// the width and height of the bitmap
    ///
    pub fn new(header: ImageHeader, raster_data: Vec<u8>) -> Result<Self, Error> {
        let width = header.width();
        let height = header.height();
        if raster_data.len() != usize::from(width) * usize::from(height) {
            return Err(Error::MismatchWidthHeight {
                header: header.general_parameters.image_width_height,
                header_calculated: width as usize * height as usize,
                raster_length: raster_data.len(),
            });
        }
        Ok(Self {
            header,
            bitmap: BitMap::new(width, height, raster_data),
        })
    }

    /// Returns a reference to the [`ImageHeader`]
    #[must_use]
    pub const fn header(&self) -> &ImageHeader {
        &self.header
    }

    /// Reads a BSB file and returns just the header
    ///
    /// # Errors
    ///
    /// This function errors if the header data is invalid
    pub(crate) fn get_header(r: &mut (impl BufRead + Seek)) -> Result<ImageHeader> {
        match r.stream_position()? {
            0 => {}
            _ => r.rewind()?,
        }
        let mut header = Vec::new();
        let read = r.read_until(CTRL_Z, &mut header)?;
        debug!("read {read} for header");
        let header = String::from_utf8(header)?;
        trace!("Header:\n{}", &header);
        header.parse()
    }

    /// Tries to read a [`Self`] from an buffer
    ///
    /// # Errors
    ///
    /// This function will error if the underlying buffer is invalid data for any reason:
    /// - KAP header is invalid
    /// - Depth is not one of (1, 4, 7)
    /// - Raster index has an invalid size
    // TODO: more
    pub fn from_reader(mut r: impl BufRead + Seek) -> Result<Self> {
        let header = Self::get_header(&mut r)?;
        // keep reading here since we haven't seeked after getting header

        // TODO: replace with `skip_until` when <https://github.com/rust-lang/rust/issues/111735>
        // lands on stable
        let mut dump = vec![];
        let read = r.read_until(0x0, &mut dump)?;
        info!("read {read} until data start");
        drop(dump);

        // Binary section consisting of:
        // One or more rows of run-length compressed raster data
        // An index table consisting of 32-bit integers storing file offsets to each image row
        //https://www.yachtingmonthly.com/cruising-life/through-the-french-canals-to-the-med-in-a-yacht-95086
        let mut depth = [0];
        let read = r.read(&mut depth)?;
        let depth = depth[0];
        debug!("read {read} for depth {}", depth);

        // keep pointer here to begin image
        let raster_start = r.stream_position()?;

        // TODO: warn?
        ensure!(
            u8::from(header.ifm) == depth,
            "Kap image file must contain image depth"
        );

        let (width, height) = header.general_parameters.image_width_height;
        debug!("Raster width, height: {:?}", (&width, &height));

        debug!("OST: {:?}", &header.ost);

        // index is the ordered image indexes
        let (index, _raster_end) = read_index(0, &mut r, height)?;
        debug!("Index len: {}", index.len());

        // go back to where raster data starts before doing the seek dance?
        r.seek(SeekFrom::Start(raster_start))?;

        let mut bitmap = BitMap::empty(width, height);

        debug!("Decompressing BSB bitmap");
        match depth {
            1 => {
                Decompressor::<1>::decompress_bsb_from_reader(&mut r, &mut bitmap, &index)?;
            }
            4 => {
                Decompressor::<4>::decompress_bsb_from_reader(&mut r, &mut bitmap, &index)?;
            }
            7 => {
                Decompressor::<7>::decompress_bsb_from_reader(&mut r, &mut bitmap, &index)?;
            }
            _ => unimplemented!("Only 1, 4, and 7 pixel depth is supported by KAP/BSB"),
        }

        Ok(Self { header, bitmap })
    }
    /// Tries to read [`Self`] from a provided file path
    ///
    /// # Errors
    ///
    /// This function will error if the file cannot be opened or if the file contains invalid data.
    /// See [`Self::from_reader`] for potential errors
    pub fn from_file<P: AsRef<Path>>(filename: P) -> Result<Self> {
        let file = File::open(filename)?;
        Self::from_reader(BufReader::new(file))
    }

    /// Attempts to serialize and save [`Self`] as a file at the provided path
    ///
    /// # Errors
    ///
    /// This will error if unable to open and/or write to the provided filename
    ///
    pub fn into_file(mut self, filename: impl AsRef<Path>) -> Result<()> {
        let f = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(filename)?;
        let mut f = BufWriter::new(f);
        let h = self.header.into_header_format();
        trace!("HEADER:\n{h}");
        f.write(h.as_bytes()).context("Write header to file")?;
        let mut index = Vec::new();
        let mut i = 0;
        // TODO: This should take expected size
        let mut compressed_buf = Vec::new();
        let depth = self.header.ifm.into();
        ensure!(depth <= 7, "BSB depth cannot be more than 7");
        f.write(&[CTRL_Z, 0x00, depth])
            .context("Header terminating characters")?;
        let width_in = self.width();
        // TODO: what is width out? related to pixel compression level?
        let width_out = width_in;
        while let Some(ref mut row) = self.bitmap.get_row_mut(i) {
            let _len = compress_bsb_row(row, &mut compressed_buf, depth, i, width_in, width_out);

            // write index by adding the file position before writing the row
            index.push(f.stream_position().context("stream position")?);
            f.write_all(&compressed_buf)?;
            compressed_buf.clear();
            i += 1;
        }
        index.push(f.stream_position().context("stream position")?);
        let index: Vec<_> = index
            .iter()
            .flat_map(|&i| u32::try_from(i).unwrap_or(u32::MAX).to_be_bytes())
            .collect();
        f.write_all(&index)?;
        f.flush()?;
        info!("Finished writing to file");

        Ok(())
    }

    /// Returns an iterator over the palette colors the pixel indexes correspond to
    /// (defined in [`ImageHeader::rgb`])
    ///
    /// # Errors
    ///
    /// Will return an error if [`ImageHeader::rgb`] is [`None`].
    pub fn as_palette_iter(
        &self,
        palette: ColorPalette,
    ) -> Result<impl Iterator<Item = [u8; 3]> + '_> {
        let rgbs = match palette {
            ColorPalette::Rgb => self.header().rgb.as_ref(),
            ColorPalette::Day => self.header().day.as_ref(),
            ColorPalette::Dsk => self.header().dsk.as_ref(),
            ColorPalette::Ngt => self.header().ngt.as_ref(),
            ColorPalette::Ngr => self.header().ngr.as_ref(),
            ColorPalette::Gry => self.header().gry.as_ref(),
            ColorPalette::Prc => self.header().prc.as_ref(),
            ColorPalette::Prg => self.header().prg.as_ref(),
        }
        .context("get color palette")?;
        // let rgbs = self.header.rgb.as_ref().context("RGB not found")?;
        let out = self.bitmap.pixel_indexes().iter().map(|bsb_p| {
            // NOTE: we subtract one since bsb file indexes start at 1
            <[u8; 3]>::from(rgbs[(*bsb_p as usize).saturating_sub(1)])
        });
        Ok(out)
    }

    /// Returns the image width
    // TODO: can never be different than bitmap
    #[must_use]
    pub const fn width(&self) -> u16 {
        self.header.general_parameters.image_width_height.0
    }

    /// Returns the image height
    // TODO: can never be different than bitmap
    #[must_use]
    pub const fn height(&self) -> u16 {
        self.header.general_parameters.image_width_height.1
    }
}

impl From<Depth> for u8 {
    fn from(value: Depth) -> Self {
        match value {
            Depth::One => 1,
            Depth::Four => 4,
            Depth::Seven => 7,
        }
    }
}

impl From<&Depth> for u8 {
    fn from(value: &Depth) -> Self {
        match value {
            Depth::One => 1,
            Depth::Four => 4,
            Depth::Seven => 7,
        }
    }
}

impl Display for Depth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", u8::from(self))
    }
}

impl TryFrom<u8> for Depth {
    type Error = &'static str;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::One),
            4 => Ok(Self::Four),
            7 => Ok(Self::Seven),
            _ => Err("Only 1, 4, and 7 are valid BSB depths"),
        }
    }
}
