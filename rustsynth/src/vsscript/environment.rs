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
use crate::vsscript::errors::Result;
use crate::vsscript::*;

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
    pub fn new(core: &CoreRef) -> Result<Self> {
        ScriptAPI::get();

        let handle = unsafe { ScriptAPI::get_cached().create_script(core.ptr()) };
        let environment = Self {
            handle: unsafe { NonNull::new_unchecked(handle) },
        };

        match unsafe { environment.error() } {
            None => Ok(environment),
            Some(error) => Err(VSScriptError::new(error).into()),
        }
    }

    /// Calls `ScriptAPI::eval_buffer()`.
    fn evaluate_script(&self, args: EvaluateScriptArgs) -> Result<()> {
        let (script, path) = match args {
            EvaluateScriptArgs::Script(script) => (script.to_owned(), None),
            EvaluateScriptArgs::File(path) => {
                let mut file = File::open(path).map_err(Error::FileOpen)?;
                let mut script = String::new();
                file.read_to_string(&mut script).map_err(Error::FileRead)?;

                // vsscript throws an error if it's not valid UTF-8 anyway.
                let path = path.to_str().ok_or(Error::PathInvalidUnicode)?;
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
    pub fn from_script(core: &CoreRef, script: &str) -> Result<Self> {
        let environment = Self::new(core)?;
        environment.evaluate_script(EvaluateScriptArgs::Script(script))?;
        Ok(environment)
    }

    /// Creates a script environment and evaluates a script contained in a file.
    #[inline]
    pub fn from_file<P: AsRef<Path>>(core: &CoreRef, path: P) -> Result<Self> {
        let environment = Self::new(core)?;
        environment.evaluate_script(EvaluateScriptArgs::File(path.as_ref()))?;
        Ok(environment)
    }

    /// Evaluates a script contained in a string.
    #[inline]
    pub fn eval_script(&mut self, script: &str) -> Result<()> {
        self.evaluate_script(EvaluateScriptArgs::Script(script))
    }

    /// Evaluates a script contained in a file.
    #[inline]
    pub fn eval_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.evaluate_script(EvaluateScriptArgs::File(path.as_ref()))
    }

    /// Retrieves a variable from the script environment.
    pub fn get_variable(&self, name: &str, map: &mut Map) -> Result<()> {
        let name = CString::new(name)?;
        let rv = unsafe {
            ScriptAPI::get_cached().get_variable(
                self.handle.as_ptr(),
                name.as_ptr(),
                map.deref_mut(),
            )
        };
        if rv != 0 {
            Err(Error::NoSuchVariable)
        } else {
            Ok(())
        }
    }

    /// Sets variables in the script environment.
    pub fn set_variables(&self, variables: &Map) -> Result<()> {
        let rv = unsafe {
            ScriptAPI::get_cached().set_variables(self.handle.as_ptr(), variables.deref())
        };
        if rv != 0 {
            Err(Error::NoSuchVariable)
        } else {
            Ok(())
        }
    }

    pub fn get_output(&self, index: usize) -> Option<Node<'_>> {
        let ptr = unsafe { ScriptAPI::get_cached().get_output(self.handle.as_ptr(), index as i32) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { Node::from_ptr(ptr) })
        }
    }

    pub fn get_output_alpha(&self, index: usize) -> Option<Node<'_>> {
        let ptr =
            unsafe { ScriptAPI::get_cached().get_output_alpha(self.handle.as_ptr(), index as i32) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { Node::from_ptr(ptr) })
        }
    }

    pub fn clear_output(&self, index: usize) -> Result<()> {
        todo!()
    }
}
