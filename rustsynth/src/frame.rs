//! VapourSynth frames.

use rustsynth_sys as ffi;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::slice;

use thiserror::Error;

use crate::api::API;
use crate::core::CoreRef;
use crate::format::{Component, VideoFormat};
use crate::map::{MapRef, MapRefMut};

/// An error indicating that the frame data has non-zero padding.
#[derive(Error, Debug, Clone, Copy, Eq, PartialEq)]
#[error("Frame data has non-zero padding: {}", _0)]
pub struct NonZeroPadding(isize);

/// One frame of a clip.
// This type is intended to be publicly used only in reference form.
#[derive(Debug)]
pub struct Frame<'core> {
    // The actual mutability of this depends on whether it's accessed via `&Frame` or `&mut Frame`.
    handle: NonNull<ffi::VSFrame>,
    _owner: PhantomData<&'core ()>,
}

/// A reference to a ref-counted frame.
#[derive(Debug)]
pub struct FrameRef<'core> {
    // Only immutable references to this are allowed.
    frame: Frame<'core>,
}

/// A reference to a mutable frame.
#[derive(Debug)]
pub struct FrameRefMut<'core> {
    // Both mutable and immutable references to this are allowed.
    frame: Frame<'core>,
}

unsafe impl<'core> Send for Frame<'core> {}
unsafe impl<'core> Sync for Frame<'core> {}

#[doc(hidden)]
impl<'core> Deref for Frame<'core> {
    type Target = ffi::VSFrame;

    // Technically this should return `&'core`.
    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.handle.as_ref() }
    }
}

#[doc(hidden)]
impl<'core> DerefMut for Frame<'core> {
    // Technically this should return `&'core`.
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.handle.as_mut() }
    }
}

impl<'core> Drop for Frame<'core> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            API::get_cached().free_frame(self);
        }
    }
}

impl<'core> Deref for FrameRef<'core> {
    type Target = Frame<'core>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.frame
    }
}

impl<'core> Deref for FrameRefMut<'core> {
    type Target = Frame<'core>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.frame
    }
}

impl<'core> DerefMut for FrameRefMut<'core> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.frame
    }
}

impl<'core> FrameRef<'core> {
    /// Wraps `handle` in a `FrameRef`.
    ///
    /// # Safety
    /// The caller must ensure `handle` and the lifetime is valid and API is cached.
    #[inline]
    pub(crate) unsafe fn from_ptr(handle: *const ffi::VSFrame) -> Self {
        Self {
            frame: Frame::from_ptr(handle),
        }
    }
}

impl<'core> FrameRefMut<'core> {
    /// Wraps `handle` in a `FrameRefMut`.
    ///
    /// # Safety
    /// The caller must ensure `handle` and the lifetime is valid and API is cached.
    #[inline]
    pub(crate) unsafe fn from_ptr(handle: *mut ffi::VSFrame) -> Self {
        Self {
            frame: Frame::from_ptr(handle),
        }
    }

    /// Creates a copy of the given frame.
    ///
    /// The plane data is copy-on-write, so this isn't very expensive by itself.
    ///
    /// Judging by the underlying implementation, it seems that any valid `core` can be used.
    #[inline]
    pub fn copy_of(core: CoreRef, frame: &Frame<'core>) -> Self {
        Self {
            frame: unsafe { Frame::from_ptr(API::get_cached().copy_frame(frame, core.ptr())) },
        }
    }

    /// Creates a new frame with uninitialized plane data.
    ///
    /// Optionally copies the frame properties from the provided `prop_src` frame.
    ///
    /// # Safety
    /// The returned frame contains uninitialized plane data. This should be handled carefully. See
    /// the docs for `std::mem::uninitialized()` for more information.
    ///
    /// # Panics
    /// Panics if the given resolution has components that don't fit into an `i32`.
    #[inline]
    pub unsafe fn new_uninitialized(
        core: CoreRef<'core>,
        prop_src: Option<&Frame<'core>>,
        format: VideoFormat,
        width: i32,
        height: i32,
    ) -> Self {
        todo!();
        //Self {
        //    frame: unsafe {
        //        Frame::from_ptr(API::get_cached().new_video_frame(
        //            format,
        //            width,
        //            height,
        //            prop_src.map(|f| f.deref() as _).unwrap_or(ptr::null()),
        //            core.ptr(),
        //        ))
        //    },
        //}
    }
}

