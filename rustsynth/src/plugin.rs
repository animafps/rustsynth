//! Interface for `VapourSynth` plugins and their functions.
use bitflags::bitflags;
use ffi::VSPluginFunction;
use rustsynth_sys::{self as ffi, VSPluginConfigFlags};
use std::{
    ffi::{c_void, CStr, CString, NulError},
    marker::PhantomData,
    ptr::{self, NonNull},
};
use thiserror::Error;

use crate::{
    api::API,
    core::CoreRef,
    map::{Map, MapError, MapRef},
};

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Function '{0}' not found in plugin")]
    FunctionNotFound(String),
    #[error("Plugin function call failed: {0}")]
    FunctionCallFailed(#[from] PluginFunctionError),
    #[error("Error: {0}")]
    InvokeError(String),
    #[error("Output map error: {0}")]
    OutputMapError(MapError),
    #[error("CString conversion error: {0}")]
    CStringConversion(#[from] NulError),
    #[error("Failed to register function")]
    RegistrationFailed,
}

/// A `VapourSynth` plugin.
///
/// There are a few of these built into the core, and therefore available at all times: the basic filters (identifier `com.vapoursynth.std`, namespace `std`), the resizers (identifier `com.vapoursynth.resize`, namespace `resize`), and the Avisynth compatibility module, if running in Windows (identifier `com.vapoursynth.avisynth`, namespace `avs`).
#[derive(Debug, Clone, Copy)]
pub struct Plugin<'core> {
    handle: NonNull<ffi::VSPlugin>,
    _owner: PhantomData<&'core ()>,
}

unsafe impl Send for Plugin<'_> {}
unsafe impl Sync for Plugin<'_> {}

