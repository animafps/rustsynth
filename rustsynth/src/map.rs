//! VapourSynth map.
use rustsynth_sys as ffi;
use std::ffi::{c_char, CStr, CString};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::slice;

use crate::api::API;

/// The types of values that can be set in a map
#[derive(Debug)]
pub enum ValueType {
    Unset,
    Int,
    Float,
    Data,
    Function,
    VideoNode,
    AudioNode,
    VideoFrame,
    AudioFrame,
}

/// The types of data that can be set as data type
#[derive(Clone, Debug)]
pub enum DataType<'a> {
    Unknown(*const c_char),
    String(String),
    Binary(&'a [u8]),
}

impl std::convert::TryFrom<i32> for ValueType {
    fn try_from(value: i32) -> Result<ValueType, Self::Error> {
        match value {
            0 => Ok(ValueType::Unset),
            1 => Ok(ValueType::Int),
            2 => Ok(ValueType::Float),
            3 => Ok(ValueType::Data),
            4 => Ok(ValueType::Function),
            5 => Ok(ValueType::VideoNode),
            6 => Ok(ValueType::AudioNode),
            7 => Ok(ValueType::VideoFrame),
            8 => Ok(ValueType::AudioFrame),
            _ => Err("Not a valid map value type"),
        }
    }

    type Error = &'static str;
}

/// A VapourSynth map.
///
/// A map contains key-value pairs where the value is zero or more elements of a certain type.
///
/// values may be
/// - an integer (`i64`)
/// - an array of integers (`Vec<i64>`)
///
/// It is currently immutable
///
/// # Examples
///
/// ```
/// use rustsynth::map::Map;
/// let map = Map::new();
/// ```
#[derive(Debug, Copy, Clone)]
pub struct Map<'elem> {
    // The actual mutability of this depends on whether it's accessed via `&Map` or `&mut Map`.
    handle: NonNull<ffi::VSMap>,
    _elem: PhantomData<&'elem ()>,
}

unsafe impl<'elem> Send for Map<'elem> {}
unsafe impl<'elem> Sync for Map<'elem> {}

#[doc(hidden)]
impl<'elem> Deref for Map<'elem> {
    type Target = ffi::VSMap;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.handle.as_ref() }
    }
}

#[doc(hidden)]
impl<'elem> DerefMut for Map<'elem> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.handle.as_mut() }
    }
}

///impl<'elem> Drop for Map<'elem> {
///    #[inline]
///    fn drop(&mut self) {
///        unsafe {
///            API::get_cached().free_map(self.ptr());
///        }
///    }
///}

impl<'elem> Default for Map<'elem> {
    fn default() -> Self {
        Map {
            handle: unsafe { NonNull::new_unchecked(API::get_cached().create_map()) },
            _elem: PhantomData,
        }
    }
}

