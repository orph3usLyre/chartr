#![allow(clippy::module_name_repetitions)]

pub(crate) mod bitmap;
pub(crate) mod compress;
pub(crate) mod decompress;
pub(crate) mod header;
pub(crate) mod index;

/// Module containing raw types
///
/// Types in this module are considered "unchecked"; the responsibility
/// of upholding validity is on the user.
///
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
use tracing::{debug, info, trace, warn};

/// A typed representation of a BSB/KAP image file
///
/// The BSB/KAP image file holds a header, which carries associated metadata,
/// as well as the decompressed raster data of the image.
///
/// The decompressed raster data represents the pixel indices of the image,
/// which map to pixel values through use of the palettes found in [`ImageHeader`].
#[derive(Debug, PartialEq, PartialOrd)]
pub struct KapImageFile {
    header: ImageHeader,
    bitmap: BitMap,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
/// The different palettes a BSB/KAP image file can contain
pub enum ColorPalette {
    /// Default color palette (RGB)
    ///
    /// Corresponds to [`ImageHeader::rgb`]
    Rgb,
    /// Day color palette (DAY)
    ///
    /// Corresponds to [`ImageHeader::day`]
    Day,
    /// Dusk color palette (DSK)
    ///
    /// Corresponds to [`ImageHeader::dsk`]
    Dsk,
    /// Night color palette (NGT)
    ///
    /// Corresponds to [`ImageHeader::ngt`]
    Ngt,
    /// Night red color palette (NGR)
    ///
    /// Corresponds to [`ImageHeader::ngr`]
    Ngr,
    /// Gray color palette (NGR)
    ///
    /// Corresponds to [`ImageHeader::gry`]
    Gry,
    /// Optional color palette (PRC)
    ///
    /// Corresponds to [`ImageHeader::prc`]
    Prc,
    /// Optional gray color palette (PRG)
    ///
    /// Corresponds to [`ImageHeader::prg`]
    Prg,
}

#[derive(Default, Debug, Eq, PartialEq, PartialOrd, Ord, Copy, Clone)]
/// Image depth
/// BSB/KAP image files only support 1, 4, and 7 pixel depth
pub enum Depth {
    /// 1 bit is used to represent the depth of the image
    // TODO: use builder in header parse to remove default
    #[default]
    One,
    /// 4 bits are used to represent the depth of the image
    Four,
    /// 7 bits are used to represent the depth of the image
    Seven,
}

impl KapImageFile {
    /// Creates a new [`Self`]
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
    pub(crate) fn get_header(r: &mut (impl BufRead + Seek)) -> Result<ImageHeader, crate::Error> {
        match r.stream_position()? {
            0 => {}
            _ => r.rewind()?,
        }
        let mut header = Vec::new();
        let read = r.read_until(CTRL_Z, &mut header)?;
        debug!("read {read} for header");
        let header = String::from_utf8(header)
            .map_err(|e| Error::Parse(crate::serde::error::Error::FromUtf8(e)))?;
        trace!("Header:\n{}", &header);
        header.parse()
    }

    /// Tries to read a [`Self`] from an buffer
    ///
    /// # Errors
    ///
    /// This function will error if the underlying buffer contains invalid data:
    /// - KAP header is invalid
    /// - Depth is not one of (1, 4, 7)
    /// - Raster index has an invalid size
    // TODO: more
    pub fn from_reader(mut r: impl BufRead + Seek) -> Result<Self, crate::Error> {
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
        let depth = Depth::try_from(depth[0])?;
        debug!("read {read} for depth {}", depth);

        // keep pointer to where raster data starts
        let raster_start = r.stream_position()?;

        if header.ifm != depth {
            warn!(
                "Depth indicated in header: '{}' does not match depth preceeding raster data: '{}'",
                header.ifm, depth
            );
        }

        let (width, height) = header.general_parameters.image_width_height;
        debug!("Raster width, height: {:?}", (&width, &height));

        debug!("OST: {:?}", &header.ost);

        let (index, _raster_end) = read_index(0, &mut r, height)?;
        debug!("Index len: {}", index.len());

        // go back to where raster data starts before doing the seek dance?
        r.seek(SeekFrom::Start(raster_start))?;

        let mut bitmap = BitMap::empty(width, height);

        debug!("Decompressing BSB bitmap");
        match depth {
            Depth::One => {
                Decompressor::<1>::decompress_bsb_from_reader(&mut r, &mut bitmap, &index)?;
            }
            Depth::Four => {
                Decompressor::<4>::decompress_bsb_from_reader(&mut r, &mut bitmap, &index)?;
            }
            Depth::Seven => {
                Decompressor::<7>::decompress_bsb_from_reader(&mut r, &mut bitmap, &index)?;
            }
        }

        Ok(Self { header, bitmap })
    }

