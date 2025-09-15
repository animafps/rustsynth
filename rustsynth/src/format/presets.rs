use crate::{
    core::CoreRef,
    format::{ColorFamily, SampleType, VideoFormat},
};
/// Preset video format IDs as defined by VapourSynth.
///
/// The presets suffixed with H and S have floating point sample type. The H and S suffixes stand
/// for half precision and single precision, respectively.
///
/// The compat formats are the only packed formats in VapourSynth. Everything else is planar. They
/// exist for compatibility with Avisynth plugins. They are not to be implemented in native
/// VapourSynth plugins.
#[repr(i32)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PresetVideoFormat {
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

impl PresetVideoFormat {
    /// Consumes the PresetVideoFormatID and returns the corresponding VideoFormat from the core.
    pub fn into_format(self, core: &CoreRef) -> VideoFormat {
        core.get_video_format_by_id(self as u32).unwrap()
    }
}
