use rustsynth_sys as ffi;
use std::ops::Deref;

#[derive(Clone, Copy, Debug)]
pub enum DataType {
    String = 1,
    Binary = 0,
    Unknown = -1,
}

impl DataType {
    #[must_use] 
    pub const fn from_hint(value: i32) -> Self {
        match value {
            x if x == ffi::VSDataTypeHint::dtBinary as i32 => Self::Binary,
            x if x == ffi::VSDataTypeHint::dtUtf8 as i32 => Self::String,
            x if x == ffi::VSDataTypeHint::dtUnknown as i32 => Self::Unknown,
            _ => Self::Unknown,
        }
    }
}

pub struct Data<'elem> {
    inner: &'elem [u8],
}

impl Deref for Data<'_> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'elem> Data<'elem> {
    pub(crate) const fn from_slice(slice: &'elem [u8]) -> Self {
        Self { inner: slice }
    }
}

impl<'elem> From<&'elem [u8]> for Data<'elem> {
    fn from(value: &'elem [u8]) -> Self {
        Self::from_slice(value)
    }
}

impl<'elem> From<&'elem str> for Data<'elem> {
    fn from(value: &'elem str) -> Self {
        let slice = value.as_bytes();
        Self::from_slice(slice)
    }
}
