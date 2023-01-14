extern crate rustsynth_sys;

pub mod api;
pub mod core;
pub mod plugin;

pub mod prelude {
    //! The VapourSynth prelude.
    //!
    //! Contains the types you most likely want to import anyway.
    pub use super::api::API;
    pub use super::plugin::Plugin;
}
