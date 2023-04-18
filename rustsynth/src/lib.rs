//! A safe wraper for [VapourSynth], written in Rust
//!
//! Is a fork of [vapoursynth-rs] project for the latest VapourSynth API version
//!
//! The primary goal is safety (that is, safe Rust code should not trigger undefined behavior), and secondary goals include performance and ease of use.
//!
//! [VapourSynth]: https://github.com/vapoursynth/vapoursynth
//! [vapoursynth-rs]: https://github.com/YaLTeR/vapoursynth-rs

pub extern crate rustsynth_sys;
pub use rustsynth_sys as ffi;
pub extern crate rustsynth_derive;
pub use rustsynth_derive::OwnedMap;

mod api;
pub mod core;
pub mod format;
pub mod frame;
pub mod function;
pub mod map;
pub mod node;
pub mod plugin;
#[cfg(feature = "vsscript-functions")]
pub mod vsscript;

pub mod prelude {
    //! The VapourSynth prelude.
    //!
    //! Contains the types you most likely want to import anyway.
    pub use super::map::Map;

    #[cfg(feature = "vsscript-functions")]
    pub use super::vsscript::Environment;
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
            use rustsynth::map::OwnedMap;

            let mut temp_map = OwnedMap::new();
            $(
                temp_map.set($key, $x).unwrap();
            )*
            temp_map
        }
    };
}

/// A trait for a struct that can make a `map::OwnedMap`
pub trait OwnedMap {
    fn to_map<'elem>(self) -> map::OwnedMap<'elem>;
}

// Dev notes
//
// There is one API so if something is created or executed through the API then can get it once then use cached version everytime afterwards
// so things that are at the front: Core, Ownedmaps
//
