//! Low level VapourSynth bindings to Rust
//!
//! This crate provides raw unsafe FFI bindings to the VapourSynth API.
//! For a safe wrapper, see [rustsynth](https://crates.io/crates/rustsynth).
//!
//! ## Feature Flags
//!
//! The bindings are conditionally compiled based on feature flags:
//!
//! - **`api-41`** - Enables VapourSynth API version 4.1 headers (`VS_USE_API_41`)
//! - **`vs-graph-api`** - Enables the experimental graph API (`VS_GRAPH_API`)
//! - **`script-api-42`** - Enables VSScript API 4.2 headers (`VSSCRIPT_USE_API_42`)
//! - **`vapoursynth-functions`** - Links to the main VapourSynth functions library
//! - **`vsscript-functions`** - Links to the VSScript functions library
//!
//! Different feature combinations will expose different functions and types in the generated bindings.

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

// VSHelper4 function implementations
// These are static inline functions in VSHelper4.h that bindgen cannot generate bindings for

/// Convenience function for checking if the format never changes between frames
#[inline]
pub unsafe fn isConstantVideoFormat(vi: *const VSVideoInfo) -> i32 {
    let vi = &*vi;
    (vi.height > 0 && vi.width > 0 && vi.format.colorFamily != VSColorFamily::cfUndefined as i32) as i32
}

/// Convenience function to check if two clips have the same format
#[inline]
pub unsafe fn isSameVideoFormat(v1: *const VSVideoFormat, v2: *const VSVideoFormat) -> i32 {
    let v1 = &*v1;
    let v2 = &*v2;
    (v1.colorFamily == v2.colorFamily
        && v1.sampleType == v2.sampleType
        && v1.bitsPerSample == v2.bitsPerSample
        && v1.subSamplingW == v2.subSamplingW
        && v1.subSamplingH == v2.subSamplingH) as i32
}

/// Convenience function to check if a clip has the same format as a format id
#[inline]
pub unsafe fn isSameVideoPresetFormat(
    preset_format: u32,
    v: *const VSVideoFormat,
    core: *mut VSCore,
    vsapi: *const VSAPI,
) -> i32 {
    let v = &*v;
    let vsapi = &*vsapi;
    let query_fn = vsapi.queryVideoFormatID.unwrap();
    (query_fn(
        v.colorFamily,
        v.sampleType,
        v.bitsPerSample,
        v.subSamplingW,
        v.subSamplingH,
        core,
    ) == preset_format) as i32
}

/// Convenience function to check if two clips have the same format while also including width and height
#[inline]
pub unsafe fn isSameVideoInfo(v1: *const VSVideoInfo, v2: *const VSVideoInfo) -> i32 {
    let v1 = &*v1;
    let v2 = &*v2;
    (v1.height == v2.height
        && v1.width == v2.width
        && isSameVideoFormat(&v1.format, &v2.format) != 0) as i32
}

/// Convenience function to check if two clips have the same audio format
#[inline]
pub unsafe fn isSameAudioFormat(a1: *const VSAudioFormat, a2: *const VSAudioFormat) -> i32 {
    let a1 = &*a1;
    let a2 = &*a2;
    (a1.bitsPerSample == a2.bitsPerSample
        && a1.sampleType == a2.sampleType
        && a1.channelLayout == a2.channelLayout) as i32
}

/// Convenience function to check if two clips have the same audio info
#[inline]
pub unsafe fn isSameAudioInfo(a1: *const VSAudioInfo, a2: *const VSAudioInfo) -> i32 {
    let a1 = &*a1;
    let a2 = &*a2;
    (a1.sampleRate == a2.sampleRate && isSameAudioFormat(&a1.format, &a2.format) != 0) as i32
}

/// Multiplies and divides a rational number and reduces the result
#[inline]
pub unsafe fn muldivRational(num: *mut i64, den: *mut i64, mul: i64, div: i64) {
    if *den == 0 {
        return;
    }
    assert!(div != 0);

    *num *= mul;
    *den *= div;
    let mut a = *num;
    let mut b = *den;
    while b != 0 {
        let t = a;
        a = b;
        b = t % b;
    }
    if a < 0 {
        a = -a;
    }
    *num /= a;
    *den /= a;
}

/// Reduces a rational number
#[inline]
pub unsafe fn reduceRational(num: *mut i64, den: *mut i64) {
    muldivRational(num, den, 1, 1);
}

/// Add two rational numbers and reduces the result
#[inline]
pub unsafe fn addRational(num: *mut i64, den: *mut i64, addnum: i64, addden: i64) {
    if *den == 0 {
        return;
    }
    assert!(addden != 0);

    if *den == addden {
        *num += addnum;
    } else {
        let original_den = *den;
        let scaled_addnum = addnum * original_den;
        let scaled_num = *num * addden;

        *num = scaled_num + scaled_addnum;
        *den = original_den * addden;

        reduceRational(num, den);
    }
}

/// Converts an int64 to int with saturation
#[inline]
pub fn int64ToIntS(i: i64) -> i32 {
    if i > i32::MAX as i64 {
        i32::MAX
    } else if i < i32::MIN as i64 {
        i32::MIN
    } else {
        i as i32
    }
}

/// Converts a double to float with saturation
#[inline]
pub fn doubleToFloatS(d: f64) -> f32 {
    d as f32
}

/// Bitblt function for copying image data
#[inline]
pub unsafe fn bitblt(
    dstp: *mut std::ffi::c_void,
    dst_stride: isize,
    srcp: *const std::ffi::c_void,
    src_stride: isize,
    row_size: usize,
    height: usize,
) {
    if height == 0 {
        return;
    }

    if src_stride == dst_stride && src_stride == row_size as isize {
        std::ptr::copy_nonoverlapping(srcp as *const u8, dstp as *mut u8, row_size * height);
    } else {
        let mut srcp8 = srcp as *const u8;
        let mut dstp8 = dstp as *mut u8;
        for _ in 0..height {
            std::ptr::copy_nonoverlapping(srcp8, dstp8, row_size);
            srcp8 = srcp8.offset(src_stride);
            dstp8 = dstp8.offset(dst_stride);
        }
    }
}

/// Check if the frame dimensions are valid for a given format
#[inline]
pub unsafe fn areValidDimensions(fi: *const VSVideoFormat, width: i32, height: i32) -> i32 {
    let fi = &*fi;
    (width % (1 << fi.subSamplingW) == 0 && height % (1 << fi.subSamplingH) == 0) as i32
}
