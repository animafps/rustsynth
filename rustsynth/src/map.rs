//! VapourSynth map.
use rustsynth_sys as ffi;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Index};
use std::ptr::NonNull;

use crate::api::API;

/// A VapourSynth map.
///
/// A map contains key-value pairs where the value is zero or more elements of a certain type.
/// 
/// It is currently immutable
/// 
/// # Examples
/// 
/// ```
/// use rustsynth::map::Map;
/// let map = Map::new();
/// ```
#[derive(Debug)]
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


impl<'elem> Drop for Map<'elem> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            API::get_cached().free_map(self.ptr());
        }
    }
}

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

    pub fn get(&self, key: &str) -> &Value {
        todo!()
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

    pub fn len(&self) -> usize {
        let int = unsafe { API::get_cached().map_num_keys(self.handle.as_ptr()) };
        int.try_into().unwrap()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Index<&str> for Map<'_> {
    type Output = Value;

    /// Returns a reference to the value corresponding to the supplied key.
    ///
    /// # Panics
    ///
    /// Panics if the key is not present in the `Map`.
    fn index(&self, index: &str) -> &Self::Output {
        self.get(index)
    }
}

impl<'a, 'elem> IntoIterator for &'a Map<'elem> {
    type Item = (&'a str, &'a Value);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
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
}

impl<'a> Iter<'a> {
    pub(crate) fn new(map: &'a Map) -> Self {
        Iter {
            map,
            items: map.len(),
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a str, &'a Value);

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
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

    type Item = &'a Value;
}

/// A struct holding the elements of a value in a map
pub struct Value {}