    /// Tries to read [`Self`] from a provided file path
    ///
    /// # Errors
    ///
    /// This function will error if the file cannot be opened or if the file contains invalid data.
    /// See [`Self::from_reader`] for potential errors
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, crate::Error> {
        let file = File::open(path)?;
        Self::from_reader(BufReader::new(file))
    }

    /// Attempts to serialize and save [`Self`] as a file at the provided path
    ///
    /// # Errors
    ///
    /// This will error if unable to open and/or write to the provided filename
    ///
    pub fn into_file(mut self, filename: impl AsRef<Path>) -> Result<(), crate::Error> {
        let f = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(filename)?;
        let mut f = BufWriter::new(f);
        let h = self.header.into_header_format();
        trace!("HEADER:\n{h}");
        let _ = f.write(h.as_bytes())?;
        let mut index = Vec::new();
        let mut i = 0;
        // TODO: This should take expected size
        let mut compressed_buf = Vec::new();
        let depth = self.header.ifm.into();
        let _ = f.write(&[CTRL_Z, 0x00, depth])?;
        let width_in = self.width();
        // TODO: what is width out? related to pixel compression level?
        let width_out = width_in;
        while let Some(ref mut row) = self.bitmap.get_row_mut(i) {
            let _len = compress_bsb_row(row, &mut compressed_buf, depth, i, width_in, width_out);

            // write index by adding the file position before writing the row
            index.push(f.stream_position()?);
            f.write_all(&compressed_buf)?;
            compressed_buf.clear();
            i += 1;
        }
        index.push(f.stream_position()?);
        let index: Vec<_> = index
            .iter()
            .flat_map(|&i| u32::try_from(i).unwrap_or(u32::MAX).to_be_bytes())
            .collect();
        f.write_all(&index)?;
        f.flush()?;
        info!("Finished writing to file");

        Ok(())
    }

    /// Returns an array of the pixel indices raster data. See the [`KapImageFile`] documentation
    /// for more information.
    #[must_use]
    pub fn pixel_indices(&self) -> &[u8] {
        self.bitmap.pixel_indices()
    }

    /// Returns an iterator over the palette colors the pixel indices correspond to
    /// (defined in [`ImageHeader::rgb`])
    ///
    /// # Errors
    ///
    /// Will return an error if [`ImageHeader::rgb`] is [`None`].
    pub fn as_palette_iter(
        &self,
        palette: ColorPalette,
    ) -> Result<impl Iterator<Item = [u8; 3]> + '_, crate::Error> {
        let Some(rgbs) = (match palette {
            ColorPalette::Rgb => self.header().rgb.as_ref(),
            ColorPalette::Day => self.header().day.as_ref(),
            ColorPalette::Dsk => self.header().dsk.as_ref(),
            ColorPalette::Ngt => self.header().ngt.as_ref(),
            ColorPalette::Ngr => self.header().ngr.as_ref(),
            ColorPalette::Gry => self.header().gry.as_ref(),
            ColorPalette::Prc => self.header().prc.as_ref(),
            ColorPalette::Prg => self.header().prg.as_ref(),
        }) else {
            return Err(crate::Error::NonExistentPalette);
        };
        // let rgbs = self.header.rgb.as_ref().context("RGB not found")?;
        let out = self.bitmap.pixel_indices().iter().map(|bsb_p| {
            // NOTE: we subtract one since bsb file indices start at 1
            <[u8; 3]>::from(rgbs[(*bsb_p as usize).saturating_sub(1)])
        });
        Ok(out)
    }

    /// Returns the image width
    #[must_use]
    pub const fn width(&self) -> u16 {
        self.header.general_parameters.image_width_height.0
    }

    /// Returns the image height
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

impl Display for Depth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", u8::from(*self))
    }
}

impl TryFrom<u8> for Depth {
    type Error = crate::Error;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::One),
            4 => Ok(Self::Four),
            7 => Ok(Self::Seven),
            o => Err(Error::UnsupportedDepth(o)),
        }
    }
}