impl<'core> Plugin<'core> {
    /// Creates a plugin from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be valid and point to a `VSPlugin`.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut ffi::VSPlugin) -> Self {
        Plugin {
            handle: NonNull::new_unchecked(ptr),
            _owner: PhantomData,
        }
    }

    /// Returns the underlying pointer.
    #[inline]
    #[must_use]
    pub const fn as_ptr(&self) -> *mut ffi::VSPlugin {
        self.handle.as_ptr()
    }

    /// The path to the shared object of the plugin or `None` if is a internal `VapourSynth` plugin
    #[must_use]
    pub fn path(&self) -> Option<String> {
        let ptr = unsafe { API::get_cached().get_plugin_path(self.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() })
        }
    }

    /// The id associated with the plugin or `None` if it has no id set
    #[must_use]
    pub fn id(&self) -> Option<String> {
        let ptr = unsafe { API::get_cached().get_plugin_id(self.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() })
        }
    }

    /// The namespace associated with the plugin or `None` if it has no namespace set
    #[must_use]
    pub fn namespace(&self) -> Option<String> {
        let ptr = unsafe { API::get_cached().get_plugin_ns(self.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() })
        }
    }

    /// The name associated with the plugin or `None` if it has no name set
    #[must_use]
    pub fn name(&self) -> Option<String> {
        let ptr = unsafe { API::get_cached().get_plugin_name(self.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() })
        }
    }

    #[inline]
    #[must_use]
    pub fn version(&self) -> i32 {
        unsafe { API::get_cached().get_plugin_version(self.as_ptr()) }
    }

    /// Get function struct associated with the name
    ///
    /// returns `None` if no function is found
    #[must_use]
    pub fn function(&self, name: &str) -> Option<PluginFunction<'_>> {
        let name_ptr = CString::new(name).ok()?;
        unsafe {
            let ptr =
                API::get_cached().get_plugin_function_by_name(name_ptr.as_ptr(), self.as_ptr());
            if ptr.is_null() {
                None
            } else {
                Some(PluginFunction::from_ptr(ptr, self))
            }
        }
    }

    /// Creates an iterator over all the functions of the plugin in an arbitrary order
    #[must_use]
    pub const fn functions(&'_ self) -> PluginFunctions<'_> {
        PluginFunctions {
            function: None,
            plugin: self,
        }
    }

    fn next_function(&self, function: Option<PluginFunction<'_>>) -> Option<PluginFunction<'_>> {
        unsafe {
            let function = if let Some(value) = function {
                value.as_ptr()
            } else {
                ptr::null_mut()
            };
            let ptr = API::get_cached().get_next_plugin_function(function, self.as_ptr());
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
    pub fn invoke(&self, name: &str, args: &Map<'core>) -> PluginResult<Map<'core>> {
        let func = self
            .function(name)
            .ok_or(PluginError::FunctionNotFound(name.to_string()))?;
        Ok(func.call(args)?)
    }

    /// Tries to invoke a plugin function, returning a Result instead of panicking
    pub fn try_invoke(&self, name: &str, args: &Map<'core>) -> PluginResult<Map<'core>> {
        let func = self
            .function(name)
            .ok_or_else(|| PluginError::FunctionNotFound(name.to_string()))?;
        let ret = func.call(args)?;
        if let Ok(err) = ret.error() {
            return Err(PluginError::InvokeError(err.to_string()));
        }
        Ok(ret)
    }

    /// Convenience method to invoke a function with no arguments
    pub fn invoke_no_args(&self, name: &str) -> PluginResult<Map<'core>> {
        let empty_map = Map::new().map_err(PluginError::OutputMapError)?;
        self.try_invoke(name, &empty_map)
    }

    /// Convenience method to try invoke a function with no arguments
    pub fn try_invoke_no_args(&self, name: &str) -> PluginResult<Map<'core>> {
        let empty_map = Map::new().map_err(PluginError::OutputMapError)?;
        self.try_invoke(name, &empty_map)
    }

    /// Function that registers a filter exported by the plugin. A plugin can export any number of filters. This function may only be called during the plugin loading phase unless the [`PluginConfigFlags::MODIFIABLE`] flag was set.
    pub fn register_function(
        &self,
        name: &str,
        args: &str,
        ret_type: &str,
        func: PublicFunction,
    ) -> PluginResult<()> {
        let name_c = CString::new(name)?;
        let args_c = CString::new(args)?;
        let ret_type_c = CString::new(ret_type)?;
        let user_data: Box<PublicFunction> = Box::new(func);
        let user_data_ptr = Box::into_raw(user_data).cast::<c_void>();
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
            Err(PluginError::RegistrationFailed)
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
    let user_data = unsafe { Box::from_raw(user_data.cast::<PublicFunction>()) };
    let in_map = unsafe { MapRef::from_ptr(in_map) };
    let out_map = unsafe { MapRef::from_ptr_mut(out_map) };
    let core = unsafe { CoreRef::from_ptr(core) };
    (user_data)(in_map, out_map, core);
}

pub type PublicFunction = fn(in_map: &MapRef<'_>, out_map: &mut MapRef<'_>, core: CoreRef);

bitflags! {
    pub struct PluginConfigFlags: i32 {
        /// Allow functions to be added to the plugin object after the plugin loading phase. Mostly useful for Avisynth compatibility and other foreign plugin loaders.
        const MODIFIABLE = 1;
        const NONE  = 0;
    }
}

impl PluginConfigFlags {
    #[must_use]
    pub const fn as_ffi(&self) -> ffi::VSPluginConfigFlags {
        VSPluginConfigFlags(self.bits() as u32)
    }
}

/// The iterator over the functions found in a plugin
///
/// created by [`Plugin::functions()`]
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
    /// # Safety
    /// The pointer must be valid and point to a `VSPluginFunction`.
    pub const unsafe fn from_ptr(ptr: *mut VSPluginFunction, plugin: &'a Plugin<'a>) -> Self {
        PluginFunction {
            ptr: NonNull::new_unchecked(ptr),
            plugin,
        }
    }

    #[must_use]
    pub const fn as_ptr(&self) -> *mut ffi::VSPluginFunction {
        self.ptr.as_ptr()
    }

    #[must_use]
    pub fn get_name(&self) -> Option<String> {
        let ptr = unsafe { API::get_cached().get_plugin_function_name(self.ptr.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() })
        }
    }

    #[must_use]
    pub fn get_arguments(&self) -> Option<String> {
        let ptr = unsafe { API::get_cached().get_plugin_function_arguments(self.ptr.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() })
        }
    }

    #[must_use]
    pub fn get_return_type(&self) -> Option<String> {
        let ptr = unsafe { API::get_cached().get_plugin_function_return_type(self.ptr.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() })
        }
    }

    pub fn call<'map>(&self, args: &Map<'map>) -> Result<Map<'map>, PluginFunctionError> {
        let name = self.get_name().ok_or(PluginFunctionError::NoName)?;
        let name_c = CString::new(name).map_err(PluginFunctionError::NulError)?;
        unsafe {
            Ok(Map::from_ptr(API::get_cached().invoke(
                self.plugin.as_ptr(),
                name_c.as_ptr(),
                &*args.as_ptr(),
            )))
        }
    }

    /// Convenience method to call the function with an empty argument map
    pub fn call_no_args(&self) -> Result<Map<'_>, PluginFunctionError> {
        let empty_map = Map::new().map_err(|_| PluginFunctionError::CreateMapFailed)?;
        self.call(&empty_map)
    }
}

pub type PluginResult<T> = Result<T, PluginError>;

#[derive(thiserror::Error, Debug)]
pub enum PluginFunctionError {
    #[error("Fuction has no name")]
    NoName,
    #[error("Nul error in string: {0}")]
    NulError(#[from] NulError),
    #[error("Function call failed")]
    CallFailed,
    #[error("Failed to create output map")]
    CreateMapFailed,
}
