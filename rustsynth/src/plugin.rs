use ffi::VSPluginFunction;
use rustsynth_sys as ffi;
use std::{
    ffi::{c_void, CStr, CString},
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



/// A plugin's entry point. It must be called `VapourSynthPluginInit2`. 
/// This function is called after the core loads the shared library. 
/// Its purpose is to configure the plugin and to register the filters the plugin wants to export.
/// 
/// # Arguments
/// * `plugin`: The plugin object to be initialized.
/// * `vspapi`: The VapourSynth Plugin API for configuration and registration.
pub type InitPlugin = fn(plugin: &Plugin, vspapi: &PluginAPI);

/// A wrapper for the VapourSynth Plugin API used during plugin initialization.
#[derive(Debug, Clone, Copy)]
pub struct PluginAPI {
    handle: NonNull<ffi::VSPLUGINAPI>,
}

unsafe impl Send for PluginAPI {}
unsafe impl Sync for PluginAPI {}

impl PluginAPI {
    /// Creates a PluginAPI wrapper from a raw pointer
    #[inline]
    pub unsafe fn from_ptr(ptr: *const ffi::VSPLUGINAPI) -> Self {
        PluginAPI {
            handle: NonNull::new_unchecked(ptr as *mut ffi::VSPLUGINAPI),
        }
    }

    /// Returns the API version
    pub fn version(&self) -> i32 {
        unsafe { self.handle.as_ref().getAPIVersion.unwrap()() }
    }

    /// Configure the plugin with basic information
    pub fn config_plugin(
        &self,
        identifier: &str,
        plugin_namespace: &str,
        name: &str,
        plugin_version: i32,
        api_version: i32,
        flags: i32,
        plugin: &Plugin,
    ) -> Result<(), &'static str> {
        let identifier = CString::new(identifier).map_err(|_| "Invalid identifier")?;
        let namespace = CString::new(plugin_namespace).map_err(|_| "Invalid namespace")?;
        let name = CString::new(name).map_err(|_| "Invalid name")?;

        unsafe {
            eprintln!("Calling configPlugin with params:");
            eprintln!("  identifier: {:?}", CStr::from_ptr(identifier.as_ptr()));
            eprintln!("  namespace: {:?}", CStr::from_ptr(namespace.as_ptr()));
            eprintln!("  name: {:?}", CStr::from_ptr(name.as_ptr()));
            eprintln!("  plugin_version: {}", plugin_version);
            eprintln!("  api_version: {}", api_version);
            eprintln!("  flags: {}", flags);
            
            let result = self.handle.as_ref().configPlugin.unwrap()(
                identifier.as_ptr(),
                namespace.as_ptr(),
                name.as_ptr(),
                plugin_version,
                api_version,
                flags,
                plugin.ptr(),
            );

            eprintln!("configPlugin returned: {}", result);

            if result == 0 {
                Ok(())
            } else {
                Err("Failed to configure plugin")
            }
        }
    }

    /// Register a function with the plugin
    pub fn register_function(
        &self,
        name: &str,
        args: &str,
        return_type: &str,
        create_func: ffi::VSPublicFunction,
        user_data: *mut c_void,
        plugin: &Plugin,
    ) -> Result<(), &'static str> {
        let name_cstr = CString::new(name).map_err(|_| "Invalid function name")?;
        let args_cstr = CString::new(args).map_err(|_| "Invalid arguments string")?;
        let return_type_cstr = CString::new(return_type).map_err(|_| "Invalid return type string")?;

        unsafe {
            let result = self.handle.as_ref().registerFunction.unwrap()(
                name_cstr.as_ptr(),
                args_cstr.as_ptr(),
                return_type_cstr.as_ptr(),
                create_func,
                user_data,
                plugin.ptr(),
            );

            if result == 0 {
                Ok(())
            } else {
                Err("Failed to register function")
            }
        }
    }
}

/// Macro to generate the VapourSynthPluginInit2 function
#[macro_export]
macro_rules! vapoursynth_plugin_init {
    ($init_func:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn VapourSynthPluginInit2(
            plugin: *mut $crate::ffi::VSPlugin,
            vspapi: *const $crate::ffi::VSPLUGINAPI,
        ) {
            let plugin_wrapper = $crate::plugin::Plugin::from_ptr(plugin);
            let vspapi_wrapper = $crate::plugin::PluginAPI::from_ptr(vspapi);
            
            let init_fn: $crate::plugin::InitPlugin = $init_func;
            init_fn(&plugin_wrapper, &vspapi_wrapper);
        }
    };
}