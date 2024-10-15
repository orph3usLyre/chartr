use crate::serde;
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
/// Possible `libbsb` errors
pub enum Error {
    /// Error returned if `.KAP` file parse fails
    #[error("parse error")]
    ParseError(#[from] serde::error::Error),
    /// Error returned if width/height of header do not match
    /// the width/height of the bitmap
    #[error("header width/height does not match bitmap width/height. header: {header:?}, raster_length: {raster_length:?}")]
    MismatchWidthHeight {
        /// header width/height
        header: (u16, u16),
        /// bitmap width/height
        raster_length: usize,
    },
}
