use std::io::{BufRead, Bytes, Read, Seek, SeekFrom};

use crate::{image::bitmap::BitMap, Error};

pub trait BsbDecompressor<const DEPTH: u8> {
    fn decompress_bsb_row(
        decompressed_row_buf: &mut [u8],
        stream: &mut impl BufRead,
        width: u16,
    ) -> Result<usize, Error> {
        let decin = 7 - DEPTH;
        let maxin = (1 << decin) - 1;

        // type_in seems to be related to the actual type of encoding - maybe previous formats? (BSB,
        // FN01, etc?)
        let type_in = 0;

        let mut pixel = 0;

        // our data
        let mut stream = stream.bytes();

        let line_number = bsb_decompress_nb(type_in, &mut stream, &mut pixel, 0, 0x7F)?;
        Self::decompress_bsb_row_loop(decompressed_row_buf, stream, pixel, width, maxin, decin)?;

        Ok(line_number as usize)
    }

    fn decompress_bsb_row_loop(
        decompressed_row_buf: &mut [u8],
        stream: Bytes<&mut impl BufRead>,
        pixel: u8,
        image_width: u16,
        maxin: u8,
        decin: u8,
    ) -> Result<(), Error>;

    fn decompress_bsb_from_reader(
        mut r: &mut (impl BufRead + Seek),
        bitmap: &mut BitMap,
        index: &[u64],
    ) -> Result<(), Error> {
        let (width, height) = (bitmap.width(), bitmap.height());
        for (image_row_i, &index_element) in index.iter().enumerate().take(usize::from(height)) {
            let row = u16::try_from(image_row_i).map_err(|e| {
                Error::Other(format!(
                    "Unable to convert index table i to u16. Is the index valid?: {e:?}"
                ))
            })?;
            let Some(row_buf) = bitmap.get_row_mut(row) else {
                return Err(Error::Other(
                    "Unexpected end of BitMap. Is it too short? (rows)".into(),
                ));
            };
            r.seek(SeekFrom::Start(index_element))?;
            let _row = Self::decompress_bsb_row(row_buf, &mut r, width)?;
        }
        Ok(())
    }
}

pub struct Decompressor<const DEPTH: u8>;

impl BsbDecompressor<1> for Decompressor<1> {
    fn decompress_bsb_row_loop(
        decompressed_row_buf: &mut [u8],
        mut stream: Bytes<&mut impl BufRead>,
        mut pixel: u8,
        mut image_width: u16,
        maxin: u8,
        decin: u8,
    ) -> Result<(), Error> {
        let mut xout: u16 = 0;
        while image_width != 0 {
            let mut count = bsb_decompress_nb(0, &mut stream, &mut pixel, decin, maxin)?;
            if count > image_width {
                count = image_width;
            }
            image_width = image_width.saturating_sub(count);
            while count != 0 {
                // TODO: test
                decompressed_row_buf[(xout >> 3) as usize] |= pixel << (7 - (xout & 0x7));
                xout += 1;
                count -= 1;
            }
        }
        Ok(())
    }
}
impl BsbDecompressor<4> for Decompressor<4> {
    fn decompress_bsb_row_loop(
        decompressed_row_buf: &mut [u8],
        mut stream: Bytes<&mut impl BufRead>,
        mut pixel: u8,
        mut image_width: u16,
        maxin: u8,
        decin: u8,
    ) -> Result<(), Error> {
        let mut xout: u16 = 0;
        while image_width != 0 {
            let mut count = bsb_decompress_nb(0, &mut stream, &mut pixel, decin, maxin)?;
            if count > image_width {
                count = image_width;
            }
            image_width = image_width.saturating_sub(count);
            while count != 0 {
                // This isn't efficient, since we use entire bytes to represent 4 bit pixels
                // TODO: Find a way to keep compression while still providing correctly ordered
                // bytes to image encoders
                // This works but fails with the ecoder (makes a double-width image)
                // decompressed_row_buf[(xout >> 1) as usize] |= pixel << (4 - ((xout & 1) << 2));
                // decompressed_row_buf[xout as usize] |= pixel << (4 - ((xout & 1) << 2));
                decompressed_row_buf[xout as usize] = pixel & 0x0F;
                xout += 1;
                count -= 1;
            }
        }
        Ok(())
    }
}
impl BsbDecompressor<7> for Decompressor<7> {
    fn decompress_bsb_row_loop(
        decompressed_row_buf: &mut [u8],
        mut stream: Bytes<&mut impl BufRead>,
        mut pixel: u8,
        mut image_width: u16,
        maxin: u8,
        decin: u8,
    ) -> Result<(), Error> {
        let mut xout: u16 = 0;
        while image_width != 0 {
            let mut count = bsb_decompress_nb(0, &mut stream, &mut pixel, decin, maxin)?;
            if count > image_width {
                count = image_width;
            }
            image_width = image_width.saturating_sub(count);
            while count != 0 {
                decompressed_row_buf[xout as usize] = pixel;
                xout += 1;
                count -= 1;
            }
        }
        Ok(())
    }
}

fn fgetkapc(typein: i32, stream: &mut Bytes<&mut impl BufRead>) -> Result<u8, Error> {
    let mut b = stream
        .next()
        .ok_or_else(|| Error::Other("Unexpected stream end".into()))?;
    if typein == 1025 {
        // FIXME: saturating or wrapping?
        // There is currently no typein
        b = b.map(|b| b.saturating_sub(9));
    }
    b.map_err(|e| Error::Other(format!("Error reading from stream: {e:?}")))
}

fn bsb_decompress_nb(
    typein: i32,
    stream: &mut Bytes<&mut impl BufRead>,
    pixel: &mut u8,
    decin: u8,
    maxin: u8,
) -> Result<u16, Error> {
    let mut c = fgetkapc(typein, stream)?;
    let mut count = u16::from(c) & 0x7f;
    *pixel = u8::try_from(count >> decin).unwrap_or(u8::MAX);
    count &= u16::from(maxin);
    while c & 0x80 != 0 {
        c = fgetkapc(typein, stream)?;
        count = (count << 7) + u16::from(c & 0x7f);
    }
    Ok(count + 1)
}
