//! A safe wraper for [VapourSynth], written in Rust
//!
//! The primary goal is safety (that is, safe Rust code should not trigger undefined behavior), and secondary goals include performance and ease of use.
//!
//! [VapourSynth]: https://github.com/vapoursynth/vapoursynth
#![feature(doc_cfg)]
pub extern crate rustsynth_sys;
pub use rustsynth_sys as ffi;

#[cfg(feature = "proc-macro")]
extern crate rustsynth_derive;
#[doc(cfg(feature = "proc-macro"))]
#[cfg(feature = "proc-macro")]
pub use rustsynth_derive::*;

mod api;
pub mod core;
pub mod filter;
pub mod format;
pub mod frame;
pub mod function;
pub mod log;
pub mod map;
pub mod node;
pub mod plugin;
#[cfg(feature = "vsscript-functions")]
#[doc(cfg(feature = "vsscript-functions"))]
pub mod vsscript;
pub use api::init_api;

pub mod prelude {
    //! The VapourSynth prelude.
    //!
    //! Contains the types you most likely want to import anyway.
    pub use super::{
        core::{CoreCreationFlags, CoreRef},
        filter::{Filter, FilterMode, RequestPattern},
        format::{VideoFormat, VideoInfo},
        frame::Frame,
        node::Node,
    };
}

pub fn api_version() -> i32 {
    api::API::get().unwrap().version()
}

/// A simple macro to create an owned map
///
/// its syntax is `owned_map!({"key": value}, ... , {"key": value})`
///
/// # Example
///
/// ```
/// use rustsynth::owned_map;
/// let map = owned_map!({"int": &0});
/// ```
#[macro_export(local_inner_macros)]
macro_rules! owned_map {
    ($({$key:literal: $x:expr }),*) => {
        {
            let mut temp_map = $crate::map::OwnedMap::new();
            $(
                temp_map.set($key, $x).unwrap();
            )*
            temp_map
        }
    };
}

// Dev notes
//
// There is one API so if something is created or executed through the API then can get it once then use cached version everytime afterwards
// so things that are at the front: Core, Ownedmaps
//

pub use ffi::version as MakeVersion;