impl<'elem> Map<'elem> {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Default::default()
    }

    /// Wraps pointer into `Map`.
    ///
    /// # Safety
    /// The caller needs to ensure the pointer is valid, the element lifetime is valid, and that
    /// the resulting `Map` gets put into `MapRef` or `MapRefMut` or `OwnedMap` correctly.
    #[inline]
    pub(crate) unsafe fn from_ptr(handle: *const ffi::VSMap) -> Self {
        Self {
            handle: NonNull::new_unchecked(handle as *mut ffi::VSMap),
            _elem: PhantomData,
        }
    }

    ///
    pub fn get(&self, key: &str) -> Result<Value, &'static str> {
        let value_type = self.get_type(key);
        let ckey = CString::new(key).unwrap();
        match value_type {
            ValueType::Int => Ok(Value::Int(unsafe {
                API::get_cached().map_get_int_array(self.ptr(), ckey.as_ptr())
            })),
            ValueType::Float => Ok(Value::Float(unsafe {
                API::get_cached().map_get_float_array(self.ptr(), ckey.as_ptr())
            })),
            ValueType::Data => {
                Ok(Value::Data(DataIter { map: self, len: self.len(), counter: 0, key: ckey.as_ptr() }))
            }
            _ => panic!("Not implemented"),
        }
    }

    pub fn num_elements(&self, key: &str) -> i32 {
        let key = CString::new(key).unwrap();
        unsafe { API::get_cached().map_num_elements(self.ptr(), key.as_ptr()) }
    }

    pub fn get_type(&self, key: &str) -> ValueType {
        let key = CString::new(key).unwrap();
        unsafe {
            API::get_cached()
                .map_get_type(self.ptr(), key.as_ptr())
                .try_into()
                .unwrap()
        }
    }

    pub fn set(&self, key: &str, data: Value) -> Result<(), &'static str> {
        let key = CString::new(key).unwrap();
        let status = match data {
            Value::Int(val) => unsafe {
                API::get_cached().map_set_int_array(
                    self.ptr(),
                    key.as_ptr(),
                    val.as_ptr(),
                    val.len().try_into().unwrap(),
                )
            },
            Value::Float(val) => unsafe {
                API::get_cached().map_set_float_array(
                    self.ptr(),
                    key.as_ptr(),
                    val.as_ptr(),
                    val.len().try_into().unwrap(),
                )
            },
            _ => todo!(),
        };
        if status == 0 {
            Ok(())
        } else {
            Err("Unkown Error")
        }
    }

    /// Clears the map.
    #[inline]
    pub fn clear(&mut self) {
        unsafe {
            API::get_cached().clear_map(self.ptr());
        }
    }

    pub(crate) fn ptr(&self) -> *mut ffi::VSMap {
        self.handle.as_ptr()
    }

    /// Returns the number of keys contained in a map.
    #[inline]
    pub fn key_count(&self) -> usize {
        let count = unsafe { API::get_cached().map_num_keys(self.ptr()) };
        debug_assert!(count >= 0);
        count as usize
    }

    /// Returns a key from a map.
    ///
    /// # Panics
    /// Panics if `index >= self.key_count()`.
    #[inline]
    pub(crate) fn key_raw(&self, index: usize) -> &CStr {
        assert!(index < self.key_count());
        let index = index as i32;

        unsafe { CStr::from_ptr(API::get_cached().map_get_key(self.handle.as_ptr(), index)) }
    }

    /// Returns a key from a map.
    ///
    /// # Panics
    /// Panics if `index >= self.key_count()`.
    #[inline]
    pub fn key(&self, index: usize) -> &str {
        self.key_raw(index).to_str().unwrap()
    }

    /// An iterator visiting all keys  in arbitrary order.
    pub fn keys(&self) -> Keys<'_> {
        Keys { inner: self.iter() }
    }

    /// An iterator visiting all key-value pairs in arbitrary order. The iterator element type is (&'elem str, &'elem Value)
    pub fn iter(&self) -> Iter<'_> {
        Iter::new(self)
    }

    /// An iterator visiting all values in arbitrary order.
    pub fn values(&self) -> Values<'_> {
        Values { inner: self.iter() }
    }

    pub fn len(&self) -> i32 {
        unsafe { API::get_cached().map_num_keys(self.handle.as_ptr()) }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a> IntoIterator for Map<'a> {
    type Item = (&'a str, Value<'a>);
    type IntoIter = IntoIter<'a>;

    /// Self consuming iter over Key-values in the `Map`
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            map: self,
            items: self.len(),
            counter: 0,
        }
    }
}

pub struct IntoIter<'a> {
    map: Map<'a>,
    items: i32,
    counter: usize,
}

impl<'a> Iterator for IntoIter<'a> {
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }

    type Item = (&'a str, Value<'a>);
}

/// An iterator over the keys of a `Map`.
///
/// This `struct` is created by the [`keys`] method on [`Map`]. See its
/// documentation for more.
///
/// [`keys`]: Map::keys
///
/// # Example
///
/// ```
/// use rustsynth::map::Map;
/// let map = Map::new();
///
/// let iter_keys = map.keys();
/// ```
pub struct Keys<'a> {
    inner: Iter<'a>,
}

