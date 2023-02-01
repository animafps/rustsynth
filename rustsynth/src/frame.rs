use std::{marker::PhantomData, ops::Deref, ptr::NonNull};

use rustsynth_sys as ffi;

use crate::{
    api::API,
    format::{AudioFormat, VideoFormat},
};

/// One frame of a clip.
// This type is intended to be publicly used only in reference form.
#[derive(Debug)]
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

    pub fn get_height(&self, plane: i32) -> i32 {
        unsafe { API::get_cached().get_frame_height(self.ptr(), plane) }
    }

    pub fn get_width(&self, plane: i32) -> i32 {
        unsafe { API::get_cached().get_frame_width(self.ptr(), plane) }
    }

    pub fn get_length(&self) -> i32 {
        unsafe { API::get_cached().get_frame_length(self.ptr()) }
    }
}

impl<'elem> Deref for Frame<'elem> {
    type Target = ffi::VSFrame;

    fn deref(&self) -> &Self::Target {
        unsafe { self.handle.as_ref() }
    }
}

impl<'elem> Deref for AudioFrame<'elem> {
    type Target = Frame<'elem>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'elem> Deref for VideoFrame<'elem> {
    type Target = Frame<'elem>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct AudioFrame<'elem> {
    inner: Frame<'elem>,
    format: AudioFormat,
}

impl<'elem> AudioFrame<'elem> {
    pub(crate) fn new(inner: Frame<'elem>, format: AudioFormat) -> Self {
        Self { inner, format }
    }
}

pub struct VideoFrame<'elem> {
    inner: Frame<'elem>,
    format: VideoFormat,
}

impl<'elem> VideoFrame<'elem> {
    pub(crate) fn new(inner: Frame<'elem>, format: VideoFormat) -> Self {
        Self { inner, format }
    }
}
