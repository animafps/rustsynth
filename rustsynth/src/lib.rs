pub extern crate rustsynth_sys as sys;
pub use sys as ffi;

pub mod api;
pub mod core;
pub mod map;
pub mod plugin;

pub mod prelude {
    //! The VapourSynth prelude.
    //!
    //! Contains the types you most likely want to import anyway.
    pub use super::api::API;
    pub use super::map::Map;
    pub use super::plugin::Plugin;
}
