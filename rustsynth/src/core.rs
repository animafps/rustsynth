//! A reference to a `VapourSynth` core and related functionality.
use crate::{
    api::API,
    filter::Filter,
    format::VideoFormat,
    frame::{Frame, FrameContext},
    log::{log_handler_callback, LogHandle, LogHandler, MessageType},
    map::{Map, MapError},
    node::Node,
    plugin::Plugin,
};
use bitflags::bitflags;
use rustsynth_sys as ffi;
use std::fmt;
use std::{
    ffi::{CStr, CString, NulError},
    marker::PhantomData,
    ptr::NonNull,
};
use thiserror::Error;

#[cfg(test)]
mod tests;

/// The error type for `Core` operations.
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Failed to create map: {0}")]
    MapCreationFailed(#[from] MapError),
    #[error("Invalid filter name: {0}")]
    InvalidFilterName(#[from] NulError),
    #[error("Failed to create video filter")]
    VideoFilterCreationFailed,
    #[error("Failed to create audio filter")]
    AudioFilterCreationFailed,
    #[error("{0}")]
    Custom(String),
}

/// A specialized `Result` type for `Core` operations.
pub type CoreResult<T> = Result<T, CoreError>;

bitflags! {
    /// Options when creating a core.
    pub struct CoreCreationFlags: u32 {
        /// No flags.
        const NONE = 0b00000000;
        /// Required to use the graph inspection api functions. Increases memory usage due to the extra information stored.
        const ENABLE_GRAPH_INSPECTION = 0b00000001;
        /// Don’t autoload any user plugins. Core plugins are always loaded.
        const DISABLE_AUTO_LOADING = 0b00000010;
        /// Don’t unload plugin libraries when the core is destroyed. Due to a small amount of memory leaking every load and unload (windows feature, not my fault) of a library this may help in applications with extreme amount of script reloading.
        const DISABLE_LIBRARY_UNLOADING = 0b00000100;
    }
}

/// A reference to a `VapourSynth` core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreRef<'core> {
    handle: NonNull<ffi::VSCore>,
    _owner: PhantomData<&'core ()>,
}

unsafe impl Send for CoreRef<'_> {}
unsafe impl Sync for CoreRef<'_> {}

impl<'core> CoreRef<'core> {
    /// Creates and returns a new core.
    ///
    /// # Example
    ///
    /// ```
    /// use rustsynth::core::{CoreCreationFlags, CoreRef};
    /// let core = CoreRef::new(CoreCreationFlags::ENABLE_GRAPH_INSPECTION | CoreCreationFlags::DISABLE_AUTO_LOADING);
    /// ```
    #[inline]
    #[must_use] 
    pub fn new(flags: CoreCreationFlags) -> Self {
        let api = API::get().unwrap();
        unsafe {
            let handle = api.create_core(flags.bits() as i32);
            Self::from_ptr(handle)
        }
    }
    /// Wraps `handle` in a `CoreRef`.
    ///
    /// # Safety
    /// The caller must ensure `handle` is valid and API is cached.
    #[inline]
    pub const unsafe fn from_ptr(handle: *mut ffi::VSCore) -> Self {
        Self {
            handle: NonNull::new_unchecked(handle),
            _owner: PhantomData,
        }
    }

    /// Returns the underlying pointer.
    #[inline]
    #[must_use] 
    pub const fn as_ptr(&self) -> *mut ffi::VSCore {
        self.handle.as_ptr()
    }

    /// Returns an instance of `CoreInfo`
    ///
    /// # Panics
    ///
    /// Will panic if core configuration is not valid
    #[must_use] 
    pub fn info(&self) -> CoreInfo {
        let core_info = unsafe { API::get_cached().get_core_info(self.as_ptr()) };
        let version_string = unsafe { CStr::from_ptr(core_info.versionString).to_str().unwrap() };
        debug_assert!(core_info.numThreads >= 0);
        debug_assert!(core_info.maxFramebufferSize >= 0);
        debug_assert!(core_info.usedFramebufferSize >= 0);

        CoreInfo {
            version_string,
            core_version: core_info.core,
            api_version: core_info.api,
            num_threads: core_info.numThreads as usize,
            max_framebuffer_size: core_info.maxFramebufferSize as u64,
            used_framebuffer_size: core_info.usedFramebufferSize as u64,
        }
    }

