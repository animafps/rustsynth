use bitflags::bitflags;
use std::fmt::Display;

use rustsynth_sys as ffi;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum MediaType {
    Video,
    Audio,
}

bitflags! {
    /// Audio channel layout bitmask
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
    pub struct ChannelLayout: u64 {
        // Individual channel constants (1 << VSAudioChannels::*)
        const FRONT_LEFT = 1 << ffi::VSAudioChannels::acFrontLeft as u64;
        const FRONT_RIGHT = 1 << ffi::VSAudioChannels::acFrontRight as u64;
        const FRONT_CENTER = 1 << ffi::VSAudioChannels::acFrontCenter as u64;
        const LOW_FREQUENCY = 1 << ffi::VSAudioChannels::acLowFrequency as u64;
        const BACK_LEFT = 1 << ffi::VSAudioChannels::acBackLeft as u64;
        const BACK_RIGHT = 1 << ffi::VSAudioChannels::acBackRight as u64;
        const FRONT_LEFT_OF_CENTER = 1 << ffi::VSAudioChannels::acFrontLeftOFCenter as u64;
        const FRONT_RIGHT_OF_CENTER = 1 << ffi::VSAudioChannels::acFrontRightOFCenter as u64;
        const BACK_CENTER = 1 << ffi::VSAudioChannels::acBackCenter as u64;
        const SIDE_LEFT = 1 << ffi::VSAudioChannels::acSideLeft as u64;
        const SIDE_RIGHT = 1 << ffi::VSAudioChannels::acSideRight as u64;
        const TOP_CENTER = 1 << ffi::VSAudioChannels::acTopCenter as u64;
        const TOP_FRONT_LEFT = 1 << ffi::VSAudioChannels::acTopFrontLeft as u64;
        const TOP_FRONT_CENTER = 1 << ffi::VSAudioChannels::acTopFrontCenter as u64;
        const TOP_FRONT_RIGHT = 1 << ffi::VSAudioChannels::acTopFrontRight as u64;
        const TOP_BACK_LEFT = 1 << ffi::VSAudioChannels::acTopBackLeft as u64;
        const TOP_BACK_CENTER = 1 << ffi::VSAudioChannels::acTopBackCenter as u64;
        const TOP_BACK_RIGHT = 1 << ffi::VSAudioChannels::acTopBackRight as u64;
        const STEREO_LEFT = 1 << ffi::VSAudioChannels::acStereoLeft as u64;
        const STEREO_RIGHT = 1 << ffi::VSAudioChannels::acStereoRight as u64;
        const WIDE_LEFT = 1 << ffi::VSAudioChannels::acWideLeft as u64;
        const WIDE_RIGHT = 1 << ffi::VSAudioChannels::acWideRight as u64;
        const SURROUND_DIRECT_LEFT = 1 << ffi::VSAudioChannels::acSurroundDirectLeft as u64;
        const SURROUND_DIRECT_RIGHT = 1 << ffi::VSAudioChannels::acSurroundDirectRight as u64;
        const LOW_FREQUENCY_2 = 1 << ffi::VSAudioChannels::acLowFrequency2 as u64;
    }
}

impl ChannelLayout {
    // Common layout combinations
    pub const MONO: Self = Self::FRONT_LEFT;
    pub const STEREO: Self = Self::FRONT_LEFT.union(Self::FRONT_RIGHT);
    pub const SURROUND_2_1: Self = Self::FRONT_LEFT
        .union(Self::FRONT_RIGHT)
        .union(Self::LOW_FREQUENCY);
    pub const SURROUND_3_0: Self = Self::FRONT_LEFT
        .union(Self::FRONT_RIGHT)
        .union(Self::FRONT_CENTER);
    pub const SURROUND_4_0: Self = Self::FRONT_LEFT
        .union(Self::FRONT_RIGHT)
        .union(Self::BACK_LEFT)
        .union(Self::BACK_RIGHT);
    pub const SURROUND_4_1: Self = Self::FRONT_LEFT
        .union(Self::FRONT_RIGHT)
        .union(Self::BACK_LEFT)
        .union(Self::BACK_RIGHT)
        .union(Self::LOW_FREQUENCY);
    pub const SURROUND_5_0: Self = Self::FRONT_LEFT
        .union(Self::FRONT_RIGHT)
        .union(Self::FRONT_CENTER)
        .union(Self::BACK_LEFT)
        .union(Self::BACK_RIGHT);
    pub const SURROUND_5_1: Self = Self::FRONT_LEFT
        .union(Self::FRONT_RIGHT)
        .union(Self::FRONT_CENTER)
        .union(Self::LOW_FREQUENCY)
        .union(Self::BACK_LEFT)
        .union(Self::BACK_RIGHT);
    pub const SURROUND_7_1: Self = Self::FRONT_LEFT
        .union(Self::FRONT_RIGHT)
        .union(Self::FRONT_CENTER)
        .union(Self::LOW_FREQUENCY)
        .union(Self::BACK_LEFT)
        .union(Self::BACK_RIGHT)
        .union(Self::SIDE_LEFT)
        .union(Self::SIDE_RIGHT);

    /// Create a new empty channel layout
    #[must_use]
    pub const fn new() -> Self {
        Self::empty()
    }

    /// Check if a specific channel is present
    #[must_use]
    pub const fn has_channel(self, channel: Self) -> bool {
        self.contains(channel)
    }

    /// Add a channel to the layout
    #[must_use]
    pub const fn with_channel(self, channel: Self) -> Self {
        self.union(channel)
    }

    /// Remove a channel from the layout
    #[must_use]
    pub const fn without_channel(self, channel: Self) -> Self {
        self.difference(channel)
    }

    /// Count the number of channels
    #[must_use]
    pub const fn channel_count(self) -> u32 {
        self.bits().count_ones()
    }
}

impl Display for ChannelLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut channels = Vec::new();
        for i in 0..64 {
            let channel = Self::from_bits_truncate(1u64 << i);
            if self.contains(channel) {
                channels.push(format!("ch{i}"));
            }
        }
        write!(f, "ChannelLayout({})", channels.join(", "))
    }
}

impl Default for ChannelLayout {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ColorFamily {
    Undefined = 0,
    Gray = 1,
    RGB = 2,
    YUV = 3,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SampleType {
    Integer = 0,
    Float = 1,
}

impl MediaType {
    pub(crate) fn from_ffi(from: i32) -> Self {
        match from {
            x if x == ffi::VSMediaType::mtVideo as i32 => Self::Video,
            x if x == ffi::VSMediaType::mtAudio as i32 => Self::Audio,
            _ => unreachable!(),
        }
    }
}
