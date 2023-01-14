use rustsynth_sys as ffi;
use std::{
    ffi::c_char,
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{core::CoreRef, plugin::Plugin};

/// A wrapper for the VapourSynth API.
#[derive(Debug, Clone, Copy)]
pub struct API {
    // Note that this is *const, not *mut.
    handle: NonNull<ffi::VSAPI>,
}

unsafe impl Send for API {}
unsafe impl Sync for API {}

/// A cached API pointer. Note that this is `*const ffi::VSAPI`, not `*mut`.
static RAW_API: AtomicPtr<ffi::VSAPI> = AtomicPtr::new(ptr::null_mut());

impl API {
    /// Retrieves the VapourSynth API.
    ///
    /// Returns `None` on error
    // If we're linking to VSScript anyway, use the VSScript function.
    #[inline]
    pub fn get() -> Option<Self> {
        // Check if we already have the API.
        let handle = RAW_API.load(Ordering::Relaxed);

        let handle = if handle.is_null() {
            // Attempt retrieving it otherwise.
            let handle = unsafe { ffi::getVapourSynthAPI(40) } as *mut ffi::VSAPI;

            if !handle.is_null() {
                // If we successfully retrieved the API, cache it.
                RAW_API.store(handle, Ordering::Relaxed);
            }
            handle
        } else {
            handle
        };

        if handle.is_null() {
            None
        } else {
            Some(Self {
                handle: unsafe { NonNull::new_unchecked(handle) },
            })
        }
    }

    /// Returns the cached API.
    ///
    /// # Safety
    /// This function assumes the cache contains a valid API pointer.
    #[inline]
    pub(crate) unsafe fn get_cached() -> Self {
        Self {
            handle: NonNull::new_unchecked(RAW_API.load(Ordering::Relaxed)),
        }
    }

    /// Creates and returns a new core.
    ///
    /// Note that there's currently no safe way of freeing the returned core, and the lifetime is
    /// unbounded, because it can live for an arbitrary long time. You may use the (unsafe)
    /// `rustsynth_sys::VSAPI::freeCore()` after ensuring that all frame requests have completed
    /// and all objects belonging to the core have been released.
    #[inline]
    pub fn create_core<'core>(self, flags: i32) -> CoreRef<'core> {
        unsafe {
            let handle = (self.handle.as_ref().createCore).unwrap()(flags);
            CoreRef::from_ptr(handle)
        }
    }

    pub fn get_next_plugin<'core>(self, plugin: Option<Plugin>, core: CoreRef) -> Plugin<'core> {
        unsafe {
            let pluginptr = if plugin.is_none() {
                ptr::null_mut()
            } else {
                plugin.unwrap().ptr()
            };
            let handle = self.handle.as_ref().getNextPlugin.unwrap()(pluginptr, core.ptr());
            Plugin::from_ptr(handle)
        }
    }

    pub(crate) unsafe fn get_plugin_path(self, plugin: *mut ffi::VSPlugin) -> *const c_char {
        self.handle.as_ref().getPluginPath.unwrap()(plugin)
    }
}
