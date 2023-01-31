use std::{marker::PhantomData, ops::Deref, ptr::NonNull};

use rustsynth_sys as ffi;

use crate::{AudioFormat, VideoFormat};

/// One frame of a clip.
// This type is intended to be publicly used only in reference form.
#[derive(Debug)]
pub struct Frame<'elem> {
    // The actual mutability of this depends on whether it's accessed via `&Frame` or `&mut Frame`.
    handle: NonNull<ffi::VSFrame>,
    _owner: PhantomData<&'elem ()>,
}

/// A reference to a ref-counted frame.
#[derive(Debug)]
pub struct FrameRef<'core> {
    // Only immutable references to this are allowed.
    frame: Frame<'core>,
}

#[derive(Debug)]
pub struct FrameContext<'core> {
    _owner: PhantomData<&'core ()>,
}

impl<'core> Frame<'core> {
    pub(crate) fn from_ptr(ptr: *const ffi::VSFrame) -> Self {
        Self {
            handle: unsafe { NonNull::new_unchecked(ptr as *mut ffi::VSFrame) },
            _owner: PhantomData,
        }
    }
}

impl<'core> Deref for Frame<'core> {
    type Target = ffi::VSFrame;

    fn deref(&self) -> &Self::Target {
        unsafe { self.handle.as_ref() }
    }
}

#[derive(Debug)]
pub enum Format<'elem> {
    Video(VideoFormat<'elem>),
    Audio(AudioFormat<'elem>),
}

pub struct AudioFrame<'elem> {
    inner: Frame<'elem>,
    format: AudioFormat<'elem>,
}
