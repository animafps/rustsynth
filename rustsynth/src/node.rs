use rustsynth_sys as ffi;
use std::{
    marker::PhantomData,
    ops::Deref,
    ptr::{self, NonNull},
};

use crate::{
    format::{AudioInfo, MediaType, VideoInfo},
    frame::Frame,
    prelude::API,
};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum CacheMode {
    Auto,
    ForceEnable,
    ForceDisable,
}

#[derive(Debug, Clone)]
pub struct Node<'elem> {
    handle: NonNull<ffi::VSNode>,
    _elem: PhantomData<&'elem ()>,
}

impl<'elem> Drop for Node<'elem> {
    fn drop(&mut self) {
        unsafe { API::get_cached().free_node(self.handle.as_ptr()) }
    }
}

impl<'elem> Node<'elem> {
    pub(crate) fn from_ptr(ptr: *mut ffi::VSNode) -> Self {
        Self {
            handle: unsafe { NonNull::new_unchecked(ptr) },
            _elem: PhantomData,
        }
    }

    pub fn get_type(&self) -> MediaType {
        let result = unsafe { API::get_cached().get_node_type(self.handle.as_ptr()) };
        match result {
            1 => MediaType::Video,
            2 => MediaType::Audio,
            _ => panic!("Not a valid node"),
        }
    }

    /// Returns the `VideoInfo` struct if the node is a video node
    fn video_info(&self) -> Option<VideoInfo> {
        if self.get_type() == MediaType::Audio {
            return None;
        }
        let info = unsafe {
            API::get_cached()
                .get_video_info(self.handle.as_ptr())
                .read()
        };
        Some(VideoInfo::from(info))
    }

    fn audio_info(&self) -> Option<AudioInfo> {
        if self.get_type() == MediaType::Video {
            return None;
        }
        let info = unsafe {
            API::get_cached()
                .get_audio_info(self.handle.as_ptr())
                .read()
        };
        Some(AudioInfo::from(info))
    }

    pub fn set_cache_mode(&mut self, mode: CacheMode) {
        unsafe { API::get_cached().set_cache_mode(self.handle.as_ptr(), mode as i32) }
    }

    pub fn set_cache_options(&mut self) {
        todo!()
    }

    pub(crate) fn ptr(&self) -> *mut ffi::VSNode {
        self.handle.as_ptr()
    }

    pub fn get_frame(&self, n: i32) -> Option<Frame> {
        let ptr = unsafe {
            API::get_cached().node_get_frame(self.handle.as_ptr(), n, ptr::null_mut(), 0)
        };
        if ptr.is_null() {
            None
        } else {
            Some(Frame::from_ptr(ptr))
        }
    }

    pub fn get_frame_async(
        &self,
        n: i32,
        callback: unsafe extern "C" fn(
            userData: *mut ::std::os::raw::c_void,
            f: *const ffi::VSFrame,
            n: ::std::os::raw::c_int,
            node: *mut ffi::VSNode,
            errorMsg: *const ::std::os::raw::c_char,
        ),
        user_data: *mut ::std::os::raw::c_void,
    ) {
        unsafe { API::get_cached().node_get_frame_async(self.ptr(), n, callback, user_data) }
    }
}
