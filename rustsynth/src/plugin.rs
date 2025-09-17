//! Interface for VapourSynth plugins and their functions.
use bitflags::bitflags;
use ffi::VSPluginFunction;
use rustsynth_sys::{self as ffi, VSPluginConfigFlags};
use std::{
    ffi::{c_void, CStr, CString},
    marker::PhantomData,
    ops::Deref,
    ptr::{self, NonNull},
};

use crate::{
    api::API,
    core::CoreRef,
    map::{Map, MapRef, OwnedMap},
};

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
    pub fn function(&self, name: &str) -> Option<PluginFunction<'_>> {
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
    pub fn functions(&'_ self) -> PluginFunctions<'_> {
        PluginFunctions {
            function: None,
            plugin: self,
        }
    }

    fn next_function(&self, function: Option<PluginFunction<'_>>) -> Option<PluginFunction<'_>> {
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

    /// Function that registers a filter exported by the plugin. A plugin can export any number of filters. This function may only be called during the plugin loading phase unless the [PluginConfigFlags::MODIFIABLE] flag was set.
    pub fn register_function(
        &self,
        name: &str,
        args: &str,
        ret_type: &str,
        func: PublicFunction,
    ) -> Result<(), ()> {
        let name_c = CString::new(name).unwrap();
        let args_c = CString::new(args).unwrap();
        let ret_type_c = CString::new(ret_type).unwrap();
        let user_data: Box<PublicFunction> = Box::new(func);
        let user_data_ptr = Box::into_raw(user_data) as *mut c_void;
        let res = unsafe {
            API::get_cached().register_function(
                name_c.as_ptr(),
                args_c.as_ptr(),
                ret_type_c.as_ptr(),
                Some(public_function),
                user_data_ptr,
                self.handle.as_ptr(),
            )
        };
        if res == 0 {
            Ok(())
        } else {
            Err(())
        }
    }
}

unsafe extern "C" fn public_function(
    in_map: *const ffi::VSMap,
    out_map: *mut ffi::VSMap,
    user_data: *mut c_void,
    core: *mut ffi::VSCore,
    _vs_api: *const ffi::VSAPI,
) {
    if in_map.is_null() || user_data.is_null() || core.is_null() {
        return;
    }
    let user_data = unsafe { Box::from_raw(user_data as *mut PublicFunction) };
    let in_map = unsafe { MapRef::from_ptr(in_map) };
    let out_map = unsafe { OwnedMap::from_ptr(out_map) };
    let core = unsafe { CoreRef::from_ptr(core) };
    (user_data)(&in_map, &out_map, core);
}

pub type PublicFunction = fn(in_map: &MapRef<'_, '_>, out_map: &OwnedMap<'_>, core: CoreRef);

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
pub struct PluginFunction<'a> {
    ptr: NonNull<ffi::VSPluginFunction>,
    plugin: &'a Plugin<'a>,
}

impl<'a> PluginFunction<'a> {
    pub(crate) unsafe fn from_ptr(ptr: *mut VSPluginFunction, plugin: &'a Plugin<'a>) -> Self {
        PluginFunction {
            ptr: NonNull::new_unchecked(ptr),
            plugin,
        }
    }

    fn ptr(&self) -> *mut ffi::VSPluginFunction {
        self.ptr.as_ptr()
    }

    pub fn get_name(&self) -> Option<String> {
        let ptr = unsafe { API::get_cached().get_plugin_function_name(self.ptr.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() })
        }
    }

    pub fn get_arguments(&self) -> Option<String> {
        let ptr = unsafe { API::get_cached().get_plugin_function_arguments(self.ptr.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() })
        }
    }

    pub fn get_return_type(&self) -> Option<String> {
        let ptr = unsafe { API::get_cached().get_plugin_function_return_type(self.ptr.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() })
        }
    }

    pub fn call(&self, args: &Map) -> OwnedMap<'_> {
        let name = self.get_name().expect("Function has no name");
        let name_c = CString::new(name).unwrap();
        unsafe {
            OwnedMap::from_ptr(API::get_cached().invoke(
                self.plugin.ptr(),
                name_c.as_ptr(),
                args.deref(),
            ))
        }
    }

    /// Convenience method to call the function with an empty argument map
    pub fn call_no_args(&self) -> OwnedMap<'_> {
        let empty_map = OwnedMap::new();
        self.call(&empty_map)
    }
}