    /// Returns an instance of [Some]<[Plugin]> if there exists a plugin loaded associated with the namespace
    ///
    /// [None] if no plugin is found
    #[must_use] 
    pub fn plugin_by_namespace(&self, namespace: &str) -> Option<Plugin<'core>> {
        let namespace = CString::new(namespace).unwrap();
        unsafe { API::get_cached() }.plugin_by_namespace(namespace.as_ptr(), self)
    }

    /// Returns an instance of [Some]<[Plugin]> if there exists a plugin loaded associated with the id
    ///
    /// [None] if no plugin is found
    #[must_use] 
    pub fn plugin_by_id(&self, id: &str) -> Option<Plugin<'_>> {
        let id = CString::new(id).unwrap();
        unsafe { API::get_cached() }.plugin_by_id(id.as_ptr(), self)
    }

    #[must_use] 
    pub fn std(&self) -> Option<Plugin<'_>> {
        unsafe {
            API::get_cached().plugin_by_id(ffi::VSH_STD_PLUGIN_ID.as_ptr().cast::<i8>(), self)
        }
    }

    #[must_use] 
    pub fn resize(&self) -> Option<Plugin<'_>> {
        unsafe {
            API::get_cached().plugin_by_id(ffi::VSH_RESIZE_PLUGIN_ID.as_ptr().cast::<i8>(), self)
        }
    }

    #[must_use] 
    pub fn text(&self) -> Option<Plugin<'_>> {
        unsafe {
            API::get_cached().plugin_by_id(ffi::VSH_TEXT_PLUGIN_ID.as_ptr().cast::<i8>(), self)
        }
    }

    /// Returns a iterator over the loaded plugins
    #[must_use] 
    pub fn plugins(&self) -> Plugins<'_> {
        unsafe { API::get_cached() }.plugins(self)
    }

    /// Sets the number of threads used for processing. Pass 0 to automatically detect. Returns the number of threads that will be used for processing.
    #[inline]
    #[must_use] 
    pub fn set_thread_count(&self, count: usize) -> i32 {
        unsafe { API::get_cached().set_thread_count(self.as_ptr(), count as i32) }
    }

    /// Consumes and frees the core and core reference
    ///
    /// # Safety
    ///
    /// Must ensure that all frame requests have completed and all objects belonging to the core have been released.
    pub unsafe fn free_core(self) {
        API::get_cached().free_core(self.handle.as_ptr());
    }

    /// Sets the maximum size of the framebuffer cache. Returns the new maximum size.
    #[must_use] 
    pub fn set_max_cache_size(&self, size: i64) -> i64 {
        unsafe { API::get_cached().set_max_cache_size(self.as_ptr(), size) }
    }

    /// The format identifier: one of [`crate::format::PresetVideoFormat`] or a value gotten from [`VideoFormat::query_format_id`].
    #[must_use] 
    pub fn get_video_format_by_id(&self, id: u32) -> Option<VideoFormat> {
        let format = unsafe { API::get_cached().get_video_format_by_id(id, self.as_ptr()) };
        format.map(|f| unsafe { VideoFormat::from_ptr(f) })
    }

    /// Duplicates the frame (not just the reference). As the frame buffer is shared in a copy-on-write fashion, the frame content is not really duplicated until a write operation occurs. This is transparent for the user.
    #[must_use] 
    pub fn copy_frame(&'_ self, frame: &Frame) -> Frame<'_> {
        let new_frame = unsafe { API::get_cached().copy_frame(frame, self.as_ptr()) };
        unsafe { Frame::from_ptr(new_frame) }
    }

    /// Installs a custom handler for the various error messages `VapourSynth` emits. The message handler is per Core instance. Returns a unique handle.
    /// If no log handler is installed up to a few hundred messages are cached and will be delivered as soon as a log handler is attached. This behavior exists mostly so that warnings when auto-loading plugins (default behavior) won’t disappear
    ///
    /// See the example handler [`crate::log::LogRS`]
    pub fn add_log_handler<H: LogHandler>(&self, handler: H) -> LogHandle<H> {
        let handler_ptr = &raw const handler as *mut std::ffi::c_void;
        let ptr = unsafe {
            API::get_cached().add_log_handler(
                log_handler_callback,
                handler_ptr,
                self.handle.as_ptr(),
            )
        };
        unsafe { LogHandle::from_ptr(ptr, handler) }
    }

    /// Removes a custom handler.
    pub fn remove_log_handler<H: LogHandler>(&self, handle: LogHandle<H>) -> Result<(), i32> {
        let ret =
            unsafe { API::get_cached().remove_log_handler(handle.as_ptr(), self.handle.as_ptr()) };
        if ret != 0 {
            Ok(())
        } else {
            Err(ret)
        }
    }

    /// Send a message through `VapourSynth`'s logging framework
    pub fn log_mesage(&self, msg_type: MessageType, msg: &str) {
        let cstr = CString::new(msg).unwrap();
        unsafe {
            API::get_cached().log_message(msg_type.into(), cstr.as_ptr(), self.handle.as_ptr());
        }
    }

    /// Create a video filter using the Filter trait
    pub fn create_video_filter<F>(&self, filter: &F) -> CoreResult<Map<'_>>
    where
        F: Filter<'core>,
    {
        let out = Map::new()?;
        // Get video info from the filter
        let video_info = filter.get_video_info().map_err(CoreError::Custom)?;
        let dependencies = filter.get_dependencies();

        // Convert dependencies to FFI format
        let deps_ffi: Vec<ffi::VSFilterDependency> =
            dependencies.iter().map(super::filter::FilterDependency::as_ffi).collect();

        // Box the filter instance for storage
        let filter_box = Box::new(filter);
        let instance_data = Box::into_raw(filter_box).cast::<std::ffi::c_void>();

        // Create C strings for name
        let name_cstr = CString::new(F::NAME)?;

        unsafe {
            API::get_cached().create_video_filter(
                out.as_ptr(),
                name_cstr.as_ptr(),
                &video_info.as_ffi(),
                Some(filter_get_frame::<F>),
                Some(filter_free::<F>),
                std::ptr::from_ref(&F::MODE.as_ffi()) as i32,
                deps_ffi.as_ptr(),
                deps_ffi.len() as i32,
                instance_data,
                self.as_ptr(),
            );
        }

        Ok(out)
    }

    /// Create a video filter using the Filter trait (returns node directly)
    pub fn create_video_filter2<F>(&self, filter: &F) -> CoreResult<crate::node::Node<'core>>
    where
        F: Filter<'core>,
    {
        // Get video info from the filter
        let video_info = filter.get_video_info().map_err(CoreError::Custom)?;
        let dependencies = filter.get_dependencies();

        // Convert dependencies to FFI format
        let deps_ffi: Vec<ffi::VSFilterDependency> =
            dependencies.iter().map(super::filter::FilterDependency::as_ffi).collect();

        // Box the filter instance for storage
        let filter_box = Box::new(filter);
        let instance_data = Box::into_raw(filter_box).cast::<std::ffi::c_void>();

        // Create C strings for name
        let name_cstr = CString::new(F::NAME)?;

        let node_ptr = unsafe {
            API::get_cached().create_video_filter2(
                name_cstr.as_ptr(),
                &video_info.as_ffi(),
                Some(filter_get_frame::<F>),
                Some(filter_free::<F>),
                std::ptr::from_ref(&F::MODE.as_ffi()) as i32,
                deps_ffi.as_ptr(),
                deps_ffi.len() as i32,
                instance_data,
                self.as_ptr(),
            )
        };

        if node_ptr.is_null() {
            return Err(CoreError::VideoFilterCreationFailed);
        }

        Ok(unsafe { crate::node::Node::from_ptr(node_ptr) })
    }

    /// Create a audio filter using the Filter trait
    pub fn create_audio_filter<F>(&self, filter: &F) -> CoreResult<Map<'_>>
    where
        F: Filter<'core>,
    {
        let out = Map::new()?;
        // Get audio info from the filter
        let audio_info = filter.get_audio_info().map_err(CoreError::Custom)?;
        let dependencies = filter.get_dependencies();

        // Convert dependencies to FFI format
        let deps_ffi: Vec<ffi::VSFilterDependency> =
            dependencies.iter().map(super::filter::FilterDependency::as_ffi).collect();

        // Box the filter instance for storage
        let filter_box = Box::new(filter);
        let instance_data = Box::into_raw(filter_box).cast::<std::ffi::c_void>();

        // Create C strings for name
        let name_cstr = CString::new(F::NAME)?;

        unsafe {
            API::get_cached().create_audio_filter(
                out.as_ptr(),
                name_cstr.as_ptr(),
                &audio_info.as_ffi(),
                Some(filter_get_frame::<F>),
                Some(filter_free::<F>),
                std::ptr::from_ref(&F::MODE.as_ffi()) as i32,
                deps_ffi.as_ptr(),
                deps_ffi.len() as i32,
                instance_data,
                self.as_ptr(),
            );
        }

        Ok(out)
    }

    /// Create an audio filter using the Filter trait (returns node directly)
    pub fn create_audio_filter2<F>(&self, filter: &F) -> CoreResult<Node<'core>>
    where
        F: Filter<'core>,
    {
        // Get audio info from the filter
        let audio_info = filter.get_audio_info().map_err(CoreError::Custom)?;
        let dependencies = filter.get_dependencies();

        // Convert dependencies to FFI format
        let deps_ffi: Vec<ffi::VSFilterDependency> =
            dependencies.iter().map(super::filter::FilterDependency::as_ffi).collect();

        // Box the filter instance for storage
        let filter_box = Box::new(filter);
        let instance_data = Box::into_raw(filter_box).cast::<std::ffi::c_void>();

        // Create C strings for name
        let name_cstr = CString::new(F::NAME)?;

        let node_ptr = unsafe {
            API::get_cached().create_audio_filter2(
                name_cstr.as_ptr(),
                std::ptr::from_ref(&audio_info.as_ffi()),
                Some(filter_get_frame::<F>),
                Some(filter_free::<F>),
                std::ptr::from_ref(&F::MODE.as_ffi()) as i32,
                deps_ffi.as_ptr(),
                deps_ffi.len() as i32,
                instance_data,
                self.as_ptr(),
            )
        };

        if node_ptr.is_null() {
            return Err(CoreError::AudioFilterCreationFailed);
        }

        Ok(unsafe { crate::node::Node::from_ptr(node_ptr) })
    }
}

