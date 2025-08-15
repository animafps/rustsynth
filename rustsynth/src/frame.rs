use std::{marker::PhantomData, ops::Deref, ptr::NonNull};

use rustsynth_sys as ffi;

use crate::{
    api::API,
    core::CoreRef,
    format::{AudioFormat, VideoFormat},
};

// One frame of a clip.
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

unsafe impl<'core> Send for Frame<'core> {}
unsafe impl<'core> Sync for Frame<'core> {}

unsafe impl<'core> Send for FrameRef<'core> {}
unsafe impl<'core> Sync for FrameRef<'core> {}

impl<'core> Drop for Frame<'core> {
    fn drop(&mut self) {
        unsafe { API::get_cached().free_frame(self.handle.as_ptr()) }
    }
}

impl<'core> Deref for FrameRef<'core> {
    type Target = Frame<'core>;

    fn deref(&self) -> &Self::Target {
        &self.frame
    }
}

impl<'core> FrameRef<'core> {
    #[inline]
    pub(crate) fn from_ptr(ptr: *const ffi::VSFrame) -> Self {
        Self {
            frame: Frame::from_ptr(ptr),
        }
    }
    
    #[inline]
    #[allow(unused)]
    pub(crate) fn into_ptr(self) -> *const ffi::VSFrame {
        let ptr = self.frame.handle.as_ptr();
        std::mem::forget(self); // Don't drop the frame, transfer ownership to C
        ptr
    }
}

#[derive(Debug)]
pub struct FrameContext {
    handle: *mut ffi::VSFrameContext,
}

impl FrameContext {
    #[inline]
    #[allow(unused)]
    pub(crate) fn from_ptr(ptr: *mut ffi::VSFrameContext) -> Self {
        Self {
            handle: ptr
        }
    }

    #[inline]
    pub(crate) fn ptr(&self) -> *mut ffi::VSFrameContext {
        self.handle
    }
}

impl<'core> Frame<'core> {
    #[inline]
    pub(crate) fn from_ptr(ptr: *const ffi::VSFrame) -> Self {
        Self {
            handle: unsafe { NonNull::new_unchecked(ptr as *mut ffi::VSFrame) },
            _owner: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn ptr(&self) -> *const ffi::VSFrame {
        self.handle.as_ptr()
    }

    /// Returns the height of a plane of a given frame, in pixels. The height depends on the plane number because of the possible chroma subsampling.
    #[inline]
    pub fn get_height(&self, plane: i32) -> i32 {
        unsafe { API::get_cached().get_frame_height(self.handle.as_ref(), plane) }
    }

    /// Returns the width of a plane of a given frame, in pixels. The width depends on the plane number because of the possible chroma subsampling.
    #[inline]
    pub fn get_width(&self, plane: i32) -> i32 {
        unsafe { API::get_cached().get_frame_width(self.handle.as_ref(), plane) }
    }

    #[inline]
    pub fn get_length(&self) -> i32 {
        unsafe { API::get_cached().get_frame_length(self.handle.as_ref()) }
    }

    /// Returns the distance in bytes between two consecutive lines of a plane of a frame. The stride is always positive.
    ///
    /// Passing an invalid plane number will cause a fatal error.
    #[inline]
    pub fn get_stride(&self, plane: i32) -> isize {
        unsafe { API::get_cached().get_frame_stride(self.handle.as_ref(), plane) }
    }

    #[inline]
    pub fn get_video_format(&self) -> Option<VideoFormat> {
        let ptr = unsafe { API::get_cached().get_video_frame_format(self.handle.as_ref()) };
        if ptr.is_null() {
            None
        } else {
            Some(VideoFormat::from_ptr(ptr))
        }
    }

    pub fn get_audio_format(&self) -> Option<AudioFormat> {
        let ptr = unsafe { API::get_cached().get_audio_frame_format(self.handle.as_ptr())};
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { AudioFormat::from_ptr(ptr) })
        }
    }


    /// Creates a new video frame, optionally copying the properties attached to another frame.
    pub fn new_video_frame(
        core: &CoreRef,
        width: i32,
        height: i32,
        format: &VideoFormat,
        prop_src: Option<&FrameRef<'_>>,
    ) -> Self {
        let ptr = unsafe {
            API::get_cached().new_video_frame(
                format.as_ptr(),
                width,
                height,
                prop_src.map_or(std::ptr::null(), |f| f.ptr()),
                core.ptr(),
            )
        };
        if ptr.is_null() {
            panic!("Failed to create new video frame");
        }
        Frame::from_ptr(ptr)
    }

    /// Creates a new video frame from the planes of existing frames, optionally copying the properties attached to another frame
    pub fn new_video_frame_from_existing_planes<const T: usize>(
        core: &CoreRef,
        width: i32,
        height: i32,
        format: VideoFormat,
        planesrc: &mut [&FrameRef<'_>; T],
        planes: &[i32; T],
        propsrc: Option<&FrameRef<'_>>,
    ) -> Self {
        let ptr = unsafe {
            let mut planesrcptr: Vec<_> = planesrc.iter().map(|f| f.ptr()).collect();
            API::get_cached().new_video_frame2(
                format.as_ptr(),
                width,
                height,
                planesrcptr.as_mut_ptr(),
                planes.as_ptr(),
                propsrc.map_or(std::ptr::null(), |f| f.ptr()),
                core.ptr(),
            )
        };
        if ptr.is_null() {
            panic!("Failed to create new video frame from existing planes");
        }
        Frame::from_ptr(ptr)
    }

    /// Get read-only access to plane data
    #[inline]
    pub fn get_read_slice(&self, plane: i32) -> &[u8] {
        let ptr = unsafe { API::get_cached().get_frame_read_ptr(self.handle.as_ref(), plane) };
        let height = self.get_height(plane);
        let stride = self.get_stride(plane);
        let len = (height as isize * stride) as usize;
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }

    /// Get mutable access to plane data (only for owned frames)
    #[inline] 
    pub fn get_write_slice(&mut self, plane: i32) -> &mut [u8] {
        let ptr = unsafe { API::get_cached().get_frame_write_ptr(self.handle.as_ptr(), plane) };
        let height = self.get_height(plane);
        let stride = self.get_stride(plane);
        let len = (height as isize * stride) as usize;
        unsafe { std::slice::from_raw_parts_mut(ptr, len) }
    }

    /// Convert owned frame to FrameRef
    #[inline]
    pub fn into_frame_ref(self) -> FrameRef<'core> {
        let ptr = self.ptr();
        std::mem::forget(self); // Don't drop the frame, transfer ownership
        FrameRef::from_ptr(ptr)
    }
}

impl<'core> Deref for Frame<'core> {
    type Target = ffi::VSFrame;

    fn deref(&self) -> &Self::Target {
        unsafe { self.handle.as_ref() }
    }
}