impl<'core> From<FrameRefMut<'core>> for FrameRef<'core> {
    #[inline]
    fn from(x: FrameRefMut<'core>) -> Self {
        Self { frame: x.frame }
    }
}

impl<'core> Frame<'core> {
    /// Converts a pointer to a frame to a reference.
    ///
    /// # Safety
    /// The caller needs to ensure the pointer and the lifetime is valid, and that the resulting
    /// `Frame` gets put into `FrameRef` or `FrameRefMut` according to the input pointer
    /// mutability.
    #[inline]
    pub(crate) unsafe fn from_ptr(handle: *const ffi::VSFrame) -> Self {
        Self {
            handle: NonNull::new_unchecked(handle as *mut ffi::VSFrame),
            _owner: PhantomData,
        }
    }

    pub fn video_format(&self) -> Option<VideoFormat> {
        let ptr = unsafe { API::get_cached().get_video_frame_format(self.handle.as_ref()) };
        if ptr.is_null() {
            None
        } else {
            Some(VideoFormat::from_ptr(ptr))
        }
    }

    /// Returns the width of a plane, in pixels.
    ///
    /// The width depends on the plane number because of the possible chroma subsampling.
    ///
    /// # Panics
    /// Panics if `plane >= format().plane_count()`.
    #[inline]
    pub fn width(&self, plane: i32) -> i32 {
        assert!(plane < self.video_format().unwrap().num_planes);

        unsafe { API::get_cached().get_frame_width(self, plane) }
    }

    /// Returns the height of a plane, in pixels.
    ///
    /// The height depends on the plane number because of the possible chroma subsampling.
    ///
    /// # Panics
    /// Panics if `plane >= format().plane_count()`.
    #[inline]
    pub fn height(&self, plane: i32) -> i32 {
        assert!(plane < self.video_format().unwrap().num_planes);

        unsafe { API::get_cached().get_frame_height(self, plane) }
    }

    /// Returns the distance in bytes between two consecutive lines of a plane.
    ///
    /// # Panics
    /// Panics if `plane >= format().plane_count()`.
    #[inline]
    pub fn stride(&self, plane: i32) -> isize {
        assert!(plane < self.video_format().unwrap().num_planes);

        unsafe { API::get_cached().get_frame_stride(self, plane) }
    }

    pub fn length(&self) -> i32 {
        unsafe { API::get_cached().get_frame_length(self) }
    }

    /// Returns a slice of a plane's pixel row.
    ///
    /// # Panics
    /// Panics if the requested plane, row or component type is invalid.
    #[inline]
    pub fn plane_row<T: Component>(&self, plane: i32, row: i32) -> &[T] {
        assert!(plane < self.video_format().unwrap().num_planes);
        assert!(row < self.height(plane));
        assert!(T::is_valid(self.video_format().unwrap()));

        let stride = self.stride(plane);
        let ptr = self.data_ptr(plane);

        let offset = stride * row as isize;
        assert!(offset <= isize::max_value());
        let offset = offset;

        let row_ptr = unsafe { ptr.offset(offset) };
        let width = self.width(plane);

        unsafe { slice::from_raw_parts(row_ptr as *const T, width as usize) }
    }

