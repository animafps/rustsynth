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
pub(crate) struct ScriptAPI {
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
    pub(crate) fn get() -> Option<Self> {
        // Check if we already have the API.
        let handle = RAW_SCRIPTAPI.load(Ordering::Relaxed);

        let handle = if handle.is_null() {
            // Attempt retrieving it otherwise.
            let handle =
                unsafe { ffi::getVSScriptAPI(ffi::VSSCRIPT_API_VERSION) } as *mut ffi::VSSCRIPTAPI;
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

    #[allow(unused)]
    pub(crate) fn get_api_version(&self) -> i32 {
        unsafe { self.handle.as_ref().getAPIVersion.unwrap()() }
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

    pub(crate) unsafe fn get_alt_output_mode(&self, script: *mut ffi::VSScript, index: i32) -> i32 {
        self.handle.as_ref().getAltOutputMode.unwrap()(script, index)
    }

    pub(crate) unsafe fn eval_set_working_dir(&self, script: *mut ffi::VSScript, set_cwd: i32) {
        self.handle.as_ref().evalSetWorkingDir.unwrap()(script, set_cwd)
    }

    pub(crate) unsafe fn get_core(&self, script: *mut ffi::VSScript) -> *mut ffi::VSCore {
        self.handle.as_ref().getCore.unwrap()(script)
    }

    pub(crate) unsafe fn get_exit_code(&self, script: *mut ffi::VSScript) -> i32 {
        self.handle.as_ref().getExitCode.unwrap()(script)
    }

    #[allow(unused)]
    pub(crate) unsafe fn get_vsapi(&self, version: i32) -> *const ffi::VSAPI {
        self.handle.as_ref().getVSAPI.unwrap()(version)
    }
}

#[cfg(feature = "script-api-42")]
impl ScriptAPI {
    pub fn get_available_output_nodes(
        &self,
        handle: *mut ffi::VSScript,
        size: i32,
        dst: *mut i32,
    ) -> i32 {
        unsafe { self.handle.as_ref().getAvailableOutputNodes.unwrap()(handle, size, dst) }
    }
}

mod errors;
pub use self::errors::{ScriptError, VSScriptError};

mod environment;
pub use self::environment::Environment;

#[cfg(test)]
pub mod tests;
