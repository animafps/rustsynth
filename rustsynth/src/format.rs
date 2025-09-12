use rustsynth_sys as ffi;

#[cfg(feature = "f16-pixel-type")]
use half::f16;

use crate::api::API;

#[cfg(test)]
mod tests;

const fn make_video_id(
    color_family: ColorFamily,
    sample_type: SampleType,
    bits_per_sample: i32,
    sub_sampling_w: i32,
    sub_sampling_h: i32,
) -> i32 {
    ((color_family as i32) << 28)
        | ((sample_type as i32) << 24)
        | (bits_per_sample << 16)
        | (sub_sampling_w << 8)
        | sub_sampling_h
}

// Preset VapourSynth formats.
///
/// The presets suffixed with H and S have floating point sample type. The H and S suffixes stand
/// for half precision and single precision, respectively.
///
/// The compat formats are the only packed formats in VapourSynth. Everything else is planar. They
/// exist for compatibility with Avisynth plugins. They are not to be implemented in native
/// VapourSynth plugins.
#[repr(i32)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PresetFormat {
    None = 0,
    Gray8 = make_video_id(ColorFamily::Gray, SampleType::Integer, 8, 0, 0),
    Gray9 = make_video_id(ColorFamily::Gray, SampleType::Integer, 9, 0, 0),
    Gray10 = make_video_id(ColorFamily::Gray, SampleType::Integer, 10, 0, 0),
    Gray12 = make_video_id(ColorFamily::Gray, SampleType::Integer, 12, 0, 0),
    Gray14 = make_video_id(ColorFamily::Gray, SampleType::Integer, 14, 0, 0),
    Gray16 = make_video_id(ColorFamily::Gray, SampleType::Integer, 16, 0, 0),
    Gray32 = make_video_id(ColorFamily::Gray, SampleType::Integer, 32, 0, 0),

    GrayH = make_video_id(ColorFamily::Gray, SampleType::Float, 16, 0, 0),
    GrayS = make_video_id(ColorFamily::Gray, SampleType::Float, 32, 0, 0),

    YUV410P8 = make_video_id(ColorFamily::YUV, SampleType::Integer, 8, 2, 2),
    YUV411P8 = make_video_id(ColorFamily::YUV, SampleType::Integer, 8, 2, 0),
    YUV440P8 = make_video_id(ColorFamily::YUV, SampleType::Integer, 8, 0, 1),

    YUV420P8 = make_video_id(ColorFamily::YUV, SampleType::Integer, 8, 1, 1),
    YUV422P8 = make_video_id(ColorFamily::YUV, SampleType::Integer, 8, 1, 0),
    YUV444P8 = make_video_id(ColorFamily::YUV, SampleType::Integer, 8, 0, 0),

    YUV420P9 = make_video_id(ColorFamily::YUV, SampleType::Integer, 9, 1, 1),
    YUV422P9 = make_video_id(ColorFamily::YUV, SampleType::Integer, 9, 1, 0),
    YUV444P9 = make_video_id(ColorFamily::YUV, SampleType::Integer, 9, 0, 0),

    YUV420P10 = make_video_id(ColorFamily::YUV, SampleType::Integer, 10, 1, 1),
    YUV422P10 = make_video_id(ColorFamily::YUV, SampleType::Integer, 10, 1, 0),
    YUV444P10 = make_video_id(ColorFamily::YUV, SampleType::Integer, 10, 0, 0),

    YUV420P12 = make_video_id(ColorFamily::YUV, SampleType::Integer, 12, 1, 1),
    YUV422P12 = make_video_id(ColorFamily::YUV, SampleType::Integer, 12, 1, 0),
    YUV444P12 = make_video_id(ColorFamily::YUV, SampleType::Integer, 12, 0, 0),

    YUV420P14 = make_video_id(ColorFamily::YUV, SampleType::Integer, 14, 1, 1),
    YUV422P14 = make_video_id(ColorFamily::YUV, SampleType::Integer, 14, 1, 0),
    YUV444P14 = make_video_id(ColorFamily::YUV, SampleType::Integer, 14, 0, 0),

    YUV420P16 = make_video_id(ColorFamily::YUV, SampleType::Integer, 16, 1, 1),
    YUV422P16 = make_video_id(ColorFamily::YUV, SampleType::Integer, 16, 1, 0),
    YUV444P16 = make_video_id(ColorFamily::YUV, SampleType::Integer, 16, 0, 0),

    YUV444PH = make_video_id(ColorFamily::YUV, SampleType::Float, 16, 0, 0),
    YUV444PS = make_video_id(ColorFamily::YUV, SampleType::Float, 32, 0, 0),

    RGB24 = make_video_id(ColorFamily::RGB, SampleType::Integer, 8, 0, 0),
    RGB27 = make_video_id(ColorFamily::RGB, SampleType::Integer, 9, 0, 0),
    RGB30 = make_video_id(ColorFamily::RGB, SampleType::Integer, 10, 0, 0),
    RGB36 = make_video_id(ColorFamily::RGB, SampleType::Integer, 12, 0, 0),
    RGB42 = make_video_id(ColorFamily::RGB, SampleType::Integer, 14, 0, 0),
    RGB48 = make_video_id(ColorFamily::RGB, SampleType::Integer, 16, 0, 0),

    RGBH = make_video_id(ColorFamily::RGB, SampleType::Float, 16, 0, 0),
    RGBS = make_video_id(ColorFamily::RGB, SampleType::Float, 32, 0, 0),
}

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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ColorFamily {
    Undefined = 0,
    Gray = 1,
    RGB = 2,
    YUV = 3,
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
    Integer = 0,
    Float = 1,
}

