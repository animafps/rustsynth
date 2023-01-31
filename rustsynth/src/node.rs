use rustsynth_sys as ffi;
use std::{marker::PhantomData, ptr::NonNull};

use crate::{frame::Frame, prelude::API, MediaType, VideoInfo};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum CacheMode {
    Auto,
    ForceEnable,
    ForceDisable,
}

#[derive(Debug, Clone, Copy)]
pub struct Node<'elem> {
    handle: NonNull<ffi::VSNode>,
    _elem: PhantomData<&'elem ()>,
}

impl<'elem> Node<'elem> {
    pub(crate) fn from_ptr(ptr: *mut ffi::VSNode) -> Self {
        Self {
            handle: unsafe { NonNull::new_unchecked(ptr) },
            _elem: PhantomData,
        }
    }

    pub fn get_type(&self) -> MediaType {
        let result = unsafe { API::get_cached().get_node_type(self.ptr()) };
        match result {
            1 => MediaType::Video,
            2 => MediaType::Audio,
            _ => panic!("Not a valid node"),
        }
    }

    /// Returns the `VideoInfo` struct if the node is a video node
    pub fn video_info(&self) -> Option<VideoInfo> {
        if self.get_type() == MediaType::Audio {
            return None;
        }
        let info = unsafe { API::get_cached().get_video_info(self.ptr()).read() };
        Some(info.into())
    }

    pub fn set_cache_mode(&self, mode: CacheMode) {
        unsafe { API::get_cached().set_cache_mode(self.ptr(), mode as i32) }
    }

    pub fn set_cache_options(&self) {
        todo!()
    }

    pub(crate) fn ptr(self) -> *mut ffi::VSNode {
        self.handle.as_ptr()
    }

    pub fn get_frame(&self, n: i32) -> Option<Frame> {
        todo!()
    }
}

pub struct AudioNode {}

pub struct VideoNode {}
