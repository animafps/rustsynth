//! VapourSynth map and structs to manipulate a map.
//!
use rustsynth_sys as ffi;
use std::ffi::{c_char, CStr, CString};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::slice;

use crate::api::API;

/// The types of values that can be set in a map
#[derive(Debug, PartialEq)]
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
#[derive(Clone, Debug, PartialEq)]
pub enum DataType<'a> {
    /// Unkown pointer to data
    Unknown(*const c_char),
    /// A valid UTF-8 string
    String(String),
    /// A slice of bytes
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
/// Keys are string slices
///
/// values may be a vector of
/// - integers (`Vec<i64>`)
/// - floats (`Vec<f64>`)
/// - a data type (array of strings or raw binary) (`Vec<DataType<'a>>`) see [DataType]
/// - nodes (`Vec<Node>`) see [Node]
/// - functions (`Vec<Function>`) see [Function]
/// - filters (`Vec<Filter>`) see [Filter]
/// - frames (`Vec<Frame>`) see [Frames]
/// 
/// or empty
///
/// # Examples
///
/// ```
/// use rustsynth::map::{Map, Value};
/// let map = Map::new();
/// map.set("best", Value::Int(vec![1,26,4])).unwrap();
/// assert_eq!(map.get("best").unwrap(), Value::Int(vec![1,26,4]))
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

    /// Fetches the `Value` enum associated with the key from the map
    ///
    /// The function will return a [MapPropError] if there was a problem getting the value from the Map
    pub fn get(&self, key: &str) -> Result<Value, MapPropError> {
        let value_type = self.get_type(key);
        let ckey = CString::new(key).unwrap();
        match value_type {
            ValueType::Int => Ok(Value::Int(unsafe {
                API::get_cached().map_get_int_array(self.ptr(), ckey.as_ptr())
            })),
            ValueType::Float => Ok(Value::Float(unsafe {
                API::get_cached().map_get_float_array(self.ptr(), ckey.as_ptr())
            })),
            ValueType::Data => Ok(Value::Data(
                DataIter {
                    map: self,
                    len: self.num_keys(),
                    counter: 0,
                    key: ckey.as_ptr(),
                }
                .collect(),
            )),
            ValueType::Unset => Ok(Value::Empty),
            _ => unreachable!(),
        }
    }

    /// The number of elements at the associated key
    pub fn num_elements(&self, key: &str) -> i32 {
        let key = CString::new(key).unwrap();
        unsafe { API::get_cached().map_num_elements(self.ptr(), key.as_ptr()) }
    }

    /// Returns the type of value at the associated key
    pub fn get_type(&self, key: &str) -> ValueType {
        let key = CString::new(key).unwrap();
        unsafe {
            API::get_cached()
                .map_get_type(self.ptr(), key.as_ptr())
                .try_into()
                .unwrap()
        }
    }

    /// Sets a value at a key
    ///
    /// if the key is not present then will create a key
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
            Value::Empty => unsafe {
                API::get_cached().map_set_empty(self.ptr(), key.as_ptr())
            }
            _ => unreachable!(),
        };
        if status == 0 {
            Ok(())
        } else if status == 1 {
            Err("Size is negative")
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

    /// Returns a key from a map.
    ///
    /// # Panics
    /// Panics if `index >= self.key_count()`.
    #[inline]
    pub(crate) fn key_raw(&self, index: usize) -> &CStr {
        assert!(index <= self.num_keys());
        let index = index as i32;

        unsafe { CStr::from_ptr(API::get_cached().map_get_key(self.handle.as_ptr(), index)) }
    }

    /// Returns a key from a map.
    ///
    /// # Panics
    /// Panics if `index >= self.num_keys()`.
    #[inline]
    pub fn key(&self, index: usize) -> &str {
        self.key_raw(index).to_str().unwrap()
    }

    /// Returns an iterator visiting all keys  in arbitrary order.
    pub fn keys(&self) -> Keys<'_> {
        Keys { inner: self.iter() }
    }

    /// Returns an iterator visiting all key-value pairs in arbitrary order. The iterator element type is `(&'elem str, &'elem Value)`
    pub fn iter(&self) -> Iter<'_> {
        Iter::new(self)
    }

    /// Returns an iterator visiting all values in arbitrary order.
    pub fn values(&self) -> Values<'_> {
        Values { inner: self.iter() }
    }

    /// Retuns the number of keys
    pub fn num_keys(&self) -> usize {
        unsafe {
            API::get_cached()
                .map_num_keys(self.handle.as_ptr())
                .try_into()
                .unwrap()
        }
    }

    /// Returns `true` if the number of keys in the array are equal to 0
    ///
    /// `false` otherwise
    pub fn is_empty(&self) -> bool {
        self.num_keys() == 0
    }
}

