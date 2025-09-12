//! VapourSynth nodes.

use futures::channel::oneshot;
use rustsynth_sys as ffi;
use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::process;
use std::ptr::NonNull;
use std::{mem, panic};

use crate::api::API;
use crate::format::{AudioInfo, MediaType, VideoInfo};
use crate::frame::{Frame, FrameContext};

mod errors;
pub use self::errors::GetFrameError;

/// A reference to a node in the constructed filter graph.
#[derive(Debug)]
pub struct Node {
    handle: NonNull<ffi::VSNode>,
}

unsafe impl Send for Node {}
unsafe impl Sync for Node {}

impl Drop for Node {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            API::get_cached().free_node(self.handle.as_ptr());
        }
    }
}

impl Clone for Node {
    #[inline]
    fn clone(&self) -> Self {
        let handle = unsafe { API::get_cached().clone_node(self.handle.as_ptr()) };
        Self {
            handle: unsafe { NonNull::new_unchecked(handle) },
        }
    }
}

impl Node {
    /// Wraps `handle` in a `Node`.
    ///
    /// # Safety
    /// The caller must ensure `handle` and the lifetime is valid and API is cached.
    #[inline]
    pub(crate) unsafe fn from_ptr(handle: *mut ffi::VSNode) -> Self {
        Self {
            handle: NonNull::new_unchecked(handle),
        }
    }

    /// Returns the underlying pointer.
    #[inline]
    pub(crate) fn ptr(&self) -> *mut ffi::VSNode {
        self.handle.as_ptr()
    }

    ///Determines the strategy for frame caching. Pass a [CacheMode] constant. Mostly useful for cache debugging since the auto mode should work well in just about all cases.
    ///
    ///Resets the cache to default options when called, discarding [Node::set_cache_options] changes.
    #[inline]
    pub fn set_cache_mode(&self, mode: CacheMode) {
        unsafe { API::get_cached().set_cache_mode(self.ptr(), mode as i32) }
    }

    /// Call after [Node::set_cache_mode] or the changes will be discarded. Sets internal details of a node’s associated cache.
    /// # Arguments
    ///
    /// * `fixed_size`: Set to non-zero to make the cache always hold maxSize frames.
    /// * `max_size`: The maximum number of frames to cache. Note that this value is automatically adjusted using an internal algorithm unless fixedSize is set.
    /// * `max_history_size`: How many frames that have been recently evicted from the cache to keep track off. Used to determine if growing or shrinking the cache is beneficial. Has no effect when fixedSize is set.
    #[inline]
    pub fn set_cache_options(&self, fixed_size: i32, max_size: i32, max_history_size: i32) {
        unsafe {
            API::get_cached().set_cache_options(self.ptr(), fixed_size, max_size, max_history_size)
        }
    }

    /// Returns the video info associated with this `Node`.
    // Since we don't store the pointer to the actual `ffi::VSVideoInfo` and the lifetime is that
    // of the `ffi::VSFormat`, this returns `VideoInfo<'core>` rather than `VideoInfo<'a>`.
    #[inline]
    pub fn video_info(&self) -> Option<VideoInfo> {
        unsafe {
            let ptr = API::get_cached().get_video_info(self.handle.as_ptr());
            if ptr.is_null() {
                None
            } else {
                Some(VideoInfo::from_ptr(ptr))
            }
        }
    }

    /// Returns the audio info associated with this `Node`.
    // Since we don't store the pointer to the actual `ffi::VSVideoInfo` and the lifetime is that
    // of the `ffi::VSFormat`, this returns `VideoInfo<'core>` rather than `VideoInfo<'a>`.
    #[inline]
    pub fn audio_info(&self) -> Option<AudioInfo> {
        unsafe {
            let ptr = API::get_cached().get_audio_info(self.handle.as_ptr());
            if ptr.is_null() {
                None
            } else {
                Some(AudioInfo::from_ptr(ptr))
            }
        }
    }

    /// Generates a frame directly.
    ///
    /// The `'error` lifetime is unbounded because this function always returns owned data.
    ///
    /// # Panics
    /// Panics is `n` is greater than [i32::max_value()].
    pub fn get_frame<'core, 'error>(
        &self,
        n: usize,
    ) -> Result<Frame<'core>, GetFrameError<'error>> {
        assert!(n <= i32::max_value() as usize);

        let vi = &self.video_info().unwrap();

        let total = vi.num_frames;
        if n >= total as usize {
            let err_cstring = CString::new("Requested frame number beyond the last one").unwrap();
            return Err(GetFrameError::new(Cow::Owned(err_cstring)));
        }

        // Kinda arbitrary. Same value as used in vsvfw.
        const ERROR_BUF_CAPACITY: usize = 32 * 1024;

        let err_buf = vec![0; ERROR_BUF_CAPACITY];
        let mut err_buf = err_buf.into_boxed_slice();

        let handle =
            unsafe { API::get_cached().get_frame(n as i32, self.handle.as_ptr(), &mut err_buf) };

        if handle.is_null() {
            // TODO: remove this extra allocation by reusing `Box<[c_char]>`.
            let error = unsafe { CStr::from_ptr(err_buf.as_ptr()) }.to_owned();
            Err(GetFrameError::new(Cow::Owned(error)))
        } else {
            Ok(Frame::from_ptr(handle))
        }
    }

