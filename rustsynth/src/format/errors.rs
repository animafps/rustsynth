use crate::format::{ColorFamily, SampleType};

/// Errors that can occur when creating formats
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    /// Invalid video format parameters
    InvalidVideoFormat {
        color_family: ColorFamily,
        sample_type: SampleType,
        bits_per_sample: i32,
        sub_sampling_w: i32,
        sub_sampling_h: i32,
    },
    /// Invalid audio format parameters
    InvalidAudioFormat {
        sample_type: SampleType,
        bits_per_sample: i32,
        channel_layout: u64,
    },
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidVideoFormat {
                color_family,
                sample_type,
                bits_per_sample,
                sub_sampling_w,
                sub_sampling_h,
            } => write!(
                f,
                "Invalid video format: color_family={color_family:?}, sample_type={sample_type:?}, bits_per_sample={bits_per_sample}, sub_sampling_w={sub_sampling_w}, sub_sampling_h={sub_sampling_h}"
            ),
            Self::InvalidAudioFormat {
                sample_type,
                bits_per_sample,
                channel_layout,
            } => write!(
                f,
                "Invalid audio format: sample_type={sample_type:?}, bits_per_sample={bits_per_sample}, channel_layout={channel_layout}"
            ),
        }
    }
}

impl std::error::Error for FormatError {}
