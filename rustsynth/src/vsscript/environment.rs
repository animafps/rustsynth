use rustsynth_sys as ffi;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::Read;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::ptr;
use std::ptr::NonNull;

use crate::core::CoreRef;
use crate::map::Map;
use crate::node::Node;
use crate::vsscript::errors::ScriptResult;
use crate::{init_api, vsscript::*};

use crate::vsscript::VSScriptError;

/// Contains two possible variants of arguments to `Environment::evaluate_script()`.
#[derive(Clone, Copy)]
enum EvaluateScriptArgs<'a> {
    /// Evaluate a script contained in the string.
    Script(&'a str),
    /// Evaluate a script contained in the file.
    File(&'a Path),
}

/// A wrapper for the VSScript environment.
#[derive(Debug)]
pub struct Environment {
    handle: NonNull<ffi::VSScript>,
}

unsafe impl Send for Environment {}
unsafe impl Sync for Environment {}

impl Drop for Environment {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            ScriptAPI::get_cached().free_script(self.handle.as_ptr());
        }
    }
}

impl Environment {
    /// Retrieves the VSScript error message.
    ///
    /// # Safety
    /// This function must only be called if an error is present.
    #[inline]
    unsafe fn error(&self) -> Option<CString> {
        let message = ScriptAPI::get_cached().get_error(self.handle.as_ptr());
        if message.is_null() {
            None
        } else {
            Some(CStr::from_ptr(message).to_owned())
        }
    }

    /// Creates an empty script environment.
    ///
    /// Useful if it is necessary to set some variable in the script environment before evaluating
    /// any scripts.
    pub fn new(core: &CoreRef) -> ScriptResult<Self> {
        let api = ScriptAPI::get().unwrap();

        let handle = unsafe { api.create_script(core.ptr()) };
        let environment = Self {
            handle: unsafe { NonNull::new_unchecked(handle) },
        };

        match unsafe { environment.error() } {
            None => Ok(environment),
            Some(error) => Err(VSScriptError::new(error).into()),
        }
    }

    /// Calls `ScriptAPI::eval_buffer()`.
    fn evaluate_script(&self, args: EvaluateScriptArgs) -> ScriptResult<()> {
        let (script, path) = match args {
            EvaluateScriptArgs::Script(script) => (script.to_owned(), None),
            EvaluateScriptArgs::File(path) => {
                let mut file = File::open(path).map_err(ScriptError::FileOpen)?;
                let mut script = String::new();
                file.read_to_string(&mut script).map_err(ScriptError::FileRead)?;

                // vsscript throws an error if it's not valid UTF-8 anyway.
                let path = path.to_str().ok_or(ScriptError::PathInvalidUnicode)?;
                let path = CString::new(path)?;

                (script, Some(path))
            }
        };

        let script = CString::new(script)?;

        let rv = unsafe {
            ScriptAPI::get_cached().eval_buffer(
                self.handle.as_ptr(),
                script.as_ptr(),
                path.as_ref().map(|p| p.as_ptr()).unwrap_or(ptr::null()),
            )
        };

        if rv != 0 {
            Err(VSScriptError::new(unsafe { self.error().unwrap() }).into())
        } else {
            Ok(())
        }
    }

    /// Creates a script environment and evaluates a script contained in a string.
    #[inline]
    pub fn from_script(core: &CoreRef, script: &str) -> ScriptResult<Self> {
        let environment = Self::new(core)?;
        environment.evaluate_script(EvaluateScriptArgs::Script(script))?;
        Ok(environment)
    }

    /// Creates a script environment and evaluates a script contained in a file.
    #[inline]
    pub fn from_file<P: AsRef<Path>>(core: &CoreRef, path: P) -> ScriptResult<Self> {
        let environment = Self::new(core)?;
        environment.evaluate_script(EvaluateScriptArgs::File(path.as_ref()))?;
        Ok(environment)
    }

