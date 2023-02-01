use rustsynth_sys as ffi;
use std::{
    marker::PhantomData,
    ops::Deref,
    ptr::{self, NonNull},
};

use crate::{
    format::{AudioInfo, MediaType, VideoInfo},
    frame::{AudioFrame, Frame, VideoFrame},
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

    pub(crate) fn get_type(&self) -> MediaType {
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

    fn get_frame(&self, n: i32) -> Option<Frame> {
        let ptr = unsafe {
            API::get_cached().node_get_frame(self.handle.as_ptr(), n, ptr::null_mut(), 0)
        };
        if ptr.is_null() {
            None
        } else {
            Some(Frame::from_ptr(ptr))
        }
    }
}

pub struct AudioNode<'elem> {
    inner: Node<'elem>,
}

impl<'elem> AudioNode<'elem> {
    pub(crate) fn new(node: Node<'elem>) -> Option<Self> {
        if node.get_type() == MediaType::Video {
            None
        } else {
            Some(Self { inner: node })
        }
    }

    pub fn info(&self) -> AudioInfo {
        self.inner.audio_info().unwrap()
    }

    pub fn get_frame(&self, n: i32) -> Option<AudioFrame> {
        let inner_frame = self.inner.get_frame(n);
        match inner_frame {
            Some(frame) => Some(AudioFrame::new(frame, self.info().format)),
            None => None,
        }
    }
}

impl<'elem> Deref for AudioNode<'elem> {
    type Target = Node<'elem>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct VideoNode<'elem> {
    pub(crate) inner: Node<'elem>,
}

impl<'elem> VideoNode<'elem> {
    pub(crate) fn new(node: Node<'elem>) -> Option<Self> {
        if node.get_type() == MediaType::Audio {
            None
        } else {
            Some(Self { inner: node })
        }
    }

    pub fn info(&self) -> VideoInfo {
        self.inner.video_info().unwrap()
    }

    pub fn get_frame(&self, n: i32) -> Option<VideoFrame> {
        let inner_frame = self.inner.get_frame(n);
        match inner_frame {
            Some(frame) => Some(VideoFrame::new(frame, self.info().format)),
            None => None,
        }
    }
}

impl<'elem> Deref for VideoNode<'elem> {
    type Target = Node<'elem>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
