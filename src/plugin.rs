use ffi::VSPluginFunction;
use rustsynth_sys as ffi;
use std::{
    ffi::{c_int, CStr, CString},
    marker::PhantomData,
    ptr::{self, NonNull},
};

use crate::api::API;

/// A VapourSynth plugin.
#[derive(Debug, Clone, Copy)]
pub struct Plugin<'core> {
    handle: NonNull<ffi::VSPlugin>,
    _owner: PhantomData<&'core ()>,
}

unsafe impl<'core> Send for Plugin<'core> {}
unsafe impl<'core> Sync for Plugin<'core> {}

impl<'core> Plugin<'core> {
    #[inline]
    pub(crate) unsafe fn from_ptr(ptr: *mut ffi::VSPlugin) -> Self {
        Plugin {
            handle: NonNull::new_unchecked(ptr),
            _owner: PhantomData,
        }
    }

    /// Returns the underlying pointer.
    #[inline]
    pub(crate) fn ptr(&self) -> *mut ffi::VSPlugin {
        self.handle.as_ptr()
    }

    pub fn path(&self) -> Option<&'core str> {
        let ptr = unsafe { API::get_cached().get_plugin_path(self.ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_str().unwrap() })
        }
    }

    pub fn id(&self) -> Option<&'core str> {
        let ptr = unsafe { API::get_cached().get_plugin_id(self.ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_str().unwrap() })
        }
    }

    pub fn namespace(&self) -> Option<&'core str> {
        let ptr = unsafe { API::get_cached().get_plugin_ns(self.ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_str().unwrap() })
        }
    }

    pub fn name(&self) -> Option<&'core str> {
        let ptr = unsafe { API::get_cached().get_plugin_name(self.ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_str().unwrap() })
        }
    }

    pub fn version(&self) -> c_int {
        unsafe { API::get_cached().get_plugin_version(self.ptr()) }
    }

    pub fn function(&self, name: &str) -> Option<PluginFunction<'core>> {
        let name_ptr = CString::new(name).unwrap();
        unsafe {
            let ptr = API::get_cached().get_plugin_function_by_name(name_ptr.as_ptr(), self.ptr());
            if ptr.is_null() {
                None
            } else {
                Some(PluginFunction::from_ptr(ptr))
            }
        }
    }

    pub fn next_function(&self, function: Option<PluginFunction>) -> Option<PluginFunction<'core>> {
        unsafe {
            let function = if let Some(value) = function {
                value.ptr()
            } else {
                ptr::null_mut()
            };
            let ptr = API::get_cached().get_next_plugin_function(function, self.ptr());
            if ptr.is_null() {
                None
            } else {
                Some(PluginFunction::from_ptr(ptr))
            }
        }
    }
}

pub struct PluginFunction<'core> {
    handle: NonNull<ffi::VSPluginFunction>,
    _owner: PhantomData<&'core ()>,
}

impl<'core> PluginFunction<'core> {
    pub(crate) unsafe fn from_ptr(ptr: *mut VSPluginFunction) -> Self {
        PluginFunction {
            handle: NonNull::new_unchecked(ptr),
            _owner: PhantomData,
        }
    }

    /// Returns the underlying pointer.
    #[inline]
    pub(crate) fn ptr(&self) -> *mut ffi::VSPluginFunction {
        self.handle.as_ptr()
    }

    pub fn name(&self) -> Option<&'core str> {
        let ptr = unsafe { API::get_cached().get_plugin_function_name(self.ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_str().unwrap() })
        }
    }

    pub fn arguments(&self) -> Option<&'core str> {
        let ptr = unsafe { API::get_cached().get_plugin_function_arguments(self.ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_str().unwrap() })
        }
    }
}