    /// Evaluates a script contained in a string.
    #[inline]
    pub fn eval_script(&mut self, script: &str) -> ScriptResult<()> {
        self.evaluate_script(EvaluateScriptArgs::Script(script))
    }

    /// Evaluates a script contained in a file.
    #[inline]
    pub fn eval_file<P: AsRef<Path>>(&mut self, path: P) -> ScriptResult<()> {
        self.evaluate_script(EvaluateScriptArgs::File(path.as_ref()))
    }

    /// Retrieves a variable from the script environment.
    pub fn get_variable(&self, name: &str, map: &mut Map) -> ScriptResult<()> {
        let name = CString::new(name)?;
        let rv = unsafe {
            ScriptAPI::get_cached().get_variable(
                self.handle.as_ptr(),
                name.as_ptr(),
                map.deref_mut(),
            )
        };
        if rv != 0 {
            Err(ScriptError::NoSuchVariable)
        } else {
            Ok(())
        }
    }

    /// Sets variables in the script environment.
    pub fn set_variables(&self, variables: &Map) -> ScriptResult<()> {
        let rv = unsafe {
            ScriptAPI::get_cached().set_variables(self.handle.as_ptr(), variables.deref())
        };
        if rv != 0 {
            Err(ScriptError::NoSuchVariable)
        } else {
            Ok(())
        }
    }

    /// Retrieves a node from the script environment. A node in the script must have been marked for output with the requested index.
    ///
    /// Returns [None] if there is no node at the requested index.
    pub fn get_output(&self, index: i32) -> Option<Node> {
        let ptr = unsafe { ScriptAPI::get_cached().get_output(self.handle.as_ptr(), index as i32) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { Node::from_ptr(ptr) })
        }
    }

    /// Retrieves an alpha node from the script environment. A node with associated alpha in the script must have been marked for output with the requested index.
    ///
    /// Returns [None] if there is no node at the requested index.
    pub fn get_output_alpha(&self, index: i32) -> Option<Node> {
        let ptr =
            unsafe { ScriptAPI::get_cached().get_output_alpha(self.handle.as_ptr(), index as i32) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { Node::from_ptr(ptr) })
        }
    }

    /// Retrieves the alternative output mode settings from the script. This value has no fixed meaning but in vspipe and vsvfw it indicates that alternate output formats should be used when multiple ones are available.
    /// It is up to the client application to define the exact meaning or simply disregard it completely.
    ///
    /// Returns 0 if there is no alt output mode set.
    pub fn get_alt_output_mode(&self, index: i32) -> i32 {
        unsafe { ScriptAPI::get_cached().get_alt_output_mode(self.handle.as_ptr(), index as i32) }
    }

    /// Set whether or not the working directory is temporarily changed to the same location as the script file when evaluateFile is called. Off by default.
    pub fn eval_set_working_dir(&self, set_cwd: i32) {
        unsafe { ScriptAPI::get_cached().eval_set_working_dir(self.handle.as_ptr(), set_cwd) };
    }

    /// Retrieves the VapourSynth core that was created in the script environment. If a VapourSynth core has not been created yet, it will be created now, with the default options (see the [Python Reference](https://www.vapoursynth.com/doc/pythonreference.html)).
    pub fn get_core(&'_ self) -> CoreRef<'_> {
        let ptr = unsafe { ScriptAPI::get_cached().get_core(self.handle.as_ptr()) };
        unsafe { CoreRef::from_ptr(ptr) }
    }

    pub fn load_api(version: i32) {
        unsafe {
            init_api(ScriptAPI::get_cached().get_vsapi(version));
        }
    }

    /// Returns the exit code if the script calls sys.exit(code), or 0, if the script fails for other reasons or calls sys.exit(0)
    pub fn get_exit_code(&self) -> i32 {
        unsafe { ScriptAPI::get_cached().get_exit_code(self.handle.as_ptr()) }
    }
}
