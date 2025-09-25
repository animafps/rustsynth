//! VapourSynth maps.

use rustsynth_sys as ffi;
use std::borrow::Cow;
use std::ffi::{c_int, CStr, CString};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::{mem, slice};

use crate::api::API;
use crate::frame::Frame;
use crate::function::Function;
use crate::node::Node;

mod errors;
pub use errors::{InvalidKeyError, MapError, MapResult};

mod iterators;
pub use self::iterators::{Keys, ValueIter};

mod value;
pub use self::value::{Value, ValueNotArray, ValueType};

mod data;
pub use self::data::{handle_data_hint, Data, DataType};

#[cfg(test)]
mod tests;

/// A VapourSynth map.
///
/// A map contains key-value pairs where the value is zero or more elements of a certain type.
// This type is intended to be publicly used only in reference form.
#[derive(Debug)]
pub struct Map<'elem> {
    // The actual mutability of this depends on whether it's accessed via `&Map` or `&mut Map`.
    handle: NonNull<ffi::VSMap>,
    _elem: PhantomData<&'elem ()>,
}

/// A reference to a VapourSynth map.
#[derive(Debug)]
pub struct MapRef<'owner, 'elem> {
    // Only immutable references to this are allowed.
    map: Map<'elem>,
    _owner: PhantomData<&'owner ()>,
}

/// A reference to a mutable VapourSynth map.
#[derive(Debug)]
pub struct MapRefMut<'owner, 'elem> {
    // Both mutable and immutable references to this are allowed.
    map: Map<'elem>,
    _owner: PhantomData<&'owner ()>,
}

/// An owned VapourSynth map.
#[derive(Debug)]
pub struct OwnedMap<'elem> {
    // Both mutable and immutable references to this are allowed.
    map: Map<'elem>,
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

impl<'owner, 'elem> Deref for MapRef<'owner, 'elem> {
    type Target = Map<'elem>;

    // Technically this should return `&'owner`.
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<'owner, 'elem> Deref for MapRefMut<'owner, 'elem> {
    type Target = Map<'elem>;

    // Technically this should return `&'owner`.
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<'owner, 'elem> DerefMut for MapRefMut<'owner, 'elem> {
    // Technically this should return `&'owner`.
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

impl<'elem> Drop for OwnedMap<'elem> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            API::get_cached().free_map(&mut self.map);
        }
    }
}

impl<'elem> Deref for OwnedMap<'elem> {
    type Target = Map<'elem>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<'elem> DerefMut for OwnedMap<'elem> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

impl<'elem> OwnedMap<'elem> {
    /// Creates a new map.
    ///
    /// # Panics
    ///
    /// If API error
    #[inline]
    pub fn new() -> Self {
        let api = API::get().unwrap();
        Self {
            map: unsafe { Map::from_ptr(api.create_map()) },
        }
    }

    /// Wraps pointer into `OwnedMap`.
    ///
    /// # Safety
    /// The caller needs to ensure the pointer and the lifetime is valid and that this is an owned
    /// map pointer.
    #[inline]
    pub unsafe fn from_ptr(handle: *mut ffi::VSMap) -> Self {
        Self {
            map: Map::from_ptr(handle),
        }
    }

