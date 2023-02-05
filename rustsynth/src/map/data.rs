use rustsynth_sys as ffi;
use std::ops::Deref;

#[derive(Clone, Copy, Debug)]
pub enum DataType {
    String,
    Binary,
    Unknown,
}

pub fn handle_data_hint(hint: i32) -> DataType {
    match hint {
        x if x == ffi::VSDataTypeHint::dtBinary as i32 => DataType::Binary,
        x if x == ffi::VSDataTypeHint::dtUtf8 as i32 => DataType::String,
        x if x == ffi::VSDataTypeHint::dtUnknown as i32 => DataType::Unknown,
        _ => unreachable!(),
    }
}

pub struct Data<'elem> {
    inner: &'elem [u8],
}

impl<'elem> Deref for Data<'elem> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'elem> Data<'elem> {
    pub(crate) fn from_slice(slice: &'elem [u8]) -> Self {
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
