use crate::api::API;
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
