//! A safe wraper for [VapourSynth], written in Rust
//!
//! Is a fork of [vapoursynth-rs] project for the latest VapourSynth API version
//!
//! The primary goal is safety (that is, safe Rust code should not trigger undefined behavior), and secondary goals include performance and ease of use.
//!
//! [VapourSynth]: https://github.com/vapoursynth/vapoursynth
//! [vapoursynth-rs]: https://github.com/YaLTeR/vapoursynth-rs

// Preventing all those warnings with #[cfg] directives would be really diffucult.
#![allow(unused, dead_code)]
#![allow(clippy::trivially_copy_pass_by_ref)]

pub extern crate rustsynth_sys;

pub use rustsynth_sys as ffi;

pub mod api;
pub mod core;
pub mod filter;
pub mod format;
pub mod frame;
pub mod function;
pub mod map;
pub mod node;
pub mod plugin;
pub mod vsscript;

pub mod prelude {
    //! The VapourSynth prelude.
    //!
    //! Contains the types you most likely want to import anyway.
    pub use super::api::API;
    pub use super::map::Map;
    pub use super::vsscript::Environment;
}