    /// Returns a mutable slice of a plane's pixel row.
    ///
    /// # Panics
    /// Panics if the requested plane, row or component type is invalid.
    #[inline]
    pub fn plane_row_mut<T: Component>(&mut self, plane: i32, row: i32) -> &mut [T] {
        assert!(plane < self.video_format().unwrap().num_planes);
        assert!(row < self.height(plane));
        assert!(T::is_valid(self.video_format().unwrap()));

        let stride = self.stride(plane);
        let ptr = self.data_ptr_mut(plane);

        let offset = stride * row as isize;
        assert!(offset <= isize::max_value());
        let offset = offset;

        let row_ptr = unsafe { ptr.offset(offset) };
        let width = self.width(plane);

        unsafe { slice::from_raw_parts_mut(row_ptr as *mut T, width.try_into().unwrap()) }
    }

    /// Returns a slice of the plane's pixels.
    ///
    /// The length of the returned slice is `height() * width()`. If the pixel data has non-zero
    /// padding (that is, `stride()` is larger than `width()`), an error is returned, since
    /// returning the data slice would open access to uninitialized bytes.
    ///
    /// # Panics
    /// Panics if the requested plane or component type is invalid.
    pub fn plane<T: Component>(&self, plane: i32) -> Result<&[T], NonZeroPadding> {
        assert!(plane < self.video_format().unwrap().num_planes);
        assert!(T::is_valid(self.video_format().unwrap()));

        let stride = self.stride(plane);
        let width_in_bytes = self.width(plane) * self.video_format().unwrap().bytes_per_sample;
        if stride != width_in_bytes as isize {
            return Err(NonZeroPadding(stride - width_in_bytes as isize));
        }

        let height = self.height(plane);
        let length = height * self.width(plane);
        let ptr = self.data_ptr(plane);

        Ok(unsafe { slice::from_raw_parts(ptr as *const T, length as usize) })
    }

    /// Returns a mutable slice of the plane's pixels.
    ///
    /// The length of the returned slice is `height() * width()`. If the pixel data has non-zero
    /// padding (that is, `stride()` is larger than `width()`), an error is returned, since
    /// returning the data slice would open access to uninitialized bytes.
    ///
    /// # Panics
    /// Panics if the requested plane or component type is invalid.
    pub fn plane_mut<T: Component>(&mut self, plane: i32) -> Result<&mut [T], NonZeroPadding> {
        assert!(plane < self.video_format().unwrap().num_planes);
        assert!(T::is_valid(self.video_format().unwrap()));

        let stride = self.stride(plane);
        let width_in_bytes =
            (self.width(plane) * self.video_format().unwrap().bytes_per_sample) as isize;
        if stride != width_in_bytes {
            return Err(NonZeroPadding(stride - width_in_bytes));
        }

        let height = self.height(plane);
        let length = height * self.width(plane);
        let ptr = self.data_ptr_mut(plane);

        Ok(unsafe { slice::from_raw_parts_mut(ptr as *mut T, length.try_into().unwrap()) })
    }

    /// Returns a pointer to the plane's pixels.
    ///
    /// The pointer points to an array with a length of `height() * stride()` and is valid for as
    /// long as the frame is alive.
    ///
    /// # Panics
    /// Panics if `plane >= format().plane_count()`.
    #[inline]
    pub fn data_ptr(&self, plane: i32) -> *const u8 {
        assert!(plane < self.video_format().unwrap().num_planes);

        unsafe { API::get_cached().get_frame_read_ptr(self, plane) }
    }

    /// Returns a mutable pointer to the plane's pixels.
    ///
    /// The pointer points to an array with a length of `height() * stride()` and is valid for as
    /// long as the frame is alive.
    ///
    /// # Panics
    /// Panics if `plane >= format().plane_count()`.
    #[inline]
    pub fn data_ptr_mut(&mut self, plane: i32) -> *mut u8 {
        assert!(plane < self.video_format().unwrap().num_planes);

        unsafe { API::get_cached().get_frame_write_ptr(self.handle.as_mut(), plane) }
    }

