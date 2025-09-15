use std::fmt::Display;

use rustsynth_sys as ffi;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum MediaType {
    Video,
    Audio,
}

/// Audio channel layout bitmask using VSAudioChannels constants
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct ChannelLayout(pub u64);

impl ChannelLayout {
    // Individual channel constants (1 << VSAudioChannels::*)
    pub const FRONT_LEFT: u64 = 1 << ffi::VSAudioChannels::acFrontLeft as u64;
    pub const FRONT_RIGHT: u64 = 1 << ffi::VSAudioChannels::acFrontRight as u64;
    pub const FRONT_CENTER: u64 = 1 << ffi::VSAudioChannels::acFrontCenter as u64;
    pub const LOW_FREQUENCY: u64 = 1 << ffi::VSAudioChannels::acLowFrequency as u64;
    pub const BACK_LEFT: u64 = 1 << ffi::VSAudioChannels::acBackLeft as u64;
    pub const BACK_RIGHT: u64 = 1 << ffi::VSAudioChannels::acBackRight as u64;
    pub const FRONT_LEFT_OF_CENTER: u64 = 1 << ffi::VSAudioChannels::acFrontLeftOFCenter as u64;
    pub const FRONT_RIGHT_OF_CENTER: u64 = 1 << ffi::VSAudioChannels::acFrontRightOFCenter as u64;
    pub const BACK_CENTER: u64 = 1 << ffi::VSAudioChannels::acBackCenter as u64;
    pub const SIDE_LEFT: u64 = 1 << ffi::VSAudioChannels::acSideLeft as u64;
    pub const SIDE_RIGHT: u64 = 1 << ffi::VSAudioChannels::acSideRight as u64;
    pub const TOP_CENTER: u64 = 1 << ffi::VSAudioChannels::acTopCenter as u64;
    pub const TOP_FRONT_LEFT: u64 = 1 << ffi::VSAudioChannels::acTopFrontLeft as u64;
    pub const TOP_FRONT_CENTER: u64 = 1 << ffi::VSAudioChannels::acTopFrontCenter as u64;
    pub const TOP_FRONT_RIGHT: u64 = 1 << ffi::VSAudioChannels::acTopFrontRight as u64;
    pub const TOP_BACK_LEFT: u64 = 1 << ffi::VSAudioChannels::acTopBackLeft as u64;
    pub const TOP_BACK_CENTER: u64 = 1 << ffi::VSAudioChannels::acTopBackCenter as u64;
    pub const TOP_BACK_RIGHT: u64 = 1 << ffi::VSAudioChannels::acTopBackRight as u64;
    pub const STEREO_LEFT: u64 = 1 << ffi::VSAudioChannels::acStereoLeft as u64;
    pub const STEREO_RIGHT: u64 = 1 << ffi::VSAudioChannels::acStereoRight as u64;
    pub const WIDE_LEFT: u64 = 1 << ffi::VSAudioChannels::acWideLeft as u64;
    pub const WIDE_RIGHT: u64 = 1 << ffi::VSAudioChannels::acWideRight as u64;
    pub const SURROUND_DIRECT_LEFT: u64 = 1 << ffi::VSAudioChannels::acSurroundDirectLeft as u64;
    pub const SURROUND_DIRECT_RIGHT: u64 = 1 << ffi::VSAudioChannels::acSurroundDirectRight as u64;
    pub const LOW_FREQUENCY_2: u64 = 1 << ffi::VSAudioChannels::acLowFrequency2 as u64;

    // Common layout combinations
    pub const MONO: ChannelLayout = ChannelLayout(Self::FRONT_LEFT);
    pub const STEREO: ChannelLayout = ChannelLayout(Self::FRONT_LEFT | Self::FRONT_RIGHT);
    pub const SURROUND_2_1: ChannelLayout =
        ChannelLayout(Self::FRONT_LEFT | Self::FRONT_RIGHT | Self::LOW_FREQUENCY);
    pub const SURROUND_3_0: ChannelLayout =
        ChannelLayout(Self::FRONT_LEFT | Self::FRONT_RIGHT | Self::FRONT_CENTER);
    pub const SURROUND_4_0: ChannelLayout =
        ChannelLayout(Self::FRONT_LEFT | Self::FRONT_RIGHT | Self::BACK_LEFT | Self::BACK_RIGHT);
    pub const SURROUND_4_1: ChannelLayout = ChannelLayout(
        Self::FRONT_LEFT
            | Self::FRONT_RIGHT
            | Self::BACK_LEFT
            | Self::BACK_RIGHT
            | Self::LOW_FREQUENCY,
    );
    pub const SURROUND_5_0: ChannelLayout = ChannelLayout(
        Self::FRONT_LEFT
            | Self::FRONT_RIGHT
            | Self::FRONT_CENTER
            | Self::BACK_LEFT
            | Self::BACK_RIGHT,
    );
    pub const SURROUND_5_1: ChannelLayout = ChannelLayout(
        Self::FRONT_LEFT
            | Self::FRONT_RIGHT
            | Self::FRONT_CENTER
            | Self::LOW_FREQUENCY
            | Self::BACK_LEFT
            | Self::BACK_RIGHT,
    );
    pub const SURROUND_7_1: ChannelLayout = ChannelLayout(
        Self::FRONT_LEFT
            | Self::FRONT_RIGHT
            | Self::FRONT_CENTER
            | Self::LOW_FREQUENCY
            | Self::BACK_LEFT
            | Self::BACK_RIGHT
            | Self::SIDE_LEFT
            | Self::SIDE_RIGHT,
    );

    /// Create a new empty channel layout
    pub const fn new() -> Self {
        Self(0)
    }

    /// Create from a raw bitmask
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    /// Get the raw bitmask
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// Check if a specific channel is present
    pub const fn has_channel(self, channel: u64) -> bool {
        (self.0 & channel) != 0
    }

    /// Add a channel to the layout
    pub const fn with_channel(self, channel: u64) -> Self {
        Self(self.0 | channel)
    }

    /// Remove a channel from the layout
    pub const fn without_channel(self, channel: u64) -> Self {
        Self(self.0 & !channel)
    }

    /// Count the number of channels
    pub const fn channel_count(self) -> u32 {
        self.0.count_ones()
    }

    /// Check if the layout is empty
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl std::ops::BitOr for ChannelLayout {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for ChannelLayout {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for ChannelLayout {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for ChannelLayout {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::BitXor for ChannelLayout {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for ChannelLayout {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl std::ops::Not for ChannelLayout {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl Display for ChannelLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut channels = Vec::new();
        for i in 0..64 {
            let channel = 1u64 << i;
            if self.has_channel(channel) {
                channels.push(format!("ch{}", i));
            }
        }
        write!(f, "ChannelLayout({})", channels.join(", "))
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