    pub fn as_ptr(&self) -> *mut ffi::VSMap {
        self.handle.as_ptr()
    }
}

impl<'owner, 'elem> MapRef<'owner, 'elem> {
    /// Wraps pointer into `MapRef`.
    ///
    /// # Safety
    /// The caller needs to ensure the pointer and the lifetimes are valid, and that there are no
    /// mutable references to the given map.
    #[inline]
    pub(crate) unsafe fn from_ptr(handle: *const ffi::VSMap) -> Self {
        Self {
            map: Map::from_ptr(handle),
            _owner: PhantomData,
        }
    }
}

impl<'owner, 'elem> MapRefMut<'owner, 'elem> {
    /// Wraps pointer into `MapRefMut`.
    ///
    /// # Safety
    /// The caller needs to ensure the pointer and the lifetimes are valid, and that there are no
    /// references to the given map.
    #[inline]
    pub(crate) unsafe fn from_ptr(handle: *mut ffi::VSMap) -> Self {
        Self {
            map: Map::from_ptr(handle),
            _owner: PhantomData,
        }
    }
}

/// Turns a `map_get_something()` error into a `Result`.
#[inline]
fn handle_get_prop_error(error: i32) -> MapResult<()> {
    if error == 0 {
        Ok(())
    } else {
        Err(match error {
            x if x == ffi::VSMapPropertyError::peUnset as i32 => MapError::KeyNotFound,
            x if x == ffi::VSMapPropertyError::peType as i32 => MapError::WrongValueType,
            x if x == ffi::VSMapPropertyError::peIndex as i32 => MapError::IndexOutOfBounds,
            x if x == ffi::VSMapPropertyError::peError as i32 => MapError::Error,
            _ => unreachable!(),
        })
    }
}

/// Turns a `map_set_something(maAppend)` error into a `Result`.
#[inline]
fn handle_append_prop_error(error: i32) -> MapResult<()> {
    if error != 0 {
        debug_assert!(error == 1);
        Err(MapError::WrongValueType)
    } else {
        Ok(())
    }
}

impl<'elem> Default for OwnedMap<'elem> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'elem> Map<'elem> {
    /// Wraps pointer into `Map`.
    ///
    /// # Safety
    /// The caller needs to ensure the pointer is valid, the element lifetime is valid, and that
    /// the resulting `Map` gets put into `MapRef` or `MapRefMut` or `OwnedMap` correctly.
    #[inline]
    pub unsafe fn from_ptr(handle: *const ffi::VSMap) -> Self {
        Self {
            handle: NonNull::new_unchecked(handle as *mut ffi::VSMap),
            _elem: PhantomData,
        }
    }

    /// Checks if the key is valid. Valid keys start with an alphabetic character or an underscore,
    /// and contain only alphanumeric characters and underscores.
    pub fn is_key_valid(key: &str) -> Result<(), InvalidKeyError> {
        if key.is_empty() {
            return Err(InvalidKeyError::EmptyKey);
        }

        let mut chars = key.chars();

        let first = chars.next().unwrap();
        if !first.is_ascii_alphabetic() && first != '_' {
            return Err(InvalidKeyError::InvalidCharacter(0));
        }

        for (i, c) in chars.enumerate() {
            if !c.is_ascii_alphanumeric() && c != '_' {
                return Err(InvalidKeyError::InvalidCharacter(i + 1));
            }
        }

        Ok(())
    }

    /// Checks if the key is valid and makes it a `CString`.
    #[inline]
    pub(crate) fn make_raw_key(key: &str) -> MapResult<CString> {
        Map::is_key_valid(key)?;
        Ok(CString::new(key).unwrap())
    }

    /// Clears the map.
    #[inline]
    pub fn clear(&mut self) {
        unsafe {
            API::get_cached().clear_map(self);
        }
    }

    /// Returns the error message contained in the map, if any.
    #[inline]
    pub fn error(&'_ self) -> Option<Cow<'_, str>> {
        let error_message = unsafe { API::get_cached().map_get_error(self) };
        if error_message.is_null() {
            return None;
        }

        let error_message = unsafe { CStr::from_ptr(error_message) };
        Some(error_message.to_string_lossy())
    }

    /// Adds an error message to a map. The map is cleared first.
    #[inline]
    pub fn set_error(&mut self, error_message: &str) -> MapResult<()> {
        let error_message = CString::new(error_message)?;
        unsafe {
            API::get_cached().map_set_error(self, error_message.as_ptr());
        }
        Ok(())
    }

    /// Returns the number of keys contained in a map.
    #[inline]
    pub fn key_count(&self) -> usize {
        let count = unsafe { API::get_cached().map_num_keys(self) };
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

        unsafe { CStr::from_ptr(API::get_cached().map_get_key(self, index)) }
    }

    /// Returns a key from a map.
    ///
    /// # Panics
    /// Panics if `index >= self.key_count()`.
    #[inline]
    pub fn key(&self, index: usize) -> &str {
        self.key_raw(index).to_str().unwrap()
    }

    /// Returns an iterator over all keys in a map.
    #[inline]
    pub fn keys(&'_ self) -> Keys<'_, 'elem> {
        Keys::new(self)
    }

    /// Returns the number of elements associated with a key in a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn value_count_raw_unchecked(&self, key: &CStr) -> MapResult<usize> {
        let rv = API::get_cached().map_num_elements(self, key.as_ptr());
        if rv == -1 {
            Err(MapError::KeyNotFound)
        } else {
            debug_assert!(rv >= 0);
            Ok(rv as usize)
        }
    }

