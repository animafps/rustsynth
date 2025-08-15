use rustsynth_sys::{VSActivationReason, VSFilterMode};

use crate::ffi;
use crate::ffi::VSRequestPattern;
use crate::frame::{FrameContext, FrameRef};
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
}

pub enum RequestPattern {
    /// Anything goes. Note that filters that may be requesting beyond the end of a VSNode length in frames (repeating the last frame) should use General and not any of the other modes.
    General,
    /// Will only request an input frame at most once if all output frames are requested exactly one time. This includes filters such as Trim, Reverse, SelectEvery.
    NoFrameReuse,
    /// Only requests frame N to output frame N. The main difference to NoFrameReuse is that the requested frame is always fixed and known ahead of time. Filter examples Lut, Expr (conditionally, see General note) and similar.
    StrictSpatial,
}

impl RequestPattern {
    pub fn from_ffi(pattern: VSRequestPattern) -> Self {
        match pattern {
            VSRequestPattern::rpGeneral => Self::General,
            VSRequestPattern::rpNoFrameReuse => Self::NoFrameReuse,
            VSRequestPattern::rpStrictSpatial => Self::StrictSpatial,
        }
    }

    pub fn as_ptr(&self) -> *const VSRequestPattern {
        match self {
            Self::General => &VSRequestPattern::rpGeneral as *const VSRequestPattern,
            Self::NoFrameReuse => &VSRequestPattern::rpNoFrameReuse as *const VSRequestPattern,
            Self::StrictSpatial => &VSRequestPattern::rpStrictSpatial as *const VSRequestPattern,
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
            val if val == VSActivationReason::arInitial as i32=> Self::Initial,
            val if val == VSActivationReason::arAllFramesReady as i32 => Self::AllFramesReady,
            val if val == VSActivationReason::arError as i32 => Self::Error,
            _ => Self::Error
        }
    }
}

/// Controls how a filter will be multithreaded, if at all.
pub enum FilterMode {
    /// Completely parallel execution. Multiple threads will call a filter’s [FilterGetFrame] function, to fetch several frames in parallel.
    Paralell,
    /// For filters that are serial in nature but can request in advance one or more frames they need. A filter’s [FilterGetFrame] function will be called from multiple threads at a time with activation reason [ActivationReason::Initial], but only one thread will call it with activation reason [ActivationReason::AllFramesReady] at a time.
    ParalellRequests,
    /// Only one thread can call the filter’s [FilterGetFrame] function at a time. Useful for filters that modify or examine their internal state to determine which frames to request.
    /// While the [FilterGetFrame] function will only run in one thread at a time, the calls can happen in any order. For example, it can be called with reason [ActivationReason::Initial] for frame 0, then again with reason [ActivationReason::Initial] for frame 1, then with reason [ActivationReason::AllFramesReady] for frame 0.
    Unordered,
    /// For compatibility with other filtering architectures. DO NOT USE IN NEW FILTERS. The filter’s [FilterGetFrame] function only ever gets called from one thread at a time. Unlike fmUnordered, only one frame is processed at a time.
    FrameState,
}

impl FilterMode {
    pub fn from_ffi(mode: VSFilterMode) -> Self {
        match mode {
            VSFilterMode::fmParallel => Self::Paralell,
            VSFilterMode::fmParallelRequests => Self::ParalellRequests,
            VSFilterMode::fmUnordered => Self::Unordered,
            VSFilterMode::fmFrameState => Self::FrameState,
        }
    }

    pub fn as_ptr(&self) -> *const VSFilterMode {
        match self {
            Self::Paralell => &VSFilterMode::fmParallel as *const VSFilterMode,
            Self::ParalellRequests => &VSFilterMode::fmParallelRequests as *const VSFilterMode,
            Self::Unordered => &VSFilterMode::fmUnordered as *const VSFilterMode,
            Self::FrameState => &VSFilterMode::fmFrameState as *const VSFilterMode,
        }
    }
}

/// A filter’s “getframe” function. It is called by the core when it needs the filter to generate a frame.
/// It is possible to allocate local data, persistent during the multiple calls requesting the output frame.
/// In case of error, call [setFilterError] and return [None].
/// Depending on the [FilterMode] set for the filter, multiple output frames could be requested concurrently.
/// It is never called concurrently for the same frame number.
/// 
/// # Arguments
/// 
/// * `n`: Requested frame number.
/// * `activation_reason`: This function is first called with [ActivationReason::Initial]. At this point the function should request the input frames it needs and return [None]. When one or all of the requested frames are ready, this function is called again with [ActivationReason::AllFramesReady]. The function should only return a frame when called with [ActivationReason::AllFramesReady].
/// If a the function is called with [ActivationReason::Error] all processing has to be aborted and any.
/// * `instance_data`: The filter’s private instance data.
/// * `frame_data`:     Optional private data associated with output frame number `n``. It must be deallocated before the last call for the given frame ([ActivationReason::AllFramesReady] or error).
pub type FilterGetFrame<'a> = fn(
    n: i32,
    activation_reason: ActivationReason,
    instance_data: &mut [u8],
    frame_data: &mut Option<&mut [u8;4]>,
    frame_ctx: &FrameContext,
) -> Option<FrameRef<'a>>;

// Free callback signature
pub type FilterFree = fn(instance_data: &mut [u8]);

// TODO!
// - Filter Traits
// - Export macros