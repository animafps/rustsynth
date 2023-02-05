use crate::{api::API, plugin::Plugin};
use core::fmt;
use rustsynth_sys as ffi;
use std::{ffi::CStr, marker::PhantomData, ptr::NonNull};

/// A reference to a VapourSynth core.
#[derive(Debug, Clone, Copy)]
pub struct CoreRef<'core> {
    handle: NonNull<ffi::VSCore>,
    _owner: PhantomData<&'core ()>,
}

unsafe impl<'core> Send for CoreRef<'core> {}
unsafe impl<'core> Sync for CoreRef<'core> {}

impl<'core> CoreRef<'core> {
    /// Wraps `handle` in a `CoreRef`.
    ///
    /// # Safety
    /// The caller must ensure `handle` is valid and API is cached.
    #[inline]
    pub(crate) unsafe fn from_ptr(handle: *mut ffi::VSCore) -> Self {
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

    /// Returns an instance of `Some(Plugin)` if there exists a plugin loaded associated with the namespace
    ///
    /// None if no plugin is found
    pub fn plugin_by_namespace(&self, namespace: &str) -> Option<Plugin<'core>> {
        unsafe { API::get_cached() }.plugin_by_namespace(namespace, self)
    }

    /// Returns an instance of `Some(Plugin)` if there exists a plugin loaded associated with the id
    ///
    /// None if no plugin is found
    pub fn plugin_by_id(&self, id: &str) -> Option<Plugin<'_>> {
        unsafe { API::get_cached() }.plugin_by_id(id, self)
    }

    /// Returns a iterator over the loaded plugins
    pub fn plugins(&self) -> Plugins<'_> {
        unsafe { API::get_cached() }.plugins(self)
    }

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
}

/// Contains information about a VapourSynth core.
#[derive(Debug, Clone, Copy, Hash)]
pub struct CoreInfo {
    pub version_string: &'static str,
    pub core_version: i32,
    pub api_version: i32,
    pub num_threads: usize,
    pub max_framebuffer_size: u64,
    pub used_framebuffer_size: u64,
}

/// An interator over the loaded plugins
///
/// created by [`CoreRef::plugins()`]
///
/// [`CoreRef::plugins()`]: crate::core::CoreRef::plugins()
#[derive(Debug, Clone, Copy)]
pub struct Plugins<'core> {
    plugin: Option<Plugin<'core>>,
    core: &'core CoreRef<'core>,
}

impl<'core> Plugins<'core> {
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
