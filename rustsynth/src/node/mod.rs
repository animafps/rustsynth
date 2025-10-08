//! VapourSynth nodes.

use futures::channel::oneshot;
use rustsynth_sys as ffi;
use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr::NonNull;

use crate::api::API;
#[cfg(feature = "api-41")]
use crate::filter::{FilterDependency, FilterMode};
use crate::format::{AudioInfo, MediaType, VideoInfo};
use crate::frame::{Frame, FrameContext};
#[cfg(feature = "graph-api")]
use crate::map::Map;

mod errors;
pub use self::errors::GetFrameError;

/// A reference to a node in the constructed filter graph.
#[derive(Debug)]
pub struct Node<'core> {
    handle: NonNull<ffi::VSNode>,
    _owner: std::marker::PhantomData<&'core ()>,
}

unsafe impl Send for Node<'_> {}
unsafe impl Sync for Node<'_> {}

impl Clone for Node<'_> {
    #[inline]
    fn clone(&self) -> Self {
        let handle = unsafe { API::get_cached().clone_node(self.handle.as_ptr()) };
        Self {
            handle: unsafe { NonNull::new_unchecked(handle) },
            _owner: self._owner,
        }
    }
}

impl Node<'_> {
    /// Wraps `handle` in a `Node`.
    ///
    /// # Safety
    /// The caller must ensure `handle` and the lifetime is valid and API is cached.
    #[inline]
    pub unsafe fn from_ptr(handle: *mut ffi::VSNode) -> Self {
        Self {
            handle: NonNull::new_unchecked(handle),
            _owner: std::marker::PhantomData,
        }
    }

    /// # Safety
    /// The node must be owned (not borrowed) and not passed to vapoursynth core
    pub unsafe fn free(self) {
        API::get_cached().free_node(self.as_ptr());
    }

    /// Returns the underlying pointer.
    #[inline]
    pub const fn as_ptr(&self) -> *mut ffi::VSNode {
        self.handle.as_ptr()
    }

    ///Determines the strategy for frame caching. Pass a [CacheMode] constant. Mostly useful for cache debugging since the auto mode should work well in just about all cases.
    ///
    ///Resets the cache to default options when called, discarding [Node::set_cache_options] changes.
    #[inline]
    pub fn set_cache_mode(&self, mode: CacheMode) {
        unsafe { API::get_cached().set_cache_mode(self.as_ptr(), mode as i32) }
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
            API::get_cached().set_cache_options(
                self.as_ptr(),
                fixed_size,
                max_size,
                max_history_size,
            )
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
    pub fn get_frame<'core, 'error>(&self, n: i32) -> Result<Frame<'core>, GetFrameError<'error>> {
        let vi = &self.video_info().unwrap();

        let total = vi.num_frames;
        if n >= total {
            let err_cstring = CString::new("Requested frame number beyond the last one").unwrap();
            return Err(GetFrameError::new(Cow::Owned(err_cstring)));
        }

        // Kinda arbitrary. Same value as used in vsvfw.
        const ERROR_BUF_CAPACITY: usize = 32 * 1024;

        let err_buf = vec![0; ERROR_BUF_CAPACITY];
        let mut err_buf = err_buf.into_boxed_slice();

        let handle = unsafe { API::get_cached().get_frame(n, self.handle.as_ptr(), &mut err_buf) };

        if handle.is_null() {
            // TODO: remove this extra allocation by reusing `Box<[c_char]>`.
            let error = unsafe { CStr::from_ptr(err_buf.as_ptr()) }.to_owned();
            Err(GetFrameError::new(Cow::Owned(error)))
        } else {
            Ok(unsafe { Frame::from_ptr(handle) })
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
    pub fn get_frame_async<'core, F>(&self, n: i32, callback: F)
    where
        F: FnOnce(Result<Frame<'core>, GetFrameError>, i32, Node) + Send + 'core,
    {
        struct CallbackData<'core> {
            callback: Box<dyn CallbackFn<'core> + 'core>,
        }

        // A little bit of magic for Box<FnOnce>.
        trait CallbackFn<'core> {
            fn call(
                self: Box<Self>,
                frame: Result<Frame<'core>, GetFrameError>,
                n: i32,
                node: Node,
            );
        }

        impl<'core, F> CallbackFn<'core> for F
        where
            F: FnOnce(Result<Frame<'core>, GetFrameError>, i32, Node),
        {
            #[allow(clippy::boxed_local)]
            fn call(
                self: Box<Self>,
                frame: Result<Frame<'core>, GetFrameError>,
                n: i32,
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
            let user_data = Box::from_raw(user_data as *mut CallbackData);
            let frame = if frame.is_null() {
                let error_msg = Cow::Borrowed(CStr::from_ptr(error_msg));
                Err(GetFrameError::new(error_msg))
            } else {
                Ok(Frame::from_ptr(frame))
            };

            let node = Node::from_ptr(node);

            user_data.callback.call(frame, n, node);
        }

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
    }

    /// Returns a future that resolves to the frame at the given index `n`.
    pub fn get_frame_future<'core>(
        &self,
        n: i32,
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
            API::get_cached().request_frame_filter(n, self.as_ptr(), frame_ctx.as_ptr());
        }
    }

    /// Get a frame from a node (used in filter's get_frame function)
    #[inline]
    pub fn get_frame_filter<'core>(
        &self,
        n: i32,
        frame_ctx: &FrameContext,
    ) -> Option<Frame<'core>> {
        let ptr =
            unsafe { API::get_cached().get_frame_filter(n, self.as_ptr(), frame_ctx.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { Frame::from_ptr(ptr) })
        }
    }

    #[inline]
    pub fn media_type(&self) -> MediaType {
        let int = unsafe { API::get_cached().get_node_type(self.as_ptr()) };
        match int {
            x if x == ffi::VSMediaType::mtAudio as i32 => MediaType::Audio,
            x if x == ffi::VSMediaType::mtVideo as i32 => MediaType::Video,
            _ => unreachable!(),
        }
    }

    /// Must be called immediately after audio or video filter creation. Returns the upper bound of how many additional frames it is reasonable to pass to [Frame::cache_frame] when trying to make a request more linear.
    pub fn set_linear_filter(&self) -> i32 {
        unsafe { API::get_cached().set_linear_filter(self.as_ptr()) }
    }

    /// By default all requested frames are referenced until a filter’s frame request is done. In extreme cases where a filter needs to reduce 20+ frames into a single output frame it may be beneficial to request these in batches and incrementally process the data instead.
    ///
    ///Should rarely be needed.
    pub fn release_frame_early(&self, n: i32, frame_ctx: &FrameContext) {
        unsafe {
            API::get_cached().release_frame_early(self.as_ptr(), n, frame_ctx.as_ptr());
        }
    }
}

