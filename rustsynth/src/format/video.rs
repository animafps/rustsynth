use crate::{api::API, format::{ColorFamily, FormatError, SampleType}};
use rustsynth_sys as ffi;


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

    /// Creates a VideoFormat using VapourSynth's validation.
    /// This ensures all derived fields are correctly calculated.
    pub fn query(
        color_family: ColorFamily,
        sample_type: SampleType,
        bits_per_sample: i32,
        sub_sampling_w: i32,
        sub_sampling_h: i32,
        core: &crate::core::CoreRef,
    ) -> Result<Self, FormatError> {
        let mut format = ffi::VSVideoFormat {
            colorFamily: 0,
            sampleType: 0,
            bitsPerSample: 0,
            bytesPerSample: 0,
            subSamplingW: 0,
            subSamplingH: 0,
            numPlanes: 0,
        };

        let success = unsafe {
            API::get_cached().query_video_format(
                &mut format,
                color_family as i32,
                sample_type as i32,
                bits_per_sample,
                sub_sampling_w,
                sub_sampling_h,
                core.ptr(),
            )
        };

        if success != 0 {
            Ok(Self::from_ptr(&format))
        } else {
            Err(FormatError::InvalidVideoFormat {
                color_family,
                sample_type,
                bits_per_sample,
                sub_sampling_w,
                sub_sampling_h,
            })
        }
    }

    /// Get the format ID for this video format
    pub fn query_format_id(&self, core: &crate::core::CoreRef) -> u32 {
        unsafe {
            API::get_cached().query_video_format_id(
                self.color_family as i32,
                self.sample_type as i32,
                self.bits_per_sample,
                self.sub_sampling_w,
                self.sub_sampling_h,
                core.ptr(),
            )
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


/// Builder for creating VideoFormat with validation
#[derive(Debug, Clone)]
pub struct VideoFormatBuilder {
    color_family: ColorFamily,
    sample_type: SampleType,
    bits_per_sample: i32,
    sub_sampling_w: i32,
    sub_sampling_h: i32,
}

impl VideoFormatBuilder {
    /// Create a new VideoFormat builder with the minimum required parameters
    pub fn new(color_family: ColorFamily, sample_type: SampleType, bits_per_sample: i32) -> Self {
        Self {
            color_family,
            sample_type,
            bits_per_sample,
            sub_sampling_w: 0,
            sub_sampling_h: 0,
        }
    }

    /// Set horizontal subsampling (for YUV formats)
    pub fn sub_sampling_w(mut self, sub_sampling_w: i32) -> Self {
        self.sub_sampling_w = sub_sampling_w;
        self
    }

    /// Set vertical subsampling (for YUV formats)
    pub fn sub_sampling_h(mut self, sub_sampling_h: i32) -> Self {
        self.sub_sampling_h = sub_sampling_h;
        self
    }

    /// Set both horizontal and vertical subsampling (for YUV formats)
    pub fn sub_sampling(mut self, sub_sampling_w: i32, sub_sampling_h: i32) -> Self {
        self.sub_sampling_w = sub_sampling_w;
        self.sub_sampling_h = sub_sampling_h;
        self
    }

    /// Build the VideoFormat using VapourSynth's validation
    pub fn build(self, core: &crate::core::CoreRef) -> Result<VideoFormat, FormatError> {
        VideoFormat::query(
            self.color_family,
            self.sample_type,
            self.bits_per_sample,
            self.sub_sampling_w,
            self.sub_sampling_h,
            core,
        )
    }
}


impl VideoFormat {
    /// Convenience method to create common YUV420P8 format
    pub fn yuv420p8(core: &crate::core::CoreRef) -> Result<Self, FormatError> {
        Self::query(ColorFamily::YUV, SampleType::Integer, 8, 1, 1, core)
    }

    /// Convenience method to create common YUV444P8 format
    pub fn yuv444p8(core: &crate::core::CoreRef) -> Result<Self, FormatError> {
        Self::query(ColorFamily::YUV, SampleType::Integer, 8, 0, 0, core)
    }

    /// Convenience method to create common RGB24 format
    pub fn rgb24(core: &crate::core::CoreRef) -> Result<Self, FormatError> {
        Self::query(ColorFamily::RGB, SampleType::Integer, 8, 0, 0, core)
    }

    /// Convenience method to create common Gray8 format
    pub fn gray8(core: &crate::core::CoreRef) -> Result<Self, FormatError> {
        Self::query(ColorFamily::Gray, SampleType::Integer, 8, 0, 0, core)
    }
}

/// Information about a video clip
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct VideoInfo {
    /// Format of the clip. Will have color_family set to [ColorFamily::Undefined] if the format can vary.
    pub format: VideoFormat,
    /// Numerator part of the clip’s frame rate. It will be 0 if the frame rate can vary. Should always be a reduced fraction.
    pub fps_num: i64,
    /// Denominator part of the clip’s frame rate. It will be 0 if the frame rate can vary. Should always be a reduced fraction.
    pub fps_den: i64,
    /// Width of the clip. Both width and height will be 0 if the clip’s dimensions can vary.
    pub width: i32,
    /// Height of the clip. Both width and height will be 0 if the clip’s dimensions can vary.
    pub height: i32,
    /// Length of the clip.
    pub num_frames: i32,
}

/// Describes the format of a clip.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct VideoFormat {
    pub color_family: ColorFamily,
    pub sample_type: SampleType,
    /// Number of significant bits.
    pub bits_per_sample: i32,
    /// Number of bytes needed for a sample. This is always a power of 2 and the smallest possible that can fit the number of bits used per sample.
    pub bytes_per_sample: i32,
    pub sub_sampling_w: i32,
    /// log2 subsampling factor, applied to second and third plane. Convenient numbers that can be used like so:
    /// `uv_width = y_width >> subSamplingW;`
    pub sub_sampling_h: i32,
    /// Number of planes.
    pub num_planes: i32,
}
