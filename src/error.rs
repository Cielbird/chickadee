#![allow(unused)]
use std::fmt;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ImageError(image::ImageError),
    IoError(std::io::Error),
    ObjLoadError(tobj::LoadError),
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
        Self::IoError(e)
    }
}

impl From<image::ImageError> for Error {
    fn from(e: image::ImageError) -> Self {
        Self::ImageError(e)
    }
}

impl From<tobj::LoadError> for Error {
    fn from(e: tobj::LoadError) -> Self {
        Self::ObjLoadError(e)
    }
}
