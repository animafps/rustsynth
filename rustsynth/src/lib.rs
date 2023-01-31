pub extern crate rustsynth_sys;
use std::marker::PhantomData;

pub use rustsynth_sys as ffi;

pub mod api;
pub mod core;
pub mod filter;
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
    pub use super::plugin::Plugin;
    pub use super::vsscript::Environment;
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum MediaType {
    Video,
    Audio,
}
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct VideoInfo<'elem> {
    pub format: VideoFormat<'elem>,
    pub fps_num: i64,
    pub fps_den: i64,
    pub width: i32,
    pub height: i32,
    pub num_frames: i32,
}
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct VideoFormat<'elem> {
    pub color_family: ColorFamily,
    pub sample_type: SampleType,
    pub bits_per_sample: i32,
    pub bytes_per_sample: i32,
    pub sub_sampling_w: i32,
    pub sub_sampling_h: i32,
    pub num_planes: i32,
    _elem: PhantomData<&'elem ()>,
}
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ColorFamily {
    Undefined,
    Gray,
    RGB,
    YUV,
}
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct AudioInfo<'elem> {
    pub format: AudioFormat<'elem>,
    pub sample_rate: i32,
    pub num_samples: i64,
    pub num_frames: i32,
}
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct AudioFormat<'elem> {
    pub sample_type: SampleType,
    pub bits_per_sample: i32,
    pub bytes_per_sample: i32,
    pub num_channels: i32,
    pub channel_layout: u64,
    _elem: PhantomData<&'elem ()>,
}
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SampleType {
    Integer,
    Float,
}

impl<'elem> From<ffi::VSVideoFormat> for VideoFormat<'elem> {
    fn from(from: ffi::VSVideoFormat) -> Self {
        let sample_type = if from.sampleType == 0 {
            SampleType::Integer
        } else if from.sampleType == 1 {
            SampleType::Float
        } else {
            panic!("Sample type not valid")
        };

        let color_family = if from.colorFamily == 0 {
            ColorFamily::Undefined
        } else if from.colorFamily == 1 {
            ColorFamily::Gray
        } else {
            panic!("Color family not valid")
        };
        Self {
            color_family,
            sample_type,
            bits_per_sample: from.bitsPerSample,
            bytes_per_sample: from.bytesPerSample,
            sub_sampling_w: from.subSamplingW,
            sub_sampling_h: from.subSamplingH,
            num_planes: from.numPlanes,
            _elem: PhantomData,
        }
    }
}

impl<'elem> From<ffi::VSAudioInfo> for AudioInfo<'elem> {
    fn from(from: ffi::VSAudioInfo) -> Self {
        Self {
            format: from.format.into(),
            sample_rate: from.sampleRate,
            num_samples: from.numSamples,
            num_frames: from.numFrames,
        }
    }
}

impl<'elem> From<ffi::VSAudioFormat> for AudioFormat<'elem> {
    fn from(from: ffi::VSAudioFormat) -> Self {
        let sample_type = if from.sampleType == 0 {
            SampleType::Integer
        } else if from.sampleType == 1 {
            SampleType::Float
        } else {
            panic!("Sample type not valid")
        };
        Self {
            sample_type,
            bits_per_sample: from.bitsPerSample,
            bytes_per_sample: from.bytesPerSample,
            num_channels: from.numChannels,
            channel_layout: from.channelLayout,
            _elem: PhantomData,
        }
    }
}

impl<'elem> From<ffi::VSVideoInfo> for VideoInfo<'elem> {
    fn from(from: ffi::VSVideoInfo) -> Self {
        Self {
            format: from.format.into(),
            fps_num: from.fpsNum,
            fps_den: from.fpsDen,
            width: from.width,
            height: from.height,
            num_frames: from.numFrames,
        }
    }
}