impl<'a> IntoIterator for Map<'a> {
    type Item = (&'a str, Value<'a>);
    type IntoIter = IntoIter<'a>;

    /// Self consuming iter over Key-values in the `Map`
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            map: self,
            items: self.num_keys(),
            counter: 0,
        }
    }
}

pub struct IntoIter<'a> {
    map: Map<'a>,
    items: usize,
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
    items: usize,
    counter: usize,
}

impl<'a> Iter<'a> {
    pub(crate) fn new(map: &'a Map) -> Self {
        Iter {
            map,
            items: map.num_keys(),
            counter: 0,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a str, Value<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter == self.items.try_into().unwrap() {
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
#[derive(Clone, Debug, PartialEq)]
pub enum Value<'a> {
    Int(Vec<i64>),
    Float(Vec<f64>),
    Data(Vec<DataType<'a>>),
    Empty,
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
            _ => panic!("Not an integer value"),
        }
    }

    /// Exposes the inner value of the float element
    ///
    /// # Panics
    ///
    /// Will panic if not an instance of an float value
    pub fn unwrap_float(self) -> Vec<f64> {
        match self {
            Self::Float(val) => val,
            _ => panic!("Not a float value"),
        }
    }

    /// Exposes the inner value of the data element
    ///
    /// # Panics
    ///
    /// Will panic if not an instance of an data value
    pub fn unwrap_data(self) -> Vec<DataType<'a>> {
        match self {
            Self::Data(val) => val,
            _ => panic!("Not a data value"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DataIter<'a> {
    map: &'a Map<'a>,
    len: usize,
    counter: usize,
    key: *const c_char,
}

impl<'a> DataIter<'a> {
    fn new(map: &'a Map, key: &'a str) -> Self {
        let key = CString::new(key).unwrap();
        let len = map.num_keys();
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

/// The error variants associated with getting and setting values in a [Map]
///
/// See [Map::get()], [Map::set()]
#[derive(Debug)]
pub enum MapPropError {
    /// There exists no value associated with this key
    Unset,
    /// Incorrect type
    Type,
    /// No value exists at this index
    Index,
}

impl MapPropError {
    fn handle(int: i32) -> Self {
        match int {
            int if int == ffi::VSMapPropertyError::peUnset as i32 => Self::Unset,
            int if int == ffi::VSMapPropertyError::peIndex as i32 => Self::Index,
            int if int == ffi::VSMapPropertyError::peType as i32 => Self::Type,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_set() {
        let map = Map::new();
        map.set("best", Value::Int(vec![1, 26, 4])).unwrap();
    }

    #[test]
    fn int_type() {
        let map = Map::new();
        map.set("best", Value::Int(vec![1, 26, 4])).unwrap();
        assert_eq!(map.get_type("best"), ValueType::Int);
    }

    #[test]
    fn int_get() {
        let map = Map::new();
        map.set("best", Value::Int(vec![1, 26, 4])).unwrap();
        assert_eq!(map.get("best").unwrap(), Value::Int(vec![1, 26, 4]))
    }
}
