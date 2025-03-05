use std::fmt;

use image::ImageError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Image(ImageError),
    Io(std::io::Error),
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        // TODO change this in the future
        None
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self { 
        return Self::Io(e);
    }
}

impl From<ImageError> for Error {
    fn from(e: ImageError) -> Self { 
        return Self::Image(e);
    }
}