impl VideoFormat {
    pub(crate) fn from_ptr(from: *const ffi::VSVideoFormat) -> Self {
        let info = unsafe { &*from };

        let sample_type = if info.sampleType == ffi::VSSampleType::stInteger as i32 {
            SampleType::Integer
        } else {
            SampleType::Float
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

    pub(crate) fn as_ptr(&self) -> ffi::VSVideoFormat {
        ffi::VSVideoFormat {
            colorFamily: self.color_family as i32,
            sampleType: self.sample_type as i32,
            bitsPerSample: self.bits_per_sample,
            bytesPerSample: self.bytes_per_sample,
            subSamplingW: self.sub_sampling_w,
            subSamplingH: self.sub_sampling_h,
            numPlanes: self.num_planes,
        }
    }

    pub fn get_name(&self) -> Option<String> {
        unsafe { API::get_cached().get_video_format_name(&self.as_ptr()) }
    }
}

impl AudioInfo {
    pub(crate) unsafe fn from_ptr(from: *const ffi::VSAudioInfo) -> Self {
        let from = &*from;
        Self {
            format: AudioFormat::from_ptr(&from.format as *const ffi::VSAudioFormat),
            sample_rate: from.sampleRate,
            num_samples: from.numSamples,
            num_frames: from.numFrames,
        }
    }

    #[allow(unused)]
    pub(crate) fn as_ptr(&self) -> ffi::VSAudioInfo {
        let info = ffi::VSAudioInfo {
            format: ffi::VSAudioFormat {
                sampleType: self.format.sample_type as i32,
                bitsPerSample: self.format.bits_per_sample,
                bytesPerSample: self.format.bytes_per_sample,
                numChannels: self.format.num_channels,
                channelLayout: self.format.channel_layout,
            },
            sampleRate: self.sample_rate,
            numSamples: self.num_samples,
            numFrames: self.num_frames,
        };
        info
    }
}

impl AudioFormat {
    pub(crate) unsafe fn from_ptr(from: *const ffi::VSAudioFormat) -> Self {
        let from = &*from;
        let sample_type = if from.sampleType == 0 {
            SampleType::Integer
        } else {
            SampleType::Float
        };

        Self {
            sample_type,
            bits_per_sample: from.bitsPerSample,
            bytes_per_sample: from.bytesPerSample,
            num_channels: from.numChannels,
            channel_layout: from.channelLayout,
        }
    }

    pub(crate) fn as_ptr(&self) -> ffi::VSAudioFormat {
        ffi::VSAudioFormat {
            sampleType: self.sample_type as i32,
            bitsPerSample: self.bits_per_sample,
            bytesPerSample: self.bytes_per_sample,
            numChannels: self.num_channels,
            channelLayout: self.channel_layout,
        }
    }

    pub fn get_name(&self) -> Option<String> {
        unsafe { API::get_cached().get_audio_format_name(&self.as_ptr()) }
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

    #[allow(unused)]
    pub fn as_ptr(&self) -> ffi::VSVideoInfo {
        ffi::VSVideoInfo {
            format: ffi::VSVideoFormat {
                colorFamily: self.format.color_family as i32,
                sampleType: self.format.sample_type as i32,
                bitsPerSample: self.format.bits_per_sample,
                bytesPerSample: self.format.bytes_per_sample,
                subSamplingW: self.format.sub_sampling_w,
                subSamplingH: self.format.sub_sampling_h,
                numPlanes: self.format.num_planes,
            },
            fpsNum: self.fps_num,
            fpsDen: self.fps_den,
            width: self.width,
            height: self.height,
            numFrames: self.num_frames,
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

impl MediaType {
    pub(crate) fn from_ffi(from: i32) -> Self {
        match from {
            x if x == ffi::VSMediaType::mtVideo as i32 => MediaType::Video,
            x if x == ffi::VSMediaType::mtAudio as i32 => MediaType::Audio,
            _ => unreachable!(),
        }
    }
}
