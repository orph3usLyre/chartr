use std::ops::Shr;
use tracing::{debug, instrument};

#[instrument(skip(buf_out), level = "trace")]
fn bsb_compress_nb(buf_out: &mut Vec<u8>, nb: u16, mut pixel: u8, max: u16) -> u16 {
    let mut count: u16 = 0;
    // If nb is larger than max, then is it the multiplier?
    // Or does it mean that we require more bytes to store the
    // run length?
    if nb > max {
        count = bsb_compress_nb(buf_out, nb.shr(7), pixel | 0x80, max);
        buf_out.push((nb & 0x7F) as u8 | (pixel & 0x80));
        return count + 1;
    }
    pixel |= u8::try_from(nb).unwrap_or(u8::MAX);
    if pixel.eq(&0) {
        buf_out.push(0x80);
        count += 1;
    }

    buf_out.push(pixel);

    count += 1;
    return count;
}

#[instrument(skip(compressed_buf, to_compress), level = "trace")]
pub fn compress_bsb_row(
    to_compress: &[u8],
    compressed_buf: &mut Vec<u8>,
    depth: u8,
    line_number: u16,
    width_in: u16,
    width_out: u16,
) -> u16 {
    let dec = 7 - depth;
    let max = (1 << dec) - 1;
    debug!("Dec is: {dec}\tMax is: {max}");

    // write the line number
    let mut ibuf = bsb_compress_nb(compressed_buf, line_number, 0, 0x7F);

    let (mut ipixel_in, mut ipixel_out) = (0u16, 0u16);

    while ipixel_in < width_in {
        let last_pixel = u16::from(to_compress[ipixel_in as usize]);
        ipixel_in += 1;
        ipixel_out += 1;

        // count the length of the same pixel
        let mut run_length = 0u16;

        while ipixel_in < width_in && u16::from(to_compress[ipixel_in as usize]) == last_pixel {
            ipixel_in += 1;
            ipixel_out += 1;
            run_length += 1;
        }

        // Extend the run length based on output width
        let xout = ((ipixel_in << 1) + 1).saturating_mul(width_out / (width_in << 1));
        if xout > ipixel_out {
            run_length += xout - ipixel_out;
            ipixel_out = xout;
        }

        // write pixel
        ibuf += bsb_compress_nb(
            compressed_buf,
            run_length,
            u8::try_from(last_pixel << dec).unwrap_or(u8::MAX),
            max,
        );
    }
    compressed_buf.push(0);
    ibuf + 1
}