    /// Requests the generation of a frame. When the frame is ready, a user-provided function is
    /// called.
    ///
    /// If multiple frames were requested, they can be returned in any order.
    ///
    /// The callback arguments are:
    ///
    /// - the generated frame or an error message if the generation failed,
    /// - the frame number (equal to `n`),
    /// - the node that generated the frame (the same as `self`).
    ///
    /// If the callback panics, the process is aborted.
    ///
    /// # Panics
    /// Panics is `n` is greater than [i32::max_value()].
    pub fn get_frame_async<'core, F>(&self, n: usize, callback: F)
    where
        F: FnOnce(Result<Frame<'core>, GetFrameError>, usize, Node) + Send + 'core,
    {
        struct CallbackData<'core> {
            callback: Box<dyn CallbackFn<'core> + 'core>,
        }

        // A little bit of magic for Box<FnOnce>.
        trait CallbackFn<'core> {
            fn call(
                self: Box<Self>,
                frame: Result<Frame<'core>, GetFrameError>,
                n: usize,
                node: Node,
            );
        }

        impl<'core, F> CallbackFn<'core> for F
        where
            F: FnOnce(Result<Frame<'core>, GetFrameError>, usize, Node),
        {
            #[allow(clippy::boxed_local)]
            fn call(
                self: Box<Self>,
                frame: Result<Frame<'core>, GetFrameError>,
                n: usize,
                node: Node,
            ) {
                (self)(frame, n, node)
            }
        }

        unsafe extern "C" fn c_callback(
            user_data: *mut c_void,
            frame: *const ffi::VSFrame,
            n: i32,
            node: *mut ffi::VSNode,
            error_msg: *const c_char,
        ) {
            // The actual lifetime isn't 'static, it's 'core, but we don't really have a way of
            // retrieving it.
            let user_data = Box::from_raw(user_data as *mut CallbackData<'static>);

            let closure = panic::AssertUnwindSafe(move || {
                let frame = if frame.is_null() {
                    debug_assert!(!error_msg.is_null());
                    let error_msg = Cow::Borrowed(CStr::from_ptr(error_msg));
                    Err(GetFrameError::new(error_msg))
                } else {
                    debug_assert!(error_msg.is_null());
                    Ok(Frame::from_ptr(frame))
                };

                let node = Node::from_ptr(node);

                debug_assert!(n >= 0);
                let n = n as usize;

                user_data.callback.call(frame, n, node);
            });

            if panic::catch_unwind(closure).is_err() {
                process::abort();
            }
        }

        assert!(n <= i32::max_value() as usize);
        let n = n as i32;

        let user_data = Box::new(CallbackData {
            callback: Box::new(callback),
        });

        let new_node = self.clone();

        unsafe {
            API::get_cached().get_frame_async(
                n,
                new_node.handle.as_ptr(),
                Some(c_callback),
                Box::into_raw(user_data) as *mut c_void,
            );
        }

        // It'll be dropped by the callback.
        mem::forget(new_node);
    }

    /// Returns a future that resolves to the frame at the given index `n`.
    pub fn get_frame_future<'core>(
        &self,
        n: usize,
    ) -> impl std::future::Future<Output = Result<Frame<'core>, String>> + 'core {
        let (sender, receiver) = oneshot::channel();
        self.get_frame_async(n, move |result, _, _| {
            let result_static: Result<Frame<'core>, String> =
                result.map_err(|e| e.into_inner().to_string_lossy().into_owned());
            let _ = sender.send(result_static);
        });

        async move { receiver.await.unwrap() }
    }

    /// Request a frame from a node (used in filter's request_frame function)
    #[inline]
    pub fn request_frame_filter(&self, n: i32, frame_ctx: &FrameContext) {
        unsafe {
            API::get_cached().request_frame_filter(n, self.ptr(), frame_ctx.ptr());
        }
    }

    /// Get a frame from a node (used in filter's get_frame function)
    #[inline]
    pub fn get_frame_filter<'core>(
        &self,
        n: i32,
        frame_ctx: &FrameContext,
    ) -> Option<Frame<'core>> {
        let ptr = unsafe { API::get_cached().get_frame_filter(n, self.ptr(), frame_ctx.ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(Frame::from_ptr(ptr))
        }
    }

    #[inline]
    pub fn media_type(&self) -> MediaType {
        let int = unsafe { API::get_cached().get_node_type(self.ptr()) };
        match int {
            x if x == ffi::VSMediaType::mtAudio as i32 => MediaType::Audio,
            x if x == ffi::VSMediaType::mtVideo as i32 => MediaType::Video,
            _ => unreachable!(),
        }
    }

    /// Must be called immediately after audio or video filter creation. Returns the upper bound of how many additional frames it is reasonable to pass to [Frame::cache_frame] when trying to make a request more linear.
    pub fn set_linear_filter(&self) -> i32 {
        unsafe { API::get_cached().set_linear_filter(self.ptr()) }
    }

    pub fn release_frame_early(&self, n: i32, frame_ctx: &FrameContext) {
        unsafe {
            API::get_cached().release_frame_early(self.ptr(), n, frame_ctx.ptr());
        }
    }
}

/// Describes how the output of a node is cached.
pub enum CacheMode {
    /// Cache is enabled or disabled based on the reported request patterns and number of consumers.
    Auto,
    /// Never cache anything.
    ForceDisable,
    /// Always cache everything.
    ForceEnable,
}

impl CacheMode {
    pub fn to_ptr(&self) -> *const ffi::VSCacheMode {
        match self {
            CacheMode::Auto => &ffi::VSCacheMode::cmAuto,
            CacheMode::ForceDisable => &ffi::VSCacheMode::cmForceDisable,
            CacheMode::ForceEnable => &ffi::VSCacheMode::cmForceEnable,
        }
    }
}