// Callback functions for Filter trait integration
unsafe extern "C" fn filter_get_frame<'core, F>(
    n: i32,
    activation_reason: i32,
    instance_data: *mut std::ffi::c_void,
    frame_data: *mut *mut std::ffi::c_void,
    frame_ctx: *mut ffi::VSFrameContext,
    core: *mut ffi::VSCore,
    _vs_api: *const ffi::VSAPI,
) -> *const ffi::VSFrame
where
    F: Filter<'core>,
{
    if instance_data.is_null() || frame_ctx.is_null() || core.is_null() {
        return std::ptr::null();
    }

    let filter = &mut *instance_data.cast::<F>();
    let frame_context = FrameContext::from_ptr(frame_ctx);
    let core_ref = CoreRef::from_ptr(core);

    let activation = crate::filter::ActivationReason::from_ffi(activation_reason);

    match activation {
        crate::filter::ActivationReason::Initial => {
            // Request input frames
            filter.request_input_frames(n, &frame_context);
            std::ptr::null()
        }
        crate::filter::ActivationReason::AllFramesReady => {
            // Process the frame
            let frame_data_array = if frame_data.is_null() {
                [0u8; 4]
            } else {
                // Convert the frame_data pointer to [u8; 4]
                let ptr = *frame_data as *const u8;
                if ptr.is_null() {
                    [0u8; 4]
                } else {
                    std::ptr::read(ptr.cast::<[u8; 4]>())
                }
            };

            match filter.process_frame(n, &frame_data_array, &frame_context, core_ref) {
                Ok(frame) => frame.as_ptr(),
                Err(error) => {
                    frame_context.set_filter_error(&error);
                    std::ptr::null()
                }
            }
        }
        crate::filter::ActivationReason::Error => {
            // Handle error case - cleanup frame data if needed
            if !frame_data.is_null() {
                let frame_data_array = if frame_data.is_null() {
                    [0u8; 4]
                } else {
                    let ptr = *frame_data as *const u8;
                    if ptr.is_null() {
                        [0u8; 4]
                    } else {
                        std::ptr::read(ptr.cast::<[u8; 4]>())
                    }
                };
                filter.cleanup_frame_data(&frame_data_array);
            }
            std::ptr::null()
        }
    }
}

