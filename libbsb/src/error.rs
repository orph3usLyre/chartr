use crate::serde;
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
/// Possible `libbsb` errors
pub enum Error {
    /// Error returned if file parse fails
    #[error("Parse error")]
    Parse(#[from] serde::error::Error),

    /// Error returned if I/O operations fail
    #[error("I/O error")]
    IO(#[from] std::io::Error),

    /// Error returned if width/height of header do not match
    /// the width/height of the bitmap
    #[error("Header width/height does not match bitmap width/height. header: {header:?}, raster_length: {raster_length:?}")]
    MismatchWidthHeight {
        /// header width/height
        header: (u16, u16),
        /// raster data length
        raster_length: usize,
    },

    /// Error returned if provided depth does not correspond to one of the supported
    /// BSB/KAP depth values
    #[error("Unsupported depth `{0}`. Suported depths are: 1, 4, 7")]
    UnsupportedDepth(u8),

    /// Error returned if user attempted to use a palette that does not exist in the BSB/KAP image
    /// header
    #[error("Palette does not exist")]
    NonExistentPalette,

    #[error("Other: `{0}`")]
    /// Other bubbled errors, such as conversion errors
    Other(String),
}
