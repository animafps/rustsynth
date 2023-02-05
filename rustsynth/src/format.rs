use rustsynth_sys as ffi;

#[cfg(feature = "f16-pixel-type")]
use half::f16;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum MediaType {
    Video,
    Audio,
}
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct VideoInfo {
    pub format: VideoFormat,
    pub fps_num: i64,
    pub fps_den: i64,
    pub width: i32,
    pub height: i32,
    pub num_frames: i32,
}
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct VideoFormat {
    pub color_family: ColorFamily,
    pub sample_type: SampleType,
    pub bits_per_sample: i32,
    pub bytes_per_sample: i32,
    pub sub_sampling_w: i32,
    pub sub_sampling_h: i32,
    pub num_planes: i32,
}
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ColorFamily {
    Undefined,
    Gray,
    RGB,
    YUV,
}
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct AudioInfo {
    pub format: AudioFormat,
    pub sample_rate: i32,
    pub num_samples: i64,
    pub num_frames: i32,
}
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct AudioFormat {
    pub sample_type: SampleType,
    pub bits_per_sample: i32,
    pub bytes_per_sample: i32,
    pub num_channels: i32,
    pub channel_layout: u64,
}
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SampleType {
    Integer,
    Float,
}

impl VideoFormat {
    pub(crate) fn from_ptr(from: *const ffi::VSVideoFormat) -> Self {
        let info = unsafe { &*from };

        let sample_type = if info.sampleType == 0 {
            SampleType::Integer
        } else if info.sampleType == 1 {
            SampleType::Float
        } else {
            panic!("Sample type not valid")
        };

        let color_family = match info.colorFamily {
            x if x == ffi::VSColorFamily::cfUndefined as i32 => ColorFamily::Undefined,
            x if x == ffi::VSColorFamily::cfGray as i32 => ColorFamily::Gray,
            x if x == ffi::VSColorFamily::cfRGB as i32 => ColorFamily::RGB,
            x if x == ffi::VSColorFamily::cfYUV as i32 => ColorFamily::YUV,
            _ => unreachable!(),
        };
        Self {
            color_family,
            sample_type,
            bits_per_sample: info.bitsPerSample,
            bytes_per_sample: info.bytesPerSample,
            sub_sampling_w: info.subSamplingW,
            sub_sampling_h: info.subSamplingH,
            num_planes: info.numPlanes,
        }
    }
}

impl<'elem> From<ffi::VSAudioInfo> for AudioInfo {
    fn from(from: ffi::VSAudioInfo) -> Self {
        Self {
            format: from.format.into(),
            sample_rate: from.sampleRate,
            num_samples: from.numSamples,
            num_frames: from.numFrames,
        }
    }
}

impl From<ffi::VSAudioFormat> for AudioFormat {
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
        }
    }
}

impl VideoInfo {
    pub(crate) unsafe fn from_ptr(from: *const ffi::VSVideoInfo) -> Self {
        let from = &*from;

        Self {
            format: VideoFormat::from_ptr(&from.format as *const ffi::VSVideoFormat),
            fps_num: from.fpsNum,
            fps_den: from.fpsDen,
            width: from.width,
            height: from.height,
            num_frames: from.numFrames,
        }
    }
}

/// A trait for possible pixel components.
///
/// # Safety
/// Implementing this trait allows retrieving slices of pixel data from the frame for the target
/// type, so the target type must be valid for the given format.
pub unsafe trait Component {
    /// Returns whether this component is valid for this format.
    fn is_valid(format: VideoFormat) -> bool;
}

unsafe impl Component for u8 {
    #[inline]
    fn is_valid(format: VideoFormat) -> bool {
        format.sample_type == SampleType::Integer && format.bytes_per_sample == 1
    }
}

unsafe impl Component for u16 {
    #[inline]
    fn is_valid(format: VideoFormat) -> bool {
        format.sample_type == SampleType::Integer && format.bytes_per_sample == 2
    }
}

unsafe impl Component for u32 {
    #[inline]
    fn is_valid(format: VideoFormat) -> bool {
        format.sample_type == SampleType::Integer && format.bytes_per_sample == 4
    }
}

#[cfg(feature = "f16-pixel-type")]
unsafe impl Component for f16 {
    #[inline]
    fn is_valid(format: Format) -> bool {
        format.sample_type == SampleType::Float && format.bytes_per_sample == 2
    }
}

unsafe impl Component for f32 {
    #[inline]
    fn is_valid(format: VideoFormat) -> bool {
        format.sample_type == SampleType::Float && format.bytes_per_sample == 4
    }
}