unsafe extern "C" fn filter_free<'core, F>(
    instance_data: *mut std::ffi::c_void,
    _core: *mut ffi::VSCore,
    _vs_api: *const ffi::VSAPI,
) where
    F: Filter<'core>,
{
    if !instance_data.is_null() {
        let filter = Box::from_raw(instance_data.cast::<F>());
        filter.cleanup();
        // Box is automatically dropped here
    }
}

#[cfg(feature = "api-41")]
#[doc(cfg(feature = "api-41"))]
impl CoreRef<'_> {
    /// Clears all caches associated with the core.
    pub fn clear_caches(&self) {
        unsafe {
            API::get_cached().clear_core_caches(self.as_ptr());
        }
    }

    /// Returns true if node timing is enabled.
    #[must_use] 
    pub fn get_node_timing(&self) -> bool {
        (unsafe { API::get_cached().get_core_node_timing(self.as_ptr()) } > 0)
    }

    /// Note that disabling simply stops the counters from incrementing
    pub fn set_node_timing(&self, enable: bool) {
        unsafe { API::get_cached().set_core_node_timing(self.as_ptr(), i32::from(enable)) }
    }

    /// Time spent processing frames in nanoseconds in all destroyed nodes, reset sets the counter to 0 again
    pub fn get_freed_node_processing_time(&self, reset: bool) -> i64 {
        unsafe { API::get_cached().get_freed_node_processing_time(self.as_ptr(), i32::from(reset)) }
    }
}

