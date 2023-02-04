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
    hint: DataType,
}

impl<'elem> Deref for Data<'elem> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'elem> Data<'elem> {
    pub fn type_hint(&self) -> DataType {
        self.hint
    }

    pub(crate) fn from_slice(slice: &'elem [u8], hint: DataType) -> Self {
        Self { inner: slice, hint }
    }
}
