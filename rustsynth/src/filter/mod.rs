use rustsynth_sys::{VSActivationReason, VSFilterMode};

use crate::ffi;
use crate::ffi::VSRequestPattern;
use crate::node::Node;

pub struct FilterDependency {
    pub source: Node,
    pub request_pattern: RequestPattern,
}

impl FilterDependency {
    pub fn as_ffi(&self) -> ffi::VSFilterDependency {
        ffi::VSFilterDependency {
            source: self.source.ptr(),
            requestPattern: self.request_pattern.as_ptr() as i32,
        }
    }

    pub fn from_ptr(ptr: *const ffi::VSFilterDependency) -> Self {
        unsafe {
            Self {
                source: Node::from_ptr((*ptr).source),
                request_pattern: (*ptr).requestPattern.into(),
            }
        }
    }
}

pub enum RequestPattern {
    /// Anything goes. Note that filters that may be requesting beyond the end of a VSNode length in frames (repeating the last frame) should use General and not any of the other modes.
    General,
    /// Will only request an input frame at most once if all output frames are requested exactly one time. This includes filters such as Trim, Reverse, SelectEvery.
    NoFrameReuse,
    /// Only requests frame N to output frame N. The main difference to NoFrameReuse is that the requested frame is always fixed and known ahead of time. Filter examples Lut, Expr (conditionally, see General note) and similar.
    StrictSpatial,
    /// This modes is basically identical NoFrameReuse except that it hints the last frame may be requested multiple times
    #[cfg(feature = "api-41")]
    FrameReuseLastOnly,
}

impl RequestPattern {
    pub fn from_ffi(pattern: VSRequestPattern) -> Self {
        match pattern {
            VSRequestPattern::rpGeneral => Self::General,
            VSRequestPattern::rpNoFrameReuse => Self::NoFrameReuse,
            VSRequestPattern::rpStrictSpatial => Self::StrictSpatial,
            #[cfg(feature = "api-41")]
            VSRequestPattern::rpFrameReuseLastOnly => Self::FrameReuseLastOnly,
        }
    }

    pub fn as_ptr(&self) -> *const VSRequestPattern {
        match self {
            Self::General => &VSRequestPattern::rpGeneral as *const VSRequestPattern,
            Self::NoFrameReuse => &VSRequestPattern::rpNoFrameReuse as *const VSRequestPattern,
            Self::StrictSpatial => &VSRequestPattern::rpStrictSpatial as *const VSRequestPattern,
            #[cfg(feature = "api-41")]
            Self::FrameReuseLastOnly => {
                &VSRequestPattern::rpFrameReuseLastOnly as *const VSRequestPattern
            }
        }
    }
}

impl From<i32> for RequestPattern {
    fn from(value: i32) -> Self {
        match value {
            val if val == VSRequestPattern::rpGeneral as i32 => Self::General,
            val if val == VSRequestPattern::rpNoFrameReuse as i32 => Self::NoFrameReuse,
            val if val == VSRequestPattern::rpStrictSpatial as i32 => Self::StrictSpatial,
            #[cfg(feature = "api-41")]
            val if val == VSRequestPattern::rpFrameReuseLastOnly as i32 => Self::FrameReuseLastOnly,
            _ => Self::General,
        }
    }
}

pub enum ActivationReason {
    Initial,
    AllFramesReady,
    Error,
}

impl ActivationReason {
    pub fn from_ffi(reason: i32) -> Self {
        match reason {
            val if val == VSActivationReason::arInitial as i32 => Self::Initial,
            val if val == VSActivationReason::arAllFramesReady as i32 => Self::AllFramesReady,
            val if val == VSActivationReason::arError as i32 => Self::Error,
            _ => Self::Error,
        }
    }
}

/// Controls how a filter will be multithreaded, if at all.
pub enum FilterMode {
    /// Completely parallel execution. Multiple threads will call a filter’s [FilterGetFrame] function, to fetch several frames in parallel.
    Parallel,
    /// For filters that are serial in nature but can request in advance one or more frames they need. A filter’s [FilterGetFrame] function will be called from multiple threads at a time with activation reason [ActivationReason::Initial], but only one thread will call it with activation reason [ActivationReason::AllFramesReady] at a time.
    ParallelRequests,
    /// Only one thread can call the filter’s [FilterGetFrame] function at a time. Useful for filters that modify or examine their internal state to determine which frames to request.
    /// While the [FilterGetFrame] function will only run in one thread at a time, the calls can happen in any order. For example, it can be called with reason [ActivationReason::Initial] for frame 0, then again with reason [ActivationReason::Initial] for frame 1, then with reason [ActivationReason::AllFramesReady] for frame 0.
    Unordered,
    /// For compatibility with other filtering architectures. DO NOT USE IN NEW FILTERS. The filter’s [FilterGetFrame] function only ever gets called from one thread at a time. Unlike fmUnordered, only one frame is processed at a time.
    FrameState,
}

impl FilterMode {
    pub fn from_ffi(mode: VSFilterMode) -> Self {
        match mode {
            VSFilterMode::fmParallel => Self::Parallel,
            VSFilterMode::fmParallelRequests => Self::ParallelRequests,
            VSFilterMode::fmUnordered => Self::Unordered,
            VSFilterMode::fmFrameState => Self::FrameState,
        }
    }

    pub fn as_ptr(&self) -> *const VSFilterMode {
        match self {
            Self::Parallel => &VSFilterMode::fmParallel as *const VSFilterMode,
            Self::ParallelRequests => &VSFilterMode::fmParallelRequests as *const VSFilterMode,
            Self::Unordered => &VSFilterMode::fmUnordered as *const VSFilterMode,
            Self::FrameState => &VSFilterMode::fmFrameState as *const VSFilterMode,
        }
    }
}

impl From<i32> for FilterMode {
    fn from(value: i32) -> Self {
        match value {
            val if val == VSFilterMode::fmParallel as i32 => Self::Parallel,
            val if val == VSFilterMode::fmParallelRequests as i32 => Self::ParallelRequests,
            val if val == VSFilterMode::fmUnordered as i32 => Self::Unordered,
            val if val == VSFilterMode::fmFrameState as i32 => Self::FrameState,
            _ => Self::Parallel,
        }
    }
}

pub mod traits;

// Macro to automatically register filters
#[macro_export]
macro_rules! register_filters {
    ($($filter:ty),* $(,)?) => {
        fn __register_filters(
            plugin: *mut rustsynth::ffi::VSPlugin,
            vspapi: *const rustsynth::ffi::VSPLUGINAPI
        ) {
            $(
                <$filter>::register_filter(plugin,vspapi);
            )*
        }
    };
}
