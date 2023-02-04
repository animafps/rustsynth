use ffi::{VSMap, VSPluginFunction};
use rustsynth_sys as ffi;
use std::{
    ffi::{CStr, CString, NulError},
    marker::PhantomData,
    ops::Deref,
    ptr::{self, NonNull},
};

use crate::{api::API, map::OwnedMap, prelude::Map};

/// A VapourSynth plugin.
///
/// There are a few of these built into the core, and therefore available at all times: the basic filters (identifier `com.vapoursynth.std`, namespace `std`), the resizers (identifier `com.vapoursynth.resize`, namespace `resize`), and the Avisynth compatibility module, if running in Windows (identifier `com.vapoursynth.avisynth`, namespace `avs`).
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

    /// The path to the shared object of the plugin or `None` if is a internal VapourSynth plugin
    pub fn path(&self) -> Option<&'core str> {
        let ptr = unsafe { API::get_cached().get_plugin_path(self.ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_str().unwrap() })
        }
    }

    /// The id associated with the plugin or `None` if it has no id set
    pub fn id(&self) -> Option<&'core str> {
        let ptr = unsafe { API::get_cached().get_plugin_id(self.ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_str().unwrap() })
        }
    }

    /// The namespace associated with the plugin or `None` if it has no namespace set
    pub fn namespace(&self) -> Option<&'core str> {
        let ptr = unsafe { API::get_cached().get_plugin_ns(self.ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_str().unwrap() })
        }
    }

    /// The name associated with the plugin or `None` if it has no name set
    pub fn name(&self) -> Option<&'core str> {
        let ptr = unsafe { API::get_cached().get_plugin_name(self.ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_str().unwrap() })
        }
    }

    pub fn version(&self) -> i32 {
        unsafe { API::get_cached().get_plugin_version(self.ptr()) }
    }

    /// Get function associated with the name
    ///
    /// returns `None` if no function is found
    pub fn function(&'core self, name: &str) -> Option<PluginFunction<'core>> {
        let name_ptr = CString::new(name).unwrap();
        unsafe {
            let ptr = API::get_cached().get_plugin_function_by_name(name_ptr.as_ptr(), self.ptr());
            if ptr.is_null() {
                None
            } else {
                Some(PluginFunction::from_ptr(ptr, self))
            }
        }
    }

    /// Creates an iterator over all the functions of the plugin in an arbitrary order
    pub fn functions(&'core self) -> PluginFunctions {
        PluginFunctions {
            function: None,
            plugin: self,
        }
    }

    fn next_function(
        &'core self,
        function: Option<PluginFunction>,
    ) -> Option<PluginFunction<'core>> {
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
                Some(PluginFunction::from_ptr(ptr, self))
            }
        }
    }
}

/// The iterator over the functions found in a plugin
///
/// created by [Plugin::functions()]
#[derive(Debug, Clone, Copy)]
pub struct PluginFunctions<'core> {
    function: Option<PluginFunction<'core>>,
    plugin: &'core Plugin<'core>,
}

impl<'core> Iterator for PluginFunctions<'core> {
    type Item = PluginFunction<'core>;

    fn next(&mut self) -> Option<Self::Item> {
        self.function = self.plugin.next_function(self.function);
        self.function
    }
}

/// A function of a plugin
#[derive(Debug, Clone, Copy)]
pub struct PluginFunction<'core> {
    handle: NonNull<ffi::VSPluginFunction>,
    _owner: PhantomData<&'core ()>,
    plugin: &'core Plugin<'core>,
}

impl<'core> PluginFunction<'core> {
    pub(crate) unsafe fn from_ptr(
        ptr: *mut VSPluginFunction,
        plugin: &'core Plugin<'core>,
    ) -> Self {
        PluginFunction {
            handle: NonNull::new_unchecked(ptr),
            _owner: PhantomData,
            plugin,
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

    /// Invokes the plugin function
    pub fn invoke(&self, args: &Map<'core>) -> OwnedMap<'core> {
        let name = CString::new(self.name().unwrap()).unwrap();
        unsafe {
            OwnedMap::from_ptr(API::get_cached().invoke(
                self.plugin.handle.as_ptr(),
                name.as_ptr(),
                args.deref(),
            ))
        }
    }
}