impl<'a> Iterator for Keys<'a> {
    fn next(self: &mut Keys<'a>) -> Option<Self::Item> {
        Some(self.inner.next()?.0)
    }

    type Item = &'a str;
}

/// An iterator over the entries of a `Map`.
///
/// This `struct` is created by the [`iter`] method on [`Map`]. See its
/// documentation for more.
///
/// [`iter`]: Map::iter
///
/// # Example
///
/// ```
/// use rustsynth::map::Map;
/// let map = Map::new();
///
/// let iter = map.iter();
/// ```
pub struct Iter<'a> {
    map: &'a Map<'a>,
    items: i32,
    counter: usize,
}

impl<'a> Iter<'a> {
    pub(crate) fn new(map: &'a Map) -> Self {
        Iter {
            map,
            items: map.len(),
            counter: 0,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a str, Value<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter > self.items.try_into().unwrap() {
            None
        } else {
            let key = self.map.key(self.counter);
            self.counter += 1;
            Some((key, self.map.get(key).unwrap()))
        }
    }
}

/// An iterator over the values of a `Map`.
///
/// This `struct` is created by the [`values`] method on [`Map`]. See its
/// documentation for more.
///
/// [`values`]: Map::values
///
/// # Example
///
/// ```
/// use rustsynth::map::Map;
/// let map = Map::new();
///
/// let iter = map.values();
/// ```
pub struct Values<'a> {
    inner: Iter<'a>,
}

impl<'a> Iterator for Values<'a> {
    fn next(self: &mut Values<'a>) -> Option<Self::Item> {
        Some(self.inner.next()?.1)
    }

    type Item = Value<'a>;
}

/// A enum of the elements of a value in a map
#[derive(Clone, Debug)]
pub enum Value<'a> {
    Int(Vec<i64>),
    Float(Vec<f64>),
    Data(DataIter<'a>),
}

impl<'a> Value<'a> {
    /// Exposes the inner value of the integer element
    ///
    /// # Panics
    ///
    /// Will panic if not an instance of an integer value
    pub fn unwrap_int(self) -> Vec<i64> {
        match self {
            Self::Int(val) => val,
            _ => panic!("Not an integer"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DataIter<'a> {
    map: &'a Map<'a>,
    len: i32,
    counter: usize,
    key: *const c_char,
}

impl<'a> DataIter<'a> {
    fn new(map: &'a Map, key: &'a str) -> Self {
        let key = CString::new(key).unwrap();
        let len = unsafe { API::get_cached().map_num_elements(map.ptr(), key.as_ptr()) };
        Self {
            map,
            len,
            counter: 0,
            key: key.as_ptr(),
        }
    }
}

impl<'a> Iterator for DataIter<'a> {
    type Item = DataType<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.len > self.counter.try_into().unwrap() {
            return None;
        }
        let ptr = unsafe {
            API::get_cached().map_get_data(
                self.map.ptr(),
                self.key,
                self.counter.try_into().unwrap(),
            )
        };
        match unsafe {
            API::get_cached().map_get_data_type_hint(
                self.map.ptr(),
                self.key,
                self.counter.try_into().unwrap(),
            )
        } {
            1 => {
                self.counter += 1;
                Some(unsafe { DataType::String(CStr::from_ptr(ptr).to_string_lossy().to_string()) })
            }
            0 => {
                let data = Some(unsafe {
                    DataType::Binary(slice::from_raw_parts(
                        ptr as *const u8,
                        API::get_cached()
                            .map_get_data_size(
                                self.map.ptr(),
                                self.key,
                                self.counter.try_into().unwrap(),
                            )
                            .try_into()
                            .unwrap(), // `len` may not be correct as assuming each part of the slice is a byte
                    ))
                });
                self.counter += 1;
                data
            }
            _ => Some(DataType::Unknown(ptr)),
        }
    }
}