    /// Returns a slice of a plane's pixel row.
    ///
    /// The length of the returned slice is equal to `width() * format().bytes_per_sample()`.
    ///
    /// # Panics
    /// Panics if `plane >= format().plane_count()` or if `row >= height()`.
    pub fn data_row(&self, plane: i32, row: i32) -> &[u8] {
        assert!(plane < self.video_format().unwrap().num_planes);
        assert!(row < self.height(plane));

        let stride = self.stride(plane);
        let ptr = self.data_ptr(plane);

        let offset = stride * row as isize;
        assert!(offset <= isize::max_value());
        let offset = offset;

        let row_ptr = unsafe { ptr.offset(offset) };
        let width = self.width(plane) * self.video_format().unwrap().bytes_per_sample;

        unsafe { slice::from_raw_parts(row_ptr, width.try_into().unwrap()) }
    }

    /// Returns a mutable slice of a plane's pixel row.
    ///
    /// The length of the returned slice is equal to `width() * format().bytes_per_sample()`.
    ///
    /// # Panics
    /// Panics if `plane >= format().plane_count()` or if `row >= height()`.
    pub fn data_row_mut(&mut self, plane: i32, row: i32) -> &mut [u8] {
        assert!(plane < self.video_format().unwrap().num_planes);
        assert!(row < self.height(plane));

        let stride = self.stride(plane);
        let ptr = self.data_ptr_mut(plane);

        let offset = stride * row as isize;
        assert!(offset <= isize::max_value());
        let offset = offset;

        let row_ptr = unsafe { ptr.offset(offset) };
        let width = self.width(plane) * self.video_format().unwrap().bytes_per_sample;

        unsafe { slice::from_raw_parts_mut(row_ptr, width.try_into().unwrap()) }
    }

    /// Returns a slice of the plane's pixels.
    ///
    /// The length of the returned slice is `height() * width() * format().bytes_per_sample()`. If
    /// the pixel data has non-zero padding (that is, `stride()` is larger than `width()`), an
    /// error is returned, since returning the data slice would open access to uninitialized bytes.
    ///
    /// # Panics
    /// Panics if `plane >= format().plane_count()` or if `row >= height()`.
    pub fn data(&self, plane: i32) -> Result<&[u8], NonZeroPadding> {
        assert!(plane < self.video_format().unwrap().num_planes);

        let stride = self.stride(plane);
        let width = (self.width(plane) * self.video_format().unwrap().bytes_per_sample) as isize;
        if stride != width {
            return Err(NonZeroPadding(stride - width));
        }

        let height = self.height(plane) as isize;
        let length = height * stride;
        let ptr = self.data_ptr(plane);

        Ok(unsafe { slice::from_raw_parts(ptr, length.try_into().unwrap()) })
    }

    /// Returns a mutable slice of the plane's pixels.
    ///
    /// The length of the returned slice is `height() * width() * format().bytes_per_sample()`. If
    /// the pixel data has non-zero padding (that is, `stride()` is larger than `width()`), an
    /// error is returned, since returning the data slice would open access to uninitialized bytes.
    ///
    /// # Panics
    /// Panics if `plane >= format().plane_count()` or if `row >= height()`.
    pub fn data_mut(&mut self, plane: i32) -> Result<&mut [u8], NonZeroPadding> {
        assert!(plane < self.video_format().unwrap().num_planes);

        let stride = self.stride(plane);
        let width = (self.width(plane) * self.video_format().unwrap().bytes_per_sample) as isize;
        if stride != width {
            return Err(NonZeroPadding(stride - width));
        }

        let height = self.height(plane) as isize;
        let length = height * stride;
        let ptr = self.data_ptr_mut(plane);

        Ok(unsafe { slice::from_raw_parts_mut(ptr, length.try_into().unwrap()) })
    }

    /// Returns a map of frame's properties.
    #[inline]
    pub fn props(&self) -> MapRef {
        unsafe { MapRef::from_ptr(API::get_cached().get_frame_props_ro(self)) }
    }

    /// Returns a mutable map of frame's properties.
    #[inline]
    pub fn props_mut(&mut self) -> MapRefMut {
        unsafe { MapRefMut::from_ptr(API::get_cached().get_frame_props_rw(self.deref_mut())) }
    }
}
