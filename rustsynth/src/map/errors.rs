use std::ffi::NulError;
use std::result;

use thiserror::Error;

/// The error type for `Map` operations.
#[derive(Error, Debug, Eq, PartialEq)]
pub enum MapError {
    #[error("The requested key wasn't found in the map")]
    KeyNotFound,
    #[error("The requested index was out of bounds")]
    IndexOutOfBounds,
    #[error("The given/requested value type doesn't match the type of the property")]
    WrongValueType,
    #[error("The key is invalid")]
    InvalidKey(#[from] InvalidKeyError),
    #[error("Couldn't convert to a CString")]
    CStringConversion(#[from] NulError),
    #[error("Failed to create map")]
    CreationFailed,
    #[error("Unknown error (see Map::error())")]
    Error,
    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

impl From<MapError> for String {
    fn from(error: MapError) -> Self {
        error.to_string()
    }
}

/// A specialized `Result` type for `Map` operations.
pub type MapResult<T> = result::Result<T, MapError>;

/// An error indicating the map key is invalid.
#[derive(Error, Debug, Eq, PartialEq)]
#[rustfmt::skip]
pub enum InvalidKeyError {
    #[error("The key is empty")]
    EmptyKey,
    #[error("The key contains an invalid character at index {}", _0)]
    InvalidCharacter(usize),
}
