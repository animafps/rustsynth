//! `VapourSynth` maps.

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
pub use self::data::{Data, DataType};

#[cfg(test)]
mod tests;

/// An owned `VapourSynth` map that frees on drop.
///
/// A map contains key-value pairs where the value is zero or more elements of a certain type.
#[derive(Debug)]
pub struct Map<'elem> {
    handle: NonNull<ffi::VSMap>,
    _elem: PhantomData<&'elem ()>,
}

unsafe impl Send for Map<'_> {}
unsafe impl Sync for Map<'_> {}

/// A borrowed reference to a `VapourSynth` map that does not free on drop.
///
/// This type is used when the map is owned by another object (like frame properties).
/// It can only be accessed through references (`&MapRef` or `&mut MapRef`).
#[derive(Debug)]
#[repr(transparent)]
pub struct MapRef<'elem> {
    inner: ffi::VSMap,
    _elem: PhantomData<&'elem ()>,
}

unsafe impl Send for MapRef<'_> {}
unsafe impl Sync for MapRef<'_> {}

// Map derefs to MapRef so they share the same API
impl<'elem> Deref for Map<'elem> {
    type Target = MapRef<'elem>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe {
            // SAFETY: MapRef is repr(transparent) over ffi::VSMap
            MapRef::from_ptr(self.handle.as_ptr())
        }
    }
}

impl<'elem> DerefMut for Map<'elem> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            // SAFETY: MapRef is repr(transparent) over ffi::VSMap
            MapRef::from_ptr_mut(self.handle.as_ptr())
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

impl<'elem> Map<'elem> {
    /// Creates a new owned map.
    pub fn new() -> MapResult<Self> {
        let handle = API::get().ok_or(MapError::CreationFailed)?.create_map();
        let handle = NonNull::new(handle).ok_or(MapError::CreationFailed)?;

        Ok(Self {
            handle,
            _elem: PhantomData,
        })
    }

    /// Wraps a raw pointer into an owned `Map`.
    ///
    /// # Safety
    /// The caller must ensure:
    /// - The pointer is valid and points to a VSMap
    /// - The map is not already owned by another Rust object
    /// - The element lifetime `'elem` is valid
    #[inline]
    pub const unsafe fn from_ptr(handle: *mut ffi::VSMap) -> Self {
        Self {
            handle: NonNull::new_unchecked(handle),
            _elem: PhantomData,
        }
    }

    /// Returns the raw pointer to the map.
    #[must_use]
    pub const fn as_ptr(&self) -> *mut ffi::VSMap {
        self.handle.as_ptr()
    }
}

