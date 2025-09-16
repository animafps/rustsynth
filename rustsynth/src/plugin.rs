//! Interface for VapourSynth plugins and their functions.
use bitflags::bitflags;
use ffi::VSPluginFunction;
use rustsynth_sys::{self as ffi, VSPluginConfigFlags};
use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    ops::Deref,
    ptr::{self, NonNull},
};

use crate::{api::API, map::Map, map::OwnedMap};

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
    pub unsafe fn from_ptr(ptr: *mut ffi::VSPlugin) -> Self {
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
    pub fn path(&self) -> Option<String> {
        let ptr = unsafe { API::get_cached().get_plugin_path(self.ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() })
        }
    }

    /// The id associated with the plugin or `None` if it has no id set
    pub fn id(&self) -> Option<String> {
        let ptr = unsafe { API::get_cached().get_plugin_id(self.ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() })
        }
    }

    /// The namespace associated with the plugin or `None` if it has no namespace set
    pub fn namespace(&self) -> Option<String> {
        let ptr = unsafe { API::get_cached().get_plugin_ns(self.ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() })
        }
    }

    /// The name associated with the plugin or `None` if it has no name set
    pub fn name(&self) -> Option<String> {
        let ptr = unsafe { API::get_cached().get_plugin_name(self.ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() })
        }
    }

    #[inline]
    pub fn version(&self) -> i32 {
        unsafe { API::get_cached().get_plugin_version(self.ptr()) }
    }

    /// Get function struct associated with the name
    ///
    /// returns `None` if no function is found
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

    /// Creates an iterator over all the functions of the plugin in an arbitrary order
    pub fn functions(&'_ self) -> PluginFunctions<'_> {
        PluginFunctions {
            function: None,
            plugin: self,
        }
    }

    fn next_function(&self, function: Option<PluginFunction>) -> Option<PluginFunction<'core>> {
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

    /// Invokes the plugin function with the name provided
    ///
    /// # Panics
    ///
    /// Will panic if there is no function with that name
    pub fn invoke(&self, name: &str, args: &Map<'core>) -> OwnedMap<'core> {
        self.function(name).expect("No Plugin found");
        let name = CString::new(name).unwrap();
        unsafe {
            OwnedMap::from_ptr(API::get_cached().invoke(
                self.handle.as_ptr(),
                name.as_ptr(),
                args.deref(),
            ))
        }
    }
}

bitflags! {
    pub struct PluginConfigFlags: i32 {
        /// Allow functions to be added to the plugin object after the plugin loading phase. Mostly useful for Avisynth compatibility and other foreign plugin loaders.
        const MODIFIABLE = 1;
        const NONE  = 0;
    }
}

impl PluginConfigFlags {
    pub fn as_ptr(&self) -> ffi::VSPluginConfigFlags {
        VSPluginConfigFlags(self.bits.try_into().unwrap())
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
    ptr: NonNull<ffi::VSPluginFunction>,
    pub name: Option<&'core str>,
    pub arguments: Option<&'core str>,
}

impl<'core> PluginFunction<'core> {
    pub(crate) unsafe fn from_ptr(ptr: *mut VSPluginFunction) -> Self {
        let name_ptr = unsafe { API::get_cached().get_plugin_function_name(ptr) };
        let name = if name_ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(name_ptr).to_str().unwrap() })
        };

        let arg_ptr = unsafe { API::get_cached().get_plugin_function_arguments(ptr) };
        let arguments = if arg_ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(arg_ptr).to_str().unwrap() })
        };
        PluginFunction {
            ptr: NonNull::new_unchecked(ptr),
            name,
            arguments,
        }
    }

    fn ptr(&self) -> *mut ffi::VSPluginFunction {
        self.ptr.as_ptr()
    }
}
