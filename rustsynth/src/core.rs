//! A reference to a VapourSynth core and related functionality.
use crate::{
    api::API,
    format::VideoFormat,
    frame::Frame,
    log::{log_handler_callback, LogHandle, LogHandler, MessageType},
    plugin::Plugin,
};
use bitflags::bitflags;
use core::fmt;
use rustsynth_sys as ffi;
use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    ops::Deref,
    ptr::NonNull,
};

#[cfg(test)]
mod tests;

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

/// A reference to a VapourSynth core.
#[derive(Debug, Clone, Copy)]
pub struct CoreRef<'core> {
    handle: NonNull<ffi::VSCore>,
    _owner: PhantomData<&'core ()>,
}

unsafe impl<'core> Send for CoreRef<'core> {}
unsafe impl<'core> Sync for CoreRef<'core> {}

impl<'core> CoreRef<'core> {
    /// Creates and returns a new core.
    ///
    /// Note that there's currently no safe way of freeing the returned core, and the lifetime is
    /// unbounded, because it can live for an arbitrary long time. You may use the (unsafe)
    /// `rustsynth_sys::VSAPI::freeCore()` after ensuring that all frame requests have completed
    /// and all objects belonging to the core have been released.
    ///
    /// # Example
    ///
    /// ```
    /// use rustsynth::core::{CoreCreationFlags, CoreRef};
    /// let core = CoreRef::new(CoreCreationFlags::ENABLE_GRAPH_INSPECTION | CoreCreationFlags::DISABLE_AUTO_LOADING);
    /// ```
    #[inline]
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
    pub unsafe fn from_ptr(handle: *mut ffi::VSCore) -> Self {
        Self {
            handle: NonNull::new_unchecked(handle),
            _owner: PhantomData,
        }
    }

    /// Returns the underlying pointer.
    #[inline]
    pub(crate) fn ptr(&self) -> *mut ffi::VSCore {
        self.handle.as_ptr()
    }

    /// Returns an instance of `CoreInfo`
    ///
    /// # Panics
    ///
    /// Will panic if core configuration is not valid
    pub fn info(&self) -> CoreInfo {
        let core_info = unsafe { API::get_cached().get_core_info(self.ptr()) };
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
    pub fn plugin_by_namespace(&self, namespace: &str) -> Option<Plugin<'core>> {
        unsafe { API::get_cached() }.plugin_by_namespace(namespace, self)
    }

