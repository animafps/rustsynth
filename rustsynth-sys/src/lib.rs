#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::approx_constant)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::redundant_static_lifetimes)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(deref_nullptr)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


/// Makes a VS compatible version integer 
/// 
/// # Example
/// If wanting to represent the version with major 2 and minor 1
/// ```
/// use rustsynth_sys::version;
/// let v = version!(2,1);
/// assert!(v == 0x20001);
/// ```
#[macro_export]
macro_rules! version {
    ($major:expr, $minor:expr) => {
        (($major) << 16) | ($minor)
    };
}

pub const VAPOURSYNTH_API_VERSION: i32 =
    version!(VAPOURSYNTH_API_MAJOR as i32, VAPOURSYNTH_API_MINOR as i32);
pub const VSSCRIPT_API_VERSION: i32 =
    version!(VSSCRIPT_API_MAJOR as i32, VSSCRIPT_API_MINOR as i32);
