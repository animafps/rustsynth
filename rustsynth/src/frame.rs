use std::{marker::PhantomData, ops::Deref, ptr::NonNull};

use rustsynth_sys as ffi;

use crate::{
    api::API,
    core::CoreRef,
    format::{AudioFormat, VideoFormat},
};

/// One frame of a clip.
// This type is intended to be publicly used only in reference form.
#[derive(Debug, Clone)]
pub struct Frame<'elem> {
    // The actual mutability of this depends on whether it's accessed via `&Frame` or `&mut Frame`.
    handle: NonNull<ffi::VSFrame>,
    _owner: PhantomData<&'elem ()>,
}

impl<'elem> Drop for Frame<'elem> {
    fn drop(&mut self) {
        unsafe { API::get_cached().free_frame(self.handle.as_ptr()) }
    }
}

#[derive(Debug)]
pub struct FrameRef<'owner, 'elem> {
    inner: Frame<'elem>,
    _owner: PhantomData<&'owner ()>,
}

impl<'owner, 'elem> Deref for FrameRef<'owner, 'elem> {
    type Target = Frame<'elem>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'owner, 'elem> FrameRef<'owner, 'elem> {
    fn from_ptr(ptr: *const ffi::VSFrame) -> Self {
        Self {
            inner: Frame::from_ptr(ptr),
            _owner: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct FrameContext<'elem> {
    _owner: PhantomData<&'elem ()>,
}

impl<'elem> Frame<'elem> {
    pub(crate) fn from_ptr(ptr: *const ffi::VSFrame) -> Self {
        Self {
            handle: unsafe { NonNull::new_unchecked(ptr as *mut ffi::VSFrame) },
            _owner: PhantomData,
        }
    }

    fn ptr(&self) -> *const ffi::VSFrame {
        self.handle.as_ptr()
    }

    /// Returns the height of a plane of a given frame, in pixels. The height depends on the plane number because of the possible chroma subsampling.
    pub fn get_height(&self, plane: i32) -> i32 {
        unsafe { API::get_cached().get_frame_height(self.ptr(), plane) }
    }

    /// Returns the width of a plane of a given frame, in pixels. The width depends on the plane number because of the possible chroma subsampling.
    pub fn get_width(&self, plane: i32) -> i32 {
        unsafe { API::get_cached().get_frame_width(self.ptr(), plane) }
    }

    pub fn get_length(&self) -> i32 {
        unsafe { API::get_cached().get_frame_length(self.ptr()) }
    }

    /// Returns the distance in bytes between two consecutive lines of a plane of a frame. The stride is always positive.
    ///
    /// Passing an invalid plane number will cause a fatal error.
    pub fn get_stride(&self, plane: i32) -> isize {
        unsafe { API::get_cached().get_frame_stride(self.ptr(), plane) }
    }

    pub fn get_video_format(&self) -> Option<VideoFormat> {
        let ptr = unsafe {API::get_cached().get_video_frame_format(self.ptr())};
        if ptr.is_null() {
            None
        } else {
            Some(VideoFormat::from(ptr))
        }
    }

    pub fn get_audio_format(&self) -> Option<AudioFormat> {
        todo!()
    }
}

impl<'elem> Deref for Frame<'elem> {
    type Target = ffi::VSFrame;

    fn deref(&self) -> &Self::Target {
        unsafe { self.handle.as_ref() }
    }
}