    /// Returns an instance of [Some]<[Plugin]> if there exists a plugin loaded associated with the id
    ///
    /// [None] if no plugin is found
    pub fn plugin_by_id(&self, id: &str) -> Option<Plugin<'_>> {
        unsafe { API::get_cached() }.plugin_by_id(id, self)
    }

    /// Returns a iterator over the loaded plugins
    pub fn plugins(&self) -> Plugins<'_> {
        unsafe { API::get_cached() }.plugins(self)
    }

    /// Sets the number of threads used for processing. Pass 0 to automatically detect. Returns the number of threads that will be used for processing.
    #[inline]
    pub fn set_thread_count(&self, count: usize) -> i32 {
        unsafe { API::get_cached().set_thread_count(self.ptr(), count as i32) }
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
    pub fn set_max_cache_size(&self, size: i64) -> i64 {
        unsafe { API::get_cached().set_max_cache_size(self.ptr(), size) }
    }

    /// The format identifier: one of [crate::format::PresetVideoFormat] or a value gotten from [VideoFormat::query_format_id].
    pub fn get_video_format_by_id(&self, id: u32) -> Option<VideoFormat> {
        let format = unsafe { API::get_cached().get_video_format_by_id(id, self.ptr()) };
        if format.is_none() {
            None
        } else {
            Some(VideoFormat::from_ptr(format.unwrap()))
        }
    }

    /// Duplicates the frame (not just the reference). As the frame buffer is shared in a copy-on-write fashion, the frame content is not really duplicated until a write operation occurs. This is transparent for the user.
    pub fn copy_frame(&'_ self, frame: &Frame) -> Frame<'_> {
        let new_frame = unsafe { API::get_cached().copy_frame(frame, self.ptr()) };
        Frame::from_ptr(new_frame)
    }

    /// Installs a custom handler for the various error messages VapourSynth emits. The message handler is per Core instance. Returns a unique handle.
    /// If no log handler is installed up to a few hundred messages are cached and will be delivered as soon as a log handler is attached. This behavior exists mostly so that warnings when auto-loading plugins (default behavior) won’t disappear
    ///
    /// See the example handler [crate::log::LogRS]
    pub fn add_log_handler(&self, handler: Box<dyn LogHandler>) -> LogHandle {
        let handler_ptr = &handler.deref() as *const &dyn LogHandler as *mut std::ffi::c_void;
        let ptr = unsafe {
            API::get_cached().add_log_handler(
                log_handler_callback,
                handler_ptr,
                self.handle.as_ptr(),
            )
        };
        LogHandle::from_ptr(ptr, handler)
    }

    /// Removes a custom handler.
    pub fn remove_log_handler(&self, handle: LogHandle) -> Result<(), i32> {
        let ret =
            unsafe { API::get_cached().remove_log_handler(handle.as_ptr(), self.handle.as_ptr()) };
        if ret != 0 {
            Ok(())
        } else {
            Err(ret)
        }
    }

    /// Send a message through VapourSynth’s logging framework
    pub fn log_mesage(&self, msg_type: MessageType, msg: &str) {
        let cstr = CString::new(msg).unwrap();
        unsafe {
            API::get_cached().log_message(msg_type.into(), cstr.as_ptr(), self.handle.as_ptr())
        }
    }
}

#[cfg(feature = "api-41")]
#[doc(cfg(feature = "api-41"))]
impl<'core> CoreRef<'core> {
    /// Clears all caches associated with the core.
    pub fn clear_caches(&self) {
        unsafe {
            API::get_cached().clear_core_caches(self.ptr());
        }
    }

    /// Returns true if node timing is enabled.
    pub fn get_node_timing(&self) -> bool {
        if unsafe { API::get_cached().get_core_node_timing(self.ptr()) } > 0 {
            true
        } else {
            false
        }
    }

    /// Note that disabling simply stops the counters from incrementing
    pub fn set_node_timing(&self, enable: bool) {
        unsafe { API::get_cached().set_core_node_timing(self.ptr(), enable as i32) }
    }

    /// Time spent processing frames in nanoseconds in all destroyed nodes, reset sets the counter to 0 again
    pub fn get_freeed_node_processing_time(&self, reset: bool) -> i64 {
        unsafe { API::get_cached().get_freed_node_processing_time(self.ptr(), reset as i32) }
    }
}

/// Contains information about a VapourSynth core.
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
    pub(crate) fn new(core: &'core CoreRef<'core>) -> Self {
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

/// Builder for creating a [CoreRef] with custom options.
pub struct CoreBuilder {
    flags: CoreCreationFlags,
}

impl<'core> CoreBuilder {
    /// Creates a new `CoreBuilder` with default flags.
    pub fn new() -> Self {
        Self {
            flags: CoreCreationFlags::NONE,
        }
    }

    /// Enables graph inspection API functions.
    pub fn with_graph_inspection(mut self) -> Self {
        self.flags |= CoreCreationFlags::ENABLE_GRAPH_INSPECTION;
        self
    }

    /// Disables autoloading of user plugins.
    pub fn disable_auto_loading(mut self) -> Self {
        self.flags |= CoreCreationFlags::DISABLE_AUTO_LOADING;
        self
    }

    /// Disables unloading of plugin libraries when the core is destroyed.
    pub fn disable_library_unloading(mut self) -> Self {
        self.flags |= CoreCreationFlags::DISABLE_LIBRARY_UNLOADING;
        self
    }

    /// Builds and returns a [CoreRef].
    pub fn build(self) -> CoreRef<'core> {
        CoreRef::new(self.flags)
    }
}