#[cfg(feature = "api-41")]
#[doc(cfg(feature = "api-41"))]
impl Node<'_> {
    /// Clears all cached frames for this node.
    pub fn clear_cache(&self) {
        unsafe {
            API::get_cached().clear_node_cache(self.as_ptr());
        }
    }

    pub fn get_name(&self) -> Option<String> {
        unsafe {
            let ptr = API::get_cached().get_node_name(self.as_ptr());
            if ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
            }
        }
    }

    pub fn get_filter_mode(&self) -> FilterMode {
        unsafe {
            let ptr = API::get_cached().get_node_filter_mode(self.as_ptr());
            ptr.into()
        }
    }

    pub fn get_num_dependencies(&self) -> i32 {
        unsafe { API::get_cached().get_num_node_dependencies(self.as_ptr()) }
    }

    /// Retrieves a dependency of this node.
    pub fn get_dependency(&'_ self, n: i32) -> Option<FilterDependency<'_>> {
        let ptr = unsafe { API::get_cached().get_node_dependency(self.as_ptr(), n) };
        if ptr.is_null() {
            None
        } else {
            unsafe { FilterDependency::from_ptr(ptr) }
        }
    }

    /// Returns an iterator over the dependencies of this node.
    pub fn dependencies(&self) -> FilterDependencies<'_> {
        FilterDependencies {
            node: self,
            index: 0,
            total: self.get_num_dependencies(),
        }
    }

    /// Time spent processing frames in nanoseconds, reset sets the counter to 0 again
    pub fn get_node_processing_time(&self, reset: bool) -> i64 {
        unsafe { API::get_cached().get_node_processing_time(self.as_ptr(), reset as i32) }
    }
}

/// !!! Experimental/expensive graph information, these function require both the major and minor version to match exactly when using them !!!
///
/// These functions only exist to retrieve internal details for debug purposes and graph visualization
/// They will only only work properly when used on a core created with ccfEnableGraphInspection and are
/// not safe to use concurrently with frame requests or other API functions. Because of this they are
/// unsuitable for use in plugins and filters.
///
#[cfg(feature = "graph-api")]
#[doc(cfg(feature = "graph-api"))]
impl Node<'_> {
    pub fn get_creation_function_name(&self, level: i32) -> Option<String> {
        unsafe {
            if API::get_cached().version() != ffi::VAPOURSYNTH_API_VERSION {
                return None;
            }
            let ptr = API::get_cached().get_node_creation_function_name(self.as_ptr(), level);
            if ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
            }
        }
    }

    pub fn get_creation_function_arguments(&'_ self, level: i32) -> Option<Map<'_>> {
        unsafe {
            if API::get_cached().version() != ffi::VAPOURSYNTH_API_VERSION {
                return None;
            }
            let ptr = API::get_cached().get_node_creation_function_arguments(self.as_ptr(), level);
            Some(Map::from_ptr(ptr))
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
    pub const fn as_ffi(&self) -> ffi::VSCacheMode {
        match self {
            CacheMode::Auto => ffi::VSCacheMode::cmAuto,
            CacheMode::ForceDisable => ffi::VSCacheMode::cmForceDisable,
            CacheMode::ForceEnable => ffi::VSCacheMode::cmForceEnable,
        }
    }
}

/// Iterator over the dependencies of a node.
#[cfg(feature = "api-41")]
#[doc(cfg(feature = "api-41"))]
pub struct FilterDependencies<'core> {
    node: &'core Node<'core>,
    index: i32,
    total: i32,
}

#[cfg(feature = "api-41")]
#[doc(cfg(feature = "api-41"))]
impl<'core> Iterator for FilterDependencies<'core> {
    type Item = FilterDependency<'core>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.total {
            None
        } else {
            let dep = self.node.get_dependency(self.index);
            self.index += 1;
            dep
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.total - self.index) as usize;
        (remaining, Some(remaining))
    }
}

#[cfg(feature = "api-41")]
#[doc(cfg(feature = "api-41"))]
impl<'a> ExactSizeIterator for FilterDependencies<'a> {}