/// Contains information about a `VapourSynth` core.
#[derive(Debug, Clone, Copy, Hash)]
pub struct CoreInfo {
    pub version_string: &'static str,
    pub core_version: i32,
    pub api_version: i32,
    /// Number of worker threads.
    pub num_threads: usize,
    /// Maximum size of the framebuffer cache in bytes.
    pub max_framebuffer_size: u64,
    /// Current size of the framebuffer cache in bytes.
    pub used_framebuffer_size: u64,
}

/// An interator over the loaded plugins
///
/// created by [`CoreRef::plugins()`]
#[derive(Debug, Clone, Copy)]
pub struct Plugins<'core> {
    plugin: Option<Plugin<'core>>,
    core: &'core CoreRef<'core>,
}

impl<'core> Plugins<'core> {
    #[inline]
    pub(crate) const fn new(core: &'core CoreRef<'core>) -> Self {
        Plugins { plugin: None, core }
    }
}

impl<'core> Iterator for Plugins<'core> {
    type Item = Plugin<'core>;

    fn next(&mut self) -> Option<Self::Item> {
        self.plugin = unsafe { API::get_cached().next_plugin(self.plugin, self.core) };
        self.plugin
    }
}

impl fmt::Display for CoreInfo {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.version_string)?;
        writeln!(f, "Worker threads: {}", self.num_threads)?;
        writeln!(
            f,
            "Max framebuffer cache size: {}",
            self.max_framebuffer_size
        )?;
        writeln!(
            f,
            "Current framebuffer cache size: {}",
            self.used_framebuffer_size
        )
    }
}

/// Builder for creating a [`CoreRef`] with custom options.
pub struct CoreBuilder {
    flags: CoreCreationFlags,
}

impl<'core> CoreBuilder {
    /// Creates a new `CoreBuilder` with default flags.
    #[must_use] 
    pub const fn new() -> Self {
        Self {
            flags: CoreCreationFlags::NONE,
        }
    }

    /// Enables graph inspection API functions.
    #[must_use] 
    pub fn with_graph_inspection(mut self) -> Self {
        self.flags |= CoreCreationFlags::ENABLE_GRAPH_INSPECTION;
        self
    }

    /// Disables autoloading of user plugins.
    #[must_use] 
    pub fn disable_auto_loading(mut self) -> Self {
        self.flags |= CoreCreationFlags::DISABLE_AUTO_LOADING;
        self
    }

    /// Disables unloading of plugin libraries when the core is destroyed.
    #[must_use] 
    pub fn disable_library_unloading(mut self) -> Self {
        self.flags |= CoreCreationFlags::DISABLE_LIBRARY_UNLOADING;
        self
    }

    /// Builds and returns a [`CoreRef`].
    #[must_use] 
    pub fn build(self) -> CoreRef<'core> {
        CoreRef::new(self.flags)
    }
}

impl Default for CoreBuilder {
    fn default() -> Self {
        Self::new()
    }
}
