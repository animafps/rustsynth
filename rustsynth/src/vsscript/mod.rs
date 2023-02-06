//! VapourSynth script-related things.

use rustsynth_sys as ffi;
use std::{
    ffi::c_char,
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, Ordering},
};

/// A wrapper for the VapourSynth Script API.
///
///
#[derive(Debug, Clone, Copy)]
pub struct ScriptAPI {
    handle: NonNull<ffi::VSSCRIPTAPI>,
}

unsafe impl Send for ScriptAPI {}
unsafe impl Sync for ScriptAPI {}

/// A cached API pointer. Note that this is `*const ffi::VSSCRIPTAPI`, not `*mut`.
static RAW_SCRIPTAPI: AtomicPtr<ffi::VSSCRIPTAPI> = AtomicPtr::new(ptr::null_mut());

impl ScriptAPI {
    // Creates and retrieves the VapourSynth API.
    ///
    /// Returns `None` on error
    // If we're linking to VSScript anyway, use the VSScript function.
    #[inline]
    pub fn get() -> Option<Self> {
        // Check if we already have the API.
        let handle = RAW_SCRIPTAPI.load(Ordering::Relaxed);

        let handle = if handle.is_null() {
            // Attempt retrieving it otherwise.
            let handle = unsafe { ffi::getVSScriptAPI(ffi::VSSCRIPT_API_MAJOR.try_into().unwrap()) }
                as *mut ffi::VSSCRIPTAPI;

            if !handle.is_null() {
                // If we successfully retrieved the API, cache it.
                RAW_SCRIPTAPI.store(handle, Ordering::Relaxed);
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
            handle: NonNull::new_unchecked(RAW_SCRIPTAPI.load(Ordering::Relaxed)),
        }
    }

    pub(crate) unsafe fn free_script(&self, script: *mut ffi::VSScript) {
        self.handle.as_ref().freeScript.unwrap()(script)
    }

    pub(crate) unsafe fn get_error(&self, script: *mut ffi::VSScript) -> *const c_char {
        self.handle.as_ref().getError.unwrap()(script)
    }

    pub(crate) unsafe fn create_script(&self, core: *mut ffi::VSCore) -> *mut ffi::VSScript {
        self.handle.as_ref().createScript.unwrap()(core)
    }

    pub(crate) unsafe fn eval_buffer(
        &self,
        script: *mut ffi::VSScript,
        buffer: *const c_char,
        file_name: *const c_char,
    ) -> i32 {
        self.handle.as_ref().evaluateBuffer.unwrap()(script, buffer, file_name)
    }

    pub(crate) unsafe fn get_variable(
        &self,
        script: *mut ffi::VSScript,
        name: *const c_char,
        map: *mut ffi::VSMap,
    ) -> i32 {
        self.handle.as_ref().getVariable.unwrap()(script, name, map)
    }

    pub(crate) unsafe fn set_variables(
        &self,
        script: *mut ffi::VSScript,
        map: *const ffi::VSMap,
    ) -> i32 {
        self.handle.as_ref().setVariables.unwrap()(script, map)
    }

    pub(crate) unsafe fn get_output(
        &self,
        script: *mut ffi::VSScript,
        index: i32,
    ) -> *mut ffi::VSNode {
        self.handle.as_ref().getOutputNode.unwrap()(script, index)
    }
    pub(crate) unsafe fn get_output_alpha(
        &self,
        script: *mut ffi::VSScript,
        index: i32,
    ) -> *mut ffi::VSNode {
        self.handle.as_ref().getOutputAlphaNode.unwrap()(script, index)
    }
}

mod errors;
pub use self::errors::{Error, VSScriptError};

mod environment;
pub use self::environment::Environment;
