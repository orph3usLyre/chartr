use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("depth not found in header")]
    MissingDepth,
    #[error("width/height not found in header")]
    MissingWidthHeight,
}