impl<'elem> MapRef<'elem> {
    /// Wraps a raw pointer into a borrowed `MapRef`.
    ///
    /// # Safety
    /// The caller must ensure:
    /// - The pointer is valid and points to a VSMap
    /// - The lifetime `'a` is valid for the duration of the borrow
    /// - The element lifetime `'elem` is valid
    #[inline]
    pub const unsafe fn from_ptr<'a>(ptr: *const ffi::VSMap) -> &'a Self {
        &*(ptr as *const MapRef<'elem>)
    }

    /// Wraps a raw mutable pointer into a borrowed `MapRef`.
    ///
    /// # Safety
    /// Same as `from_ptr`, plus no other mutable references exist
    #[inline]
    pub unsafe fn from_ptr_mut<'a>(ptr: *mut ffi::VSMap) -> &'a mut Self {
        &mut *(ptr as *mut MapRef<'elem>)
    }

    /// Returns the raw pointer to the map.
    #[must_use]
    pub const fn as_ptr(&self) -> *const ffi::VSMap {
        self as *const Self as *const ffi::VSMap
    }

    /// Returns the raw mutable pointer to the map.
    #[must_use]
    pub const fn as_mut_ptr(&mut self) -> *mut ffi::VSMap {
        self as *mut Self as *mut ffi::VSMap
    }
}

// All shared methods go on MapRef so both Map and &MapRef can use them
impl<'elem> MapRef<'elem> {
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
        MapRef::is_key_valid(key)?;
        Ok(CString::new(key)?)
    }

    /// Clears the map.
    #[inline]
    pub fn clear(&mut self) {
        unsafe {
            API::get_cached().clear_map(&mut self.inner);
        }
    }

    /// Returns the error message contained in the map, if any.
    #[inline]
    pub fn error(&'_ self) -> Result<&str, std::str::Utf8Error> {
        let error_message = unsafe { API::get_cached().map_get_error(&self.inner) };
        let error_message = unsafe { CStr::from_ptr(error_message) };
        error_message.to_str()
    }

    /// Adds an error message to a map. The map is cleared first.
    #[inline]
    pub fn set_error(&mut self, error_message: &str) -> MapResult<()> {
        let error_message = CString::new(error_message)?;
        unsafe {
            API::get_cached().map_set_error(&mut self.inner, error_message.as_ptr());
        }
        Ok(())
    }

    /// Returns the number of keys contained in a map.
    #[inline]
    #[must_use]
    pub fn key_count(&self) -> usize {
        let count = unsafe { API::get_cached().map_num_keys(&self.inner) };
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

        unsafe { CStr::from_ptr(API::get_cached().map_get_key(&self.inner, index)) }
    }

    /// Returns a key from a map.
    ///
    /// # Panics
    /// Panics if `index >= self.key_count()`.
    #[inline]
    pub fn key(&self, index: usize) -> Result<&str, std::str::Utf8Error> {
        self.key_raw(index).to_str()
    }

    /// Returns an iterator over all keys in a map.
    #[inline]
    #[must_use]
    pub fn keys(&'_ self) -> Keys<'_, 'elem> {
        Keys::new(self)
    }

    /// Returns the number of elements associated with a key in a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn value_count_raw_unchecked(&self, key: &CStr) -> MapResult<usize> {
        let rv = API::get_cached().map_num_elements(&self.inner, key.as_ptr());
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
        let key = MapRef::make_raw_key(key)?;
        unsafe { self.value_count_raw_unchecked(&key) }
    }

    /// Retrieves a value type from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn value_type_raw_unchecked(&self, key: &CStr) -> MapResult<ValueType> {
        match API::get_cached().map_get_type(&self.inner, key.as_ptr()) {
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
        let key = MapRef::make_raw_key(key)?;
        unsafe { self.value_type_raw_unchecked(&key) }
    }

