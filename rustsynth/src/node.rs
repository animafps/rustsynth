use rustsynth_sys as ffi;
use std::ptr::NonNull;

use crate::{prelude::API, MediaType, VideoInfo};

#[derive(Debug, Copy, Clone)]
pub struct Node {
    handle: NonNull<ffi::VSNode>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum CacheMode {
    Auto,
    ForceEnable,
    ForceDisable,
}

pub struct NodeRef<'a> {
    owner: &'a Node,
}

impl Node {
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
        Some(VideoInfo::from(info))
    }

    pub fn set_cache_mode(&self, mode: CacheMode) {
        todo!()
    }

    pub(crate) fn ptr(&self) -> *mut ffi::VSNode {
        self.handle.as_ptr()
    }
}