    /// Returns the number of elements associated with a key in a map.
    #[inline]
    pub fn value_count(&self, key: &str) -> MapResult<usize> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.value_count_raw_unchecked(&key) }
    }

    /// Retrieves a value type from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn value_type_raw_unchecked(&self, key: &CStr) -> MapResult<ValueType> {
        match API::get_cached().map_get_type(self, key.as_ptr()) {
            x if x == ffi::VSPropertyType::ptUnset as c_int => Err(MapError::KeyNotFound),
            x if x == ffi::VSPropertyType::ptInt as c_int => Ok(ValueType::Int),
            x if x == ffi::VSPropertyType::ptFloat as c_int => Ok(ValueType::Float),
            x if x == ffi::VSPropertyType::ptData as c_int => Ok(ValueType::Data),
            x if x
                == ffi::VSPropertyType::ptVideoNode as c_int
                    | ffi::VSPropertyType::ptAudioNode as c_int =>
            {
                Ok(ValueType::Node)
            }
            x if x
                == ffi::VSPropertyType::ptVideoFrame as c_int
                    | ffi::VSPropertyType::ptAudioFrame as c_int =>
            {
                Ok(ValueType::Frame)
            }
            x if x == ffi::VSPropertyType::ptFunction as c_int => Ok(ValueType::Function),
            _ => unreachable!(),
        }
    }

    /// Retrieves a value type from a map.
    #[inline]
    pub fn value_type(&self, key: &str) -> MapResult<ValueType> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.value_type_raw_unchecked(&key) }
    }

    /// Deletes the given key.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn delete_key_raw_unchecked(&mut self, key: &CStr) -> MapResult<()> {
        let result = API::get_cached().map_delete_key(self, key.as_ptr());
        if result == 0 {
            Err(MapError::KeyNotFound)
        } else {
            debug_assert!(result == 1);
            Ok(())
        }
    }

    /// Deletes the given key.
    #[inline]
    pub fn delete_key(&mut self, key: &str) -> MapResult<()> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.delete_key_raw_unchecked(&key) }
    }

    /// Retrieves a property value.
    #[inline]
    pub fn get<'map, T: Value<'map, 'elem> + Sized>(&'map self, key: &str) -> MapResult<T> {
        T::get_from_map(self, key)
    }

    /// Retrieves an iterator over the map values.
    #[inline]
    pub fn get_iter<'map, T: ValueNotArray<'map, 'elem>>(
        &'map self,
        key: &str,
    ) -> MapResult<ValueIter<'map, 'elem, T>> {
        T::get_iter_from_map(self, key)
    }

    /// Sets a property value.
    #[inline]
    pub fn set<'map, T: Value<'map, 'elem>>(&'map mut self, key: &str, x: &T) -> MapResult<()> {
        T::store_in_map(self, key, x)
    }

    /// Appends a property value.
    #[inline]
    pub fn append<'map, T: ValueNotArray<'map, 'elem>>(
        &'map mut self,
        key: &str,
        x: &T,
    ) -> MapResult<()> {
        T::append_to_map(self, key, x)
    }

    /// Retrieves an integer from a map.
    ///
    /// This function retrieves the first value associated with the key.
    #[inline]
    pub fn get_int(&self, key: &str) -> MapResult<i64> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.get_int_raw_unchecked(&key, 0) }
    }

    /// Retrieves integers from a map.
    #[inline]
    pub fn get_int_iter<'map>(&'map self, key: &str) -> MapResult<ValueIter<'map, 'elem, i64>> {
        let key = Map::make_raw_key(key)?;
        unsafe { ValueIter::new_int(self, key) }
    }

    /// Retrieves a floating point number from a map.
    ///
    /// This function retrieves the first value associated with the key.
    #[inline]
    pub fn get_float(&self, key: &str) -> MapResult<f64> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.get_float_raw_unchecked(&key, 0) }
    }

    /// Retrieves floating point numbers from a map.
    #[inline]
    pub fn get_float_iter<'map>(&'map self, key: &str) -> MapResult<ValueIter<'map, 'elem, f64>> {
        let key = Map::make_raw_key(key)?;
        unsafe { ValueIter::new_float(self, key) }
    }

    /// Retrieves data from a map.
    ///
    /// This function retrieves the first value associated with the key.
    #[inline]
    pub fn get_data(&self, key: &str) -> MapResult<Data<'elem>> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.get_data_raw_unchecked(&key, 0) }
    }

    /// Retrieves data from a map.
    #[inline]
    pub fn get_data_iter<'map>(
        &'map self,
        key: &str,
    ) -> MapResult<ValueIter<'map, 'elem, Data<'elem>>> {
        let key = Map::make_raw_key(key)?;
        unsafe { ValueIter::new_data(self, key) }
    }

    /// Retrieves data from a map.
    #[inline]
    pub fn get_string_iter<'map>(
        &'map self,
        key: &str,
    ) -> MapResult<ValueIter<'map, 'elem, String>> {
        let key = Map::make_raw_key(key)?;
        unsafe { ValueIter::new_string(self, key) }
    }

    #[inline]
    pub fn get_string(&self, key: &str) -> MapResult<String> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.get_string_raw_unchecked(&key, 0) }
    }

    /// Retrieves data from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn get_string_raw_unchecked(
        &self,
        key: &CStr,
        index: i32,
    ) -> MapResult<String> {
        let mut error = 0;
        let value = API::get_cached().map_get_data(self, key.as_ptr(), index, &mut error);
        handle_get_prop_error(error)?;

        let mut error = 0;
        let length = API::get_cached().map_get_data_size(self, key.as_ptr(), index, &mut error);
        debug_assert!(error == 0);
        debug_assert!(length >= 0);

        let slice = slice::from_raw_parts(value as *const u8, length as usize);

        Ok(String::from_utf8(slice.to_vec()).unwrap())
    }

    /// Retrieves a node from a map.
    ///
    /// This function retrieves the first value associated with the key.
    #[inline]
    pub fn get_node(&self, key: &str) -> MapResult<Node> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.get_node_raw_unchecked(&key, 0) }
    }

    /// Retrieves nodes from a map.
    #[inline]
    pub fn get_node_iter<'map>(&'map self, key: &str) -> MapResult<ValueIter<'map, 'elem, Node>> {
        let key = Map::make_raw_key(key)?;
        unsafe { ValueIter::new_node(self, key) }
    }

    /// Retrieves a frame from a map.
    ///
    /// This function retrieves the first value associated with the key.
    #[inline]
    pub fn get_frame(&self, key: &str) -> MapResult<Frame<'elem>> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.get_frame_raw_unchecked(&key, 0) }
    }

    /// Retrieves frames from a map.
    #[inline]
    pub fn get_frame_iter<'map>(
        &'map self,
        key: &str,
    ) -> MapResult<ValueIter<'map, 'elem, Frame<'elem>>> {
        let key = Map::make_raw_key(key)?;
        unsafe { ValueIter::new_frame(self, key) }
    }

    /// Retrieves a function from a map.
    ///
    /// This function retrieves the first value associated with the key.
    #[inline]
    pub fn get_function(&self, key: &str) -> MapResult<Function<'elem>> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.get_function_raw_unchecked(&key, 0) }
    }

    /// Retrieves functions from a map.
    #[inline]
    pub fn get_function_iter<'map>(
        &'map self,
        key: &str,
    ) -> MapResult<ValueIter<'map, 'elem, Function<'elem>>> {
        let key = Map::make_raw_key(key)?;
        unsafe { ValueIter::new_function(self, key) }
    }

    /// Retrieves int array from a map.
    #[inline]
    pub fn get_int_array(&self, key: &str) -> MapResult<Vec<i64>> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.get_int_array_raw_unchecked(&key) }
    }

    /// Retrieves float array from a map.
    #[inline]
    pub fn get_float_array(&self, key: &str) -> MapResult<Vec<f64>> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.get_float_array_raw_unchecked(&key) }
    }

    /// Retrieves an integer from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn get_int_raw_unchecked(&self, key: &CStr, index: i32) -> MapResult<i64> {
        let mut error = 0;
        let value = API::get_cached().map_get_int(self, key.as_ptr(), index, &mut error);
        handle_get_prop_error(error)?;

        Ok(value)
    }

    /// Retrieves a floating point number from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn get_float_raw_unchecked(&self, key: &CStr, index: i32) -> MapResult<f64> {
        let mut error = 0;
        let value = API::get_cached().map_get_float(self, key.as_ptr(), index, &mut error);
        handle_get_prop_error(error)?;

        Ok(value)
    }

    /// Retrieves data from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn get_data_raw_unchecked(
        &self,
        key: &CStr,
        index: i32,
    ) -> MapResult<Data<'elem>> {
        let mut error = 0;
        let value = API::get_cached().map_get_data(self, key.as_ptr(), index, &mut error);
        handle_get_prop_error(error)?;

        let mut error = 0;
        let length = API::get_cached().map_get_data_size(self, key.as_ptr(), index, &mut error);
        debug_assert!(error == 0);
        debug_assert!(length >= 0);

        let slice = slice::from_raw_parts(value as *const u8, length as usize);

        Ok(Data::from_slice(slice))
    }

    pub fn data_type_hint(&self, key: &CStr, index: i32) -> DataType {
        let hint = unsafe {
            API::get_cached().map_get_data_type_hint(self.handle.as_ptr(), key.as_ptr(), index)
        };
        handle_data_hint(hint)
    }

    /// Retrieves a node from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn get_node_raw_unchecked(&self, key: &CStr, index: i32) -> MapResult<Node> {
        let mut error = 0;
        let value = API::get_cached().map_get_node(self, key.as_ptr(), index, &mut error);
        handle_get_prop_error(error)?;

        Ok(Node::from_ptr(value))
    }

    #[inline]
    pub(crate) unsafe fn get_int_array_raw_unchecked(&self, key: &CStr) -> MapResult<Vec<i64>> {
        let mut error = 0;
        let value = API::get_cached().map_get_int_array(self, key.as_ptr(), &mut error);
        handle_get_prop_error(error)?;

        Ok(slice::from_raw_parts(value, self.value_count_raw_unchecked(key).unwrap()).to_vec())
    }

    #[inline]
    pub(crate) unsafe fn get_float_array_raw_unchecked(&self, key: &CStr) -> MapResult<Vec<f64>> {
        let mut error = 0;
        let value = API::get_cached().map_get_float_array(self, key.as_ptr(), &mut error);
        handle_get_prop_error(error)?;

        Ok(slice::from_raw_parts(value, self.value_count_raw_unchecked(key).unwrap()).to_vec())
    }

    /// Retrieves a frame from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn get_frame_raw_unchecked(
        &self,
        key: &CStr,
        index: i32,
    ) -> MapResult<Frame<'elem>> {
        let mut error = 0;
        let value = API::get_cached().map_get_frame(self, key.as_ptr(), index, &mut error);
        handle_get_prop_error(error)?;

        Ok(Frame::from_ptr(value))
    }

    /// Retrieves a function from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn get_function_raw_unchecked(
        &self,
        key: &CStr,
        index: i32,
    ) -> MapResult<Function<'elem>> {
        let mut error = 0;
        let value = API::get_cached().map_get_func(self, key.as_ptr(), index, &mut error);
        handle_get_prop_error(error)?;

        Ok(Function::from_ptr(value))
    }

    /// Appends an integer to a map.
    #[inline]
    pub fn append_int(&mut self, key: &str, x: i64) -> MapResult<()> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.append_int_raw_unchecked(&key, x) }
    }

    /// Appends a floating point number to a map.
    #[inline]
    pub fn append_float(&mut self, key: &str, x: f64) -> MapResult<()> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.append_float_raw_unchecked(&key, x) }
    }

    /// Appends data to a map.
    #[inline]
    pub fn append_data(&mut self, key: &str, x: &[u8]) -> MapResult<()> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.append_data_raw_unchecked(&key, x) }
    }

    /// Appends a node to a map.
    #[inline]
    pub fn append_node(&mut self, key: &str, x: &Node) -> MapResult<()> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.append_node_raw_unchecked(&key, x) }
    }

    /// Appends a frame to a map.
    #[inline]
    pub fn append_frame(&mut self, key: &str, x: &Frame<'elem>) -> MapResult<()> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.append_frame_raw_unchecked(&key, x) }
    }

    /// Appends a function to a map.
    #[inline]
    pub fn append_function(&mut self, key: &str, x: &Function<'elem>) -> MapResult<()> {
        let key = Map::make_raw_key(key)?;
        unsafe { self.append_function_raw_unchecked(&key, x) }
    }

    /// Appends an integer to a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn append_int_raw_unchecked(&mut self, key: &CStr, x: i64) -> MapResult<()> {
        let error =
            API::get_cached().map_set_int(self, key.as_ptr(), x, ffi::VSMapAppendMode::maAppend);

        handle_append_prop_error(error)
    }

    /// Appends a floating point number to a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn append_float_raw_unchecked(
        &mut self,
        key: &CStr,
        x: f64,
    ) -> MapResult<()> {
        let error =
            API::get_cached().map_set_float(self, key.as_ptr(), x, ffi::VSMapAppendMode::maAppend);

        handle_append_prop_error(error)
    }

    /// Appends data to a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn append_data_raw_unchecked(
        &mut self,
        key: &CStr,
        x: &[u8],
    ) -> MapResult<()> {
        let error = API::get_cached().map_set_data(
            self,
            key.as_ptr(),
            x,
            ffi::VSDataTypeHint::dtUnknown,
            ffi::VSMapAppendMode::maAppend,
        );

        handle_append_prop_error(error)
    }

    /// Appends a node to a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn append_node_raw_unchecked(
        &mut self,
        key: &CStr,
        x: &Node,
    ) -> MapResult<()> {
        let error = API::get_cached().map_set_node(
            self,
            key.as_ptr(),
            x.as_ptr(),
            ffi::VSMapAppendMode::maAppend,
        );

        handle_append_prop_error(error)
    }

    /// Appends a frame to a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn append_frame_raw_unchecked(
        &mut self,
        key: &CStr,
        x: &Frame<'elem>,
    ) -> MapResult<()> {
        let error = API::get_cached().map_set_frame(
            self,
            key.as_ptr(),
            x.deref(),
            ffi::VSMapAppendMode::maAppend,
        );

        handle_append_prop_error(error)
    }

    /// Appends a function to a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn append_function_raw_unchecked(
        &mut self,
        key: &CStr,
        x: &Function<'elem>,
    ) -> MapResult<()> {
        let error = API::get_cached().map_set_func(
            self,
            key.as_ptr(),
            x.as_ptr(),
            ffi::VSMapAppendMode::maAppend,
        );

        handle_append_prop_error(error)
    }

    /// Sets a property value to an integer.
    #[inline]
    pub fn set_int(&mut self, key: &str, x: i64) -> MapResult<()> {
        let key = Map::make_raw_key(key)?;
        unsafe {
            self.set_int_raw_unchecked(&key, x);
        }
        Ok(())
    }

    pub fn set_string(&mut self, key: &str, x: &str) -> MapResult<()> {
        let key = Map::make_raw_key(key)?;
        unsafe {
            self.set_string_raw_unchecked(&key, x);
        }
        Ok(())
    }

    /// Sets a property value to a floating point number.
    #[inline]
    pub fn set_float(&mut self, key: &str, x: f64) -> MapResult<()> {
        let key = Map::make_raw_key(key)?;
        unsafe {
            self.set_float_raw_unchecked(&key, x);
        }
        Ok(())
    }

    /// Sets a property value to data.
    #[inline]
    pub fn set_data(&mut self, key: &str, x: &[u8]) -> MapResult<()> {
        let key = Map::make_raw_key(key)?;
        unsafe {
            self.set_data_raw_unchecked(&key, x);
        }
        Ok(())
    }

    /// Sets a property value to a node.
    #[inline]
    pub fn set_node(&mut self, key: &str, x: &Node) -> MapResult<()> {
        let key = Map::make_raw_key(key)?;
        unsafe {
            self.set_node_raw_unchecked(&key, x);
        }
        Ok(())
    }

    /// Sets a property value to a frame.
    #[inline]
    pub fn set_frame(&mut self, key: &str, x: &Frame<'elem>) -> MapResult<()> {
        let key = Map::make_raw_key(key)?;
        unsafe {
            self.set_frame_raw_unchecked(&key, x);
        }
        Ok(())
    }

    /// Sets a property value to a function.
    #[inline]
    pub fn set_function(&mut self, key: &str, x: &Function<'elem>) -> MapResult<()> {
        let key = Map::make_raw_key(key)?;
        unsafe {
            self.set_function_raw_unchecked(&key, x);
        }
        Ok(())
    }

    pub fn set_int_array(&mut self, key: &str, x: Vec<i64>) -> MapResult<()> {
        let key = Map::make_raw_key(key)?;
        unsafe {
            self.set_int_array_raw_unchecked(&key, x);
        }
        Ok(())
    }

    pub fn set_float_array(&mut self, key: &str, x: Vec<f64>) -> MapResult<()> {
        let key = Map::make_raw_key(key)?;
        unsafe {
            self.set_float_array_raw_unchecked(&key, x);
        }
        Ok(())
    }

    #[inline]
    pub(crate) unsafe fn set_int_array_raw_unchecked(&mut self, key: &CStr, x: Vec<i64>) {
        let error = API::get_cached().map_set_int_array(
            self.handle.as_mut(),
            key.as_ptr(),
            x.as_ptr(),
            x.len().try_into().unwrap(),
        );

        debug_assert!(error == 0);
    }

    #[inline]
    pub(crate) unsafe fn set_float_array_raw_unchecked(&mut self, key: &CStr, x: Vec<f64>) {
        let error = API::get_cached().map_set_float_array(
            self.handle.as_mut(),
            key.as_ptr(),
            x.as_ptr(),
            x.len().try_into().unwrap(),
        );

        debug_assert!(error == 0);
    }

    /// Sets a property value to an integer.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn set_int_raw_unchecked(&mut self, key: &CStr, x: i64) {
        let error =
            API::get_cached().map_set_int(self, key.as_ptr(), x, ffi::VSMapAppendMode::maReplace);

        debug_assert!(error == 0);
    }

    /// Sets a property value to a floating point number.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn set_float_raw_unchecked(&mut self, key: &CStr, x: f64) {
        let error =
            API::get_cached().map_set_float(self, key.as_ptr(), x, ffi::VSMapAppendMode::maReplace);

        debug_assert!(error == 0);
    }

    /// Sets a property value to data.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn set_data_raw_unchecked(&mut self, key: &CStr, x: &[u8]) {
        let error = API::get_cached().map_set_data(
            self,
            key.as_ptr(),
            x,
            ffi::VSDataTypeHint::dtUnknown,
            ffi::VSMapAppendMode::maReplace,
        );

        debug_assert!(error == 0);
    }

    /// Sets a property value to data.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn set_string_raw_unchecked(&mut self, key: &CStr, x: &str) {
        let error = API::get_cached().map_set_data(
            self,
            key.as_ptr(),
            x.as_bytes(),
            ffi::VSDataTypeHint::dtUtf8,
            ffi::VSMapAppendMode::maReplace,
        );

        debug_assert!(error == 0);
    }

    /// Sets a property value to a node.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn set_node_raw_unchecked(&mut self, key: &CStr, x: &Node) {
        let error = API::get_cached().map_set_node(
            self,
            key.as_ptr(),
            x.as_ptr(),
            ffi::VSMapAppendMode::maReplace,
        );

        debug_assert!(error == 0);
    }

    /// Sets a property value to a frame.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn set_frame_raw_unchecked(&mut self, key: &CStr, x: &Frame<'elem>) {
        let error = API::get_cached().map_set_frame(
            self,
            key.as_ptr(),
            x.deref(),
            ffi::VSMapAppendMode::maReplace,
        );

        debug_assert!(error == 0);
    }

    /// Sets a property value to a function.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn set_function_raw_unchecked(&mut self, key: &CStr, x: &Function<'elem>) {
        let error = API::get_cached().map_set_func(
            self,
            key.as_ptr(),
            x.as_ptr(),
            ffi::VSMapAppendMode::maReplace,
        );

        debug_assert!(error == 0);
    }

    fn consume_frame(
        self,
        frame: Frame<'elem>,
        key: &str,
        append: ffi::VSMapAppendMode,
    ) -> MapResult<()> {
        let key = Map::make_raw_key(key)?;
        let frame = mem::ManuallyDrop::new(frame);
        let frame_ptr = frame.as_ptr();
        let res = unsafe {
            API::get_cached().map_consume_frame(
                self.handle.as_ptr(),
                key.as_ptr(),
                frame_ptr,
                append as i32,
            )
        };
        if res == 0 {
            Ok(())
        } else {
            Err(MapError::Error)
        }
    }

    /// Consumes a frame and appends or sets it in the map.
    #[inline]
    pub fn append_consume_frame(self, frame: Frame<'elem>, key: &str) -> MapResult<()> {
        self.consume_frame(frame, key, ffi::VSMapAppendMode::maAppend)
    }

    /// Consumes a frame and sets it in the map. Replaces any existing values.
    #[inline]
    pub fn set_consume_frame(self, frame: Frame<'elem>, key: &str) -> MapResult<()> {
        self.consume_frame(frame, key, ffi::VSMapAppendMode::maReplace)
    }

    fn consume_node(self, node: Node, key: &str, append: ffi::VSMapAppendMode) -> MapResult<()> {
        let key = Map::make_raw_key(key)?;
        let node = mem::ManuallyDrop::new(node);
        let node_ptr = node.as_ptr();
        let res = unsafe {
            API::get_cached().map_consume_node(
                self.handle.as_ptr(),
                key.as_ptr(),
                node_ptr,
                append as i32,
            )
        };
        if res == 0 {
            Ok(())
        } else {
            Err(MapError::Error)
        }
    }

    /// Consumes a node and appends or sets it in the map.
    #[inline]
    pub fn append_consume_node(self, node: Node, key: &str) -> MapResult<()> {
        self.consume_node(node, key, ffi::VSMapAppendMode::maAppend)
    }

    /// Consumes a node and sets it in the map. Replaces any existing values.
    #[inline]
    pub fn set_consume_node(self, node: Node, key: &str) -> MapResult<()> {
        self.consume_node(node, key, ffi::VSMapAppendMode::maReplace)
    }

    fn consume_function(
        self,
        func: Function<'elem>,
        key: &str,
        append: ffi::VSMapAppendMode,
    ) -> MapResult<()> {
        let key = Map::make_raw_key(key)?;
        let func = mem::ManuallyDrop::new(func);
        let func_ptr = func.as_ptr();
        let res = unsafe {
            API::get_cached().map_consume_function(
                self.handle.as_ptr(),
                key.as_ptr(),
                func_ptr,
                append as i32,
            )
        };
        if res == 0 {
            Ok(())
        } else {
            Err(MapError::Error)
        }
    }

    /// Consumes a function and appends or sets it in the map.
    #[inline]
    pub fn append_consume_function(self, func: Function<'elem>, key: &str) -> MapResult<()> {
        self.consume_function(func, key, ffi::VSMapAppendMode::maAppend)
    }

    /// Consumes a function and sets it in the map. Replaces any existing values.
    #[inline]
    pub fn set_consume_function(self, func: Function<'elem>, key: &str) -> MapResult<()> {
        self.consume_function(func, key, ffi::VSMapAppendMode::maReplace)
    }

    // TODO: Saturated retrival
}

pub trait IntoOwnedMap {
    fn into_owned_map<'elem>(self) -> OwnedMap<'elem>;
}