    /// Deletes the given key.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn delete_key_raw_unchecked(&mut self, key: &CStr) -> MapResult<()> {
        let result = API::get_cached().map_delete_key(&mut self.inner, key.as_ptr());
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
        let key = MapRef::make_raw_key(key)?;
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
        let key = MapRef::make_raw_key(key)?;
        unsafe { self.get_int_raw_unchecked(&key, 0) }
    }

    /// Retrieves integers from a map.
    #[inline]
    pub fn get_int_iter<'map>(&'map self, key: &str) -> MapResult<ValueIter<'map, 'elem, i64>> {
        let key = MapRef::make_raw_key(key)?;
        unsafe { ValueIter::new_int(self, key) }
    }

    /// Retrieves a floating point number from a map.
    ///
    /// This function retrieves the first value associated with the key.
    #[inline]
    pub fn get_float(&self, key: &str) -> MapResult<f64> {
        let key = MapRef::make_raw_key(key)?;
        unsafe { self.get_float_raw_unchecked(&key, 0) }
    }

    /// Retrieves floating point numbers from a map.
    #[inline]
    pub fn get_float_iter<'map>(&'map self, key: &str) -> MapResult<ValueIter<'map, 'elem, f64>> {
        let key = MapRef::make_raw_key(key)?;
        unsafe { ValueIter::new_float(self, key) }
    }

    /// Retrieves data from a map.
    ///
    /// This function retrieves the first value associated with the key.
    #[inline]
    pub fn get_data(&self, key: &str) -> MapResult<Data<'elem>> {
        let key = MapRef::make_raw_key(key)?;
        unsafe { self.get_data_raw_unchecked(&key, 0) }
    }

    /// Retrieves data from a map.
    #[inline]
    pub fn get_data_iter<'map>(
        &'map self,
        key: &str,
    ) -> MapResult<ValueIter<'map, 'elem, Data<'elem>>> {
        let key = MapRef::make_raw_key(key)?;
        unsafe { ValueIter::new_data(self, key) }
    }

    /// Retrieves data from a map.
    #[inline]
    pub fn get_string_iter<'map>(
        &'map self,
        key: &str,
    ) -> MapResult<ValueIter<'map, 'elem, String>> {
        let key = MapRef::make_raw_key(key)?;
        unsafe { ValueIter::new_string(self, key) }
    }

    #[inline]
    pub fn get_string(&self, key: &str) -> MapResult<String> {
        let key = MapRef::make_raw_key(key)?;
        unsafe { Ok(self.get_string_raw_unchecked(&key, 0)?.into_owned()) }
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
    ) -> MapResult<Cow<'elem, str>> {
        let mut error = 0;
        let value = API::get_cached().map_get_data(&self.inner, key.as_ptr(), index, &mut error);
        handle_get_prop_error(error)?;

        let mut error = 0;
        let length =
            API::get_cached().map_get_data_size(&self.inner, key.as_ptr(), index, &mut error);
        debug_assert!(error == 0);
        debug_assert!(length >= 0);

        Ok(CStr::from_ptr(value).to_string_lossy())
    }

    /// Retrieves a node from a map.
    ///
    /// This function retrieves the first value associated with the key.
    #[inline]
    pub fn get_node(&self, key: &str) -> MapResult<Node<'elem>> {
        let key = MapRef::make_raw_key(key)?;
        unsafe { self.get_node_raw_unchecked(&key, 0) }
    }

    /// Retrieves nodes from a map.
    #[inline]
    pub fn get_node_iter<'map>(
        &'map self,
        key: &str,
    ) -> MapResult<ValueIter<'map, 'elem, Node<'elem>>> {
        let key = MapRef::make_raw_key(key)?;
        unsafe { ValueIter::new_node(self, key) }
    }

    /// Retrieves a frame from a map.
    ///
    /// This function retrieves the first value associated with the key.
    #[inline]
    pub fn get_frame(&self, key: &str) -> MapResult<Frame<'elem>> {
        let key = MapRef::make_raw_key(key)?;
        unsafe { self.get_frame_raw_unchecked(&key, 0) }
    }

    /// Retrieves frames from a map.
    #[inline]
    pub fn get_frame_iter<'map>(
        &'map self,
        key: &str,
    ) -> MapResult<ValueIter<'map, 'elem, Frame<'elem>>> {
        let key = MapRef::make_raw_key(key)?;
        unsafe { ValueIter::new_frame(self, key) }
    }

    /// Retrieves a function from a map.
    ///
    /// This function retrieves the first value associated with the key.
    #[inline]
    pub fn get_function(&self, key: &str) -> MapResult<Function<'elem>> {
        let key = MapRef::make_raw_key(key)?;
        unsafe { self.get_function_raw_unchecked(&key, 0) }
    }

    /// Retrieves functions from a map.
    #[inline]
    pub fn get_function_iter<'map>(
        &'map self,
        key: &str,
    ) -> MapResult<ValueIter<'map, 'elem, Function<'elem>>> {
        let key = MapRef::make_raw_key(key)?;
        unsafe { ValueIter::new_function(self, key) }
    }

    /// Retrieves int array from a map.
    #[inline]
    pub fn get_int_array(&self, key: &str) -> MapResult<&[i64]> {
        let key = MapRef::make_raw_key(key)?;
        unsafe { self.get_int_array_raw_unchecked(&key) }
    }

    /// Retrieves float array from a map.
    #[inline]
    pub fn get_float_array(&self, key: &str) -> MapResult<&[f64]> {
        let key = MapRef::make_raw_key(key)?;
        unsafe { self.get_float_array_raw_unchecked(&key) }
    }

    /// Retrieves an integer from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn get_int_raw_unchecked(&self, key: &CStr, index: i32) -> MapResult<i64> {
        let mut error = 0;
        let value = API::get_cached().map_get_int(&self.inner, key.as_ptr(), index, &mut error);
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
        let value = API::get_cached().map_get_float(&self.inner, key.as_ptr(), index, &mut error);
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
        let value = API::get_cached().map_get_data(&self.inner, key.as_ptr(), index, &mut error);
        handle_get_prop_error(error)?;

        let mut error = 0;
        let length =
            API::get_cached().map_get_data_size(&self.inner, key.as_ptr(), index, &mut error);
        debug_assert!(error == 0);
        debug_assert!(length >= 0);

        let slice = slice::from_raw_parts(value.cast::<u8>(), length as usize);

        Ok(Data::from_slice(slice))
    }

    #[must_use]
    pub fn data_type_hint(&self, key: &CStr, index: i32) -> DataType {
        let hint = unsafe {
            API::get_cached().map_get_data_type_hint(
                &self.inner as *const _ as *mut _,
                key.as_ptr(),
                index,
            )
        };
        DataType::from_hint(hint)
    }

    /// Retrieves a node from a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn get_node_raw_unchecked(
        &self,
        key: &CStr,
        index: i32,
    ) -> MapResult<Node<'elem>> {
        let mut error = 0;
        let value = API::get_cached().map_get_node(&self.inner, key.as_ptr(), index, &mut error);
        handle_get_prop_error(error)?;

        Ok(Node::from_ptr(value))
    }

    #[inline]
    pub(crate) unsafe fn get_int_array_raw_unchecked(&self, key: &CStr) -> MapResult<&[i64]> {
        let mut error = 0;
        let value = API::get_cached().map_get_int_array(&self.inner, key.as_ptr(), &mut error);
        handle_get_prop_error(error)?;

        Ok(slice::from_raw_parts(
            value,
            self.value_count_raw_unchecked(key).unwrap(),
        ))
    }

    #[inline]
    pub(crate) unsafe fn get_float_array_raw_unchecked(&self, key: &CStr) -> MapResult<&[f64]> {
        let mut error = 0;
        let value = API::get_cached().map_get_float_array(&self.inner, key.as_ptr(), &mut error);
        handle_get_prop_error(error)?;

        Ok(slice::from_raw_parts(
            value,
            self.value_count_raw_unchecked(key).unwrap(),
        ))
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
        let value = API::get_cached().map_get_frame(&self.inner, key.as_ptr(), index, &mut error);
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
        let value = API::get_cached().map_get_func(&self.inner, key.as_ptr(), index, &mut error);
        handle_get_prop_error(error)?;

        Ok(Function::from_ptr(value))
    }

    /// Appends an integer to a map.
    #[inline]
    pub fn append_int(&mut self, key: &str, x: i64) -> MapResult<()> {
        let key = MapRef::make_raw_key(key)?;
        unsafe { self.append_int_raw_unchecked(&key, x) }
    }

    /// Appends a floating point number to a map.
    #[inline]
    pub fn append_float(&mut self, key: &str, x: f64) -> MapResult<()> {
        let key = MapRef::make_raw_key(key)?;
        unsafe { self.append_float_raw_unchecked(&key, x) }
    }

    /// Appends data to a map.
    #[inline]
    pub fn append_data(&mut self, key: &str, x: &[u8]) -> MapResult<()> {
        let key = MapRef::make_raw_key(key)?;
        unsafe { self.append_data_raw_unchecked(&key, x) }
    }

    /// Appends a node to a map.
    #[inline]
    pub fn append_node(&mut self, key: &str, x: &Node) -> MapResult<()> {
        let key = MapRef::make_raw_key(key)?;
        unsafe { self.append_node_raw_unchecked(&key, x) }
    }

    /// Appends a frame to a map.
    #[inline]
    pub fn append_frame(&mut self, key: &str, x: &Frame<'elem>) -> MapResult<()> {
        let key = MapRef::make_raw_key(key)?;
        unsafe { self.append_frame_raw_unchecked(&key, x) }
    }

    /// Appends a function to a map.
    #[inline]
    pub fn append_function(&mut self, key: &str, x: &Function<'elem>) -> MapResult<()> {
        let key = MapRef::make_raw_key(key)?;
        unsafe { self.append_function_raw_unchecked(&key, x) }
    }

    /// Appends an integer to a map.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn append_int_raw_unchecked(&mut self, key: &CStr, x: i64) -> MapResult<()> {
        let error = API::get_cached().map_set_int(
            &mut self.inner,
            key.as_ptr(),
            x,
            ffi::VSMapAppendMode::maAppend,
        );

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
        let error = API::get_cached().map_set_float(
            &mut self.inner,
            key.as_ptr(),
            x,
            ffi::VSMapAppendMode::maAppend,
        );

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
            &mut *self.as_mut_ptr(),
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
            &mut *self.as_mut_ptr(),
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
            &mut *self.as_mut_ptr(),
            key.as_ptr(),
            &raw const **x,
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
            &mut *self.as_mut_ptr(),
            key.as_ptr(),
            x.as_ptr(),
            ffi::VSMapAppendMode::maAppend,
        );

        handle_append_prop_error(error)
    }

    /// Sets a property value to an integer.
    #[inline]
    pub fn set_int(&mut self, key: &str, x: i64) -> MapResult<()> {
        let key = MapRef::make_raw_key(key)?;
        unsafe {
            self.set_int_raw_unchecked(&key, x);
        }
        Ok(())
    }

    pub fn set_string(&mut self, key: &str, x: &str) -> MapResult<()> {
        let key = MapRef::make_raw_key(key)?;
        unsafe {
            self.set_string_raw_unchecked(&key, x);
        }
        Ok(())
    }

    /// Sets a property value to a floating point number.
    #[inline]
    pub fn set_float(&mut self, key: &str, x: f64) -> MapResult<()> {
        let key = MapRef::make_raw_key(key)?;
        unsafe {
            self.set_float_raw_unchecked(&key, x);
        }
        Ok(())
    }

    /// Sets a property value to data.
    #[inline]
    pub fn set_data(&mut self, key: &str, x: &[u8]) -> MapResult<()> {
        let key = MapRef::make_raw_key(key)?;
        unsafe {
            self.set_data_raw_unchecked(&key, x);
        }
        Ok(())
    }

    /// Sets a property value to a node.
    #[inline]
    pub fn set_node(&mut self, key: &str, x: &Node) -> MapResult<()> {
        let key = MapRef::make_raw_key(key)?;
        unsafe {
            self.set_node_raw_unchecked(&key, x);
        }
        Ok(())
    }

    /// Sets a property value to a frame.
    #[inline]
    pub fn set_frame(&mut self, key: &str, x: &Frame<'elem>) -> MapResult<()> {
        let key = MapRef::make_raw_key(key)?;
        unsafe {
            self.set_frame_raw_unchecked(&key, x);
        }
        Ok(())
    }

    /// Sets a property value to a function.
    #[inline]
    pub fn set_function(&mut self, key: &str, x: &Function<'elem>) -> MapResult<()> {
        let key = MapRef::make_raw_key(key)?;
        unsafe {
            self.set_function_raw_unchecked(&key, x);
        }
        Ok(())
    }

    pub fn set_int_array(&mut self, key: &str, x: Vec<i64>) -> MapResult<()> {
        let key = MapRef::make_raw_key(key)?;
        unsafe {
            self.set_int_array_raw_unchecked(&key, x);
        }
        Ok(())
    }

    pub fn set_float_array(&mut self, key: &str, x: Vec<f64>) -> MapResult<()> {
        let key = MapRef::make_raw_key(key)?;
        unsafe {
            self.set_float_array_raw_unchecked(&key, x);
        }
        Ok(())
    }

    #[inline]
    pub(crate) unsafe fn set_int_array_raw_unchecked(&mut self, key: &CStr, x: Vec<i64>) {
        let error = API::get_cached().map_set_int_array(
            &mut self.inner,
            key.as_ptr(),
            x.as_ptr(),
            x.len().try_into().unwrap(),
        );

        debug_assert!(error == 0);
    }

    #[inline]
    pub(crate) unsafe fn set_float_array_raw_unchecked(&mut self, key: &CStr, x: Vec<f64>) {
        let error = API::get_cached().map_set_float_array(
            &mut self.inner,
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
        let error = API::get_cached().map_set_int(
            &mut self.inner,
            key.as_ptr(),
            x,
            ffi::VSMapAppendMode::maReplace,
        );

        debug_assert!(error == 0);
    }

    /// Sets a property value to a floating point number.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn set_float_raw_unchecked(&mut self, key: &CStr, x: f64) {
        let error = API::get_cached().map_set_float(
            &mut self.inner,
            key.as_ptr(),
            x,
            ffi::VSMapAppendMode::maReplace,
        );

        debug_assert!(error == 0);
    }

    /// Sets a property value to data.
    ///
    /// # Safety
    /// The caller must ensure `key` is valid.
    #[inline]
    pub(crate) unsafe fn set_data_raw_unchecked(&mut self, key: &CStr, x: &[u8]) {
        let error = API::get_cached().map_set_data(
            &mut *self.as_mut_ptr(),
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
            &mut *self.as_mut_ptr(),
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
            &mut *self.as_mut_ptr(),
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
            &mut *self.as_mut_ptr(),
            key.as_ptr(),
            &raw const **x,
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
            &mut *self.as_mut_ptr(),
            key.as_ptr(),
            x.as_ptr(),
            ffi::VSMapAppendMode::maReplace,
        );

        debug_assert!(error == 0);
    }

    fn consume_frame(
        &self,
        frame: Frame<'elem>,
        key: &str,
        append: ffi::VSMapAppendMode,
    ) -> MapResult<()> {
        let key = MapRef::make_raw_key(key)?;
        let frame = mem::ManuallyDrop::new(frame);
        let frame_ptr = frame.as_ptr();
        let res = unsafe {
            API::get_cached().map_consume_frame(
                &self.inner as *const _ as *mut _,
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
    pub fn append_consume_frame(&self, frame: Frame<'elem>, key: &str) -> MapResult<()> {
        self.consume_frame(frame, key, ffi::VSMapAppendMode::maAppend)
    }

    /// Consumes a frame and sets it in the map. Replaces any existing values.
    #[inline]
    pub fn set_consume_frame(&self, frame: Frame<'elem>, key: &str) -> MapResult<()> {
        self.consume_frame(frame, key, ffi::VSMapAppendMode::maReplace)
    }

    fn consume_node(&self, node: Node, key: &str, append: ffi::VSMapAppendMode) -> MapResult<()> {
        let key = MapRef::make_raw_key(key)?;
        let node = mem::ManuallyDrop::new(node);
        let node_ptr = node.as_ptr();
        let res = unsafe {
            API::get_cached().map_consume_node(
                &self.inner as *const _ as *mut _,
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
    pub fn append_consume_node(&self, node: Node, key: &str) -> MapResult<()> {
        self.consume_node(node, key, ffi::VSMapAppendMode::maAppend)
    }

    /// Consumes a node and sets it in the map. Replaces any existing values.
    #[inline]
    pub fn set_consume_node(&self, node: Node, key: &str) -> MapResult<()> {
        self.consume_node(node, key, ffi::VSMapAppendMode::maReplace)
    }

    fn consume_function(
        &self,
        func: Function<'elem>,
        key: &str,
        append: ffi::VSMapAppendMode,
    ) -> MapResult<()> {
        let key = MapRef::make_raw_key(key)?;
        let func = mem::ManuallyDrop::new(func);
        let func_ptr = func.as_ptr();
        let res = unsafe {
            API::get_cached().map_consume_function(
                &self.inner as *const _ as *mut _,
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
    pub fn append_consume_function(&self, func: Function<'elem>, key: &str) -> MapResult<()> {
        self.consume_function(func, key, ffi::VSMapAppendMode::maAppend)
    }

    /// Consumes a function and sets it in the map. Replaces any existing values.
    #[inline]
    pub fn set_consume_function(&self, func: Function<'elem>, key: &str) -> MapResult<()> {
        self.consume_function(func, key, ffi::VSMapAppendMode::maReplace)
    }

    // TODO: Saturated retrival
}

pub trait IntoOwnedMap {
    fn into_owned_map<'elem>(self) -> Map<'elem>;
}

impl Drop for Map<'_> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            API::get_cached().free_map(self.handle.as_mut());
        }
    }
}
