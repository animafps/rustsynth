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
    pub const MONO: ChannelLayout = ChannelLayout::FRONT_LEFT;
    pub const STEREO: ChannelLayout = ChannelLayout::FRONT_LEFT.union(ChannelLayout::FRONT_RIGHT);
    pub const SURROUND_2_1: ChannelLayout = ChannelLayout::FRONT_LEFT
        .union(ChannelLayout::FRONT_RIGHT)
        .union(ChannelLayout::LOW_FREQUENCY);
    pub const SURROUND_3_0: ChannelLayout = ChannelLayout::FRONT_LEFT
        .union(ChannelLayout::FRONT_RIGHT)
        .union(ChannelLayout::FRONT_CENTER);
    pub const SURROUND_4_0: ChannelLayout = ChannelLayout::FRONT_LEFT
        .union(ChannelLayout::FRONT_RIGHT)
        .union(ChannelLayout::BACK_LEFT)
        .union(ChannelLayout::BACK_RIGHT);
    pub const SURROUND_4_1: ChannelLayout = ChannelLayout::FRONT_LEFT
        .union(ChannelLayout::FRONT_RIGHT)
        .union(ChannelLayout::BACK_LEFT)
        .union(ChannelLayout::BACK_RIGHT)
        .union(ChannelLayout::LOW_FREQUENCY);
    pub const SURROUND_5_0: ChannelLayout = ChannelLayout::FRONT_LEFT
        .union(ChannelLayout::FRONT_RIGHT)
        .union(ChannelLayout::FRONT_CENTER)
        .union(ChannelLayout::BACK_LEFT)
        .union(ChannelLayout::BACK_RIGHT);
    pub const SURROUND_5_1: ChannelLayout = ChannelLayout::FRONT_LEFT
        .union(ChannelLayout::FRONT_RIGHT)
        .union(ChannelLayout::FRONT_CENTER)
        .union(ChannelLayout::LOW_FREQUENCY)
        .union(ChannelLayout::BACK_LEFT)
        .union(ChannelLayout::BACK_RIGHT);
    pub const SURROUND_7_1: ChannelLayout = ChannelLayout::FRONT_LEFT
        .union(ChannelLayout::FRONT_RIGHT)
        .union(ChannelLayout::FRONT_CENTER)
        .union(ChannelLayout::LOW_FREQUENCY)
        .union(ChannelLayout::BACK_LEFT)
        .union(ChannelLayout::BACK_RIGHT)
        .union(ChannelLayout::SIDE_LEFT)
        .union(ChannelLayout::SIDE_RIGHT);

    /// Create a new empty channel layout
    pub const fn new() -> Self {
        Self::empty()
    }

    /// Check if a specific channel is present
    pub const fn has_channel(self, channel: ChannelLayout) -> bool {
        self.contains(channel)
    }

    /// Add a channel to the layout
    pub const fn with_channel(self, channel: ChannelLayout) -> Self {
        self.union(channel)
    }

    /// Remove a channel from the layout
    pub const fn without_channel(self, channel: ChannelLayout) -> Self {
        self.difference(channel)
    }

    /// Count the number of channels
    pub const fn channel_count(self) -> u32 {
        self.bits().count_ones()
    }
}

impl Display for ChannelLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut channels = Vec::new();
        for i in 0..64 {
            let channel = ChannelLayout::from_bits_truncate(1u64 << i);
            if self.contains(channel) {
                channels.push(format!("ch{}", i));
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
            x if x == ffi::VSMediaType::mtVideo as i32 => MediaType::Video,
            x if x == ffi::VSMediaType::mtAudio as i32 => MediaType::Audio,
            _ => unreachable!(),
        }
    }
}
