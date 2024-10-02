use bon::Builder;

/// Decompressed bitmap of KAP/BSB image embedded raster data
#[derive(Builder, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct BitMap {
    /// The width of the image
    width: u16,
    /// The height of the image
    height: u16,
    /// Image pixels
    pixels: Vec<u8>,
}

impl BitMap {
    /// Creates a new [`BitMap`]
    // TODO:
    #[must_use]
    pub fn empty(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; usize::from(width) * usize::from(height)],
        }
    }

    /// Returns the width of the image
    #[must_use]
    pub const fn width(&self) -> u16 {
        self.width
    }

    /// Returns the height of the image
    #[must_use]
    pub const fn height(&self) -> u16 {
        self.height
    }

    /// Returns the pixel indexes of the image
    #[must_use]
    pub fn pixel_indexes(&self) -> &[u8] {
        &self.pixels
    }

    /// set the value of a specific pixel
    pub fn set_pixel(&mut self, x: u16, y: u16, value: u8) {
        if x < self.width && y < self.height {
            self.pixels[usize::from(y) * usize::from(self.width) + usize::from(x)] = value;
        }
    }

    // clear the bitmap (set all pixels to 0)
    fn _clear(&mut self) {
        for pixel in &mut self.pixels {
            *pixel = 0;
        }
    }

    // get an entire row of the bitmap
    fn _get_row(&self, y: u16) -> Option<&[u8]> {
        if y < self.height {
            let start_index = usize::from(y) * usize::from(self.width);
            let end_index = start_index + usize::from(self.width);
            Some(&self.pixels[start_index..end_index])
        } else {
            None
        }
    }

    // get an entire row of the bitmap
    pub(crate) fn get_row_mut(&mut self, y: u16) -> Option<&mut [u8]> {
        if y < self.height {
            let start_index = usize::from(y) * usize::from(self.width);
            let end_index = start_index + usize::from(self.width);
            Some(&mut self.pixels[start_index..end_index])
        } else {
            None
        }
    }
}