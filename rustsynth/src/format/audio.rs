use rustsynth_sys as ffi;

use crate::{
    api::API,
    format::{ChannelLayout, FormatError, SampleType},
};

/// Builder for creating AudioFormat with validation
#[derive(Debug, Clone)]
pub struct AudioFormatBuilder {
    sample_type: SampleType,
    bits_per_sample: i32,
    channel_layout: ChannelLayout,
}

impl AudioFormatBuilder {
    /// Create a new AudioFormat builder with the required parameters
    pub fn new(
        sample_type: SampleType,
        bits_per_sample: i32,
        channel_layout: ChannelLayout,
    ) -> Self {
        Self {
            sample_type,
            bits_per_sample,
            channel_layout,
        }
    }

    /// Set mono channel layout
    pub fn mono(mut self) -> Self {
        self.channel_layout = ChannelLayout::MONO;
        self
    }

    /// Set stereo channel layout
    pub fn stereo(mut self) -> Self {
        self.channel_layout = ChannelLayout::STEREO;
        self
    }

    /// Set custom channel layout
    pub fn channel_layout(mut self, layout: ChannelLayout) -> Self {
        self.channel_layout = layout;
        self
    }

    /// Build the AudioFormat using VapourSynth's validation
    pub fn build(self, core: &crate::core::CoreRef) -> Result<AudioFormat, FormatError> {
        AudioFormat::query(
            self.sample_type,
            self.bits_per_sample,
            self.channel_layout.bits(),
            core,
        )
    }
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
    pub channel_layout: ChannelLayout,
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
        ffi::VSAudioInfo {
            format: ffi::VSAudioFormat {
                sampleType: self.format.sample_type as i32,
                bitsPerSample: self.format.bits_per_sample,
                bytesPerSample: self.format.bytes_per_sample,
                numChannels: self.format.num_channels,
                channelLayout: self.format.channel_layout.bits(),
            },
            sampleRate: self.sample_rate,
            numSamples: self.num_samples,
            numFrames: self.num_frames,
        }
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
            channel_layout: ChannelLayout::from_bits_truncate(from.channelLayout),
        }
    }

    /// Creates an AudioFormat using VapourSynth's validation.
    /// This ensures all derived fields are correctly calculated.
    pub fn query(
        sample_type: SampleType,
        bits_per_sample: i32,
        channel_layout: u64,
        core: &crate::core::CoreRef,
    ) -> Result<Self, FormatError> {
        let mut format = ffi::VSAudioFormat {
            sampleType: 0,
            bitsPerSample: 0,
            bytesPerSample: 0,
            numChannels: 0,
            channelLayout: 0,
        };

        let success = unsafe {
            API::get_cached().query_audio_format(
                &mut format,
                sample_type as i32,
                bits_per_sample,
                channel_layout,
                core.ptr(),
            )
        };

        if success != 0 {
            Ok(unsafe { Self::from_ptr(&format) })
        } else {
            Err(FormatError::InvalidAudioFormat {
                sample_type,
                bits_per_sample,
                channel_layout,
            })
        }
    }

    pub(crate) fn as_ptr(&self) -> ffi::VSAudioFormat {
        ffi::VSAudioFormat {
            sampleType: self.sample_type as i32,
            bitsPerSample: self.bits_per_sample,
            bytesPerSample: self.bytes_per_sample,
            numChannels: self.num_channels,
            channelLayout: self.channel_layout.bits(),
        }
    }

    pub fn get_name(&self) -> Option<String> {
        unsafe { API::get_cached().get_audio_format_name(&self.as_ptr()) }
    }

    pub const STEREO16: Self = Self {
        sample_type: SampleType::Integer,
        bits_per_sample: 16,
        bytes_per_sample: 2,
        num_channels: 2,
        channel_layout: ChannelLayout::STEREO,
    };

    pub const MONO16: Self = Self {
        sample_type: SampleType::Integer,
        bits_per_sample: 16,
        bytes_per_sample: 2,
        num_channels: 1,
        channel_layout: ChannelLayout::MONO,
    };
}
