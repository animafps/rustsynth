use crate::{
    core::CoreRef,
    filter::{FilterDependency, FilterMode},
    format::{AudioInfo, VideoInfo},
    frame::{Frame, FrameContext},
    map::Map,
};

/// Trait that filter structs must implement
pub trait Filter {
    const NAME: &'static str;
    const ARGS: &'static str;
    const RETURNTYPE: &'static str;
    const MODE: FilterMode;

    /// Create filter instance from input arguments and core
    fn from_args(args: &Map, core: &CoreRef) -> Result<Self, String>
    where
        Self: Sized;

    /// Get filter dependencies
    fn get_dependencies(&self) -> Vec<FilterDependency>;

    /// Get video info for video filters - override for source filters
    fn get_video_info(&self) -> Result<VideoInfo, String> {
        // Default: use first dependency's video info
        let deps = self.get_dependencies();
        if let Some(dep) = deps.first() {
            match dep.source.video_info() {
                Some(vi) => Ok(vi),
                None => Err("Input node has no video info".to_string()),
            }
        } else {
            Err("No dependencies and get_video_info not implemented".to_string())
        }
    }

    /// Get audio info for audio filters - override for source filters
    fn get_audio_info(&self) -> Result<AudioInfo, String> {
        // Default: use first dependency's audio info
        let deps = self.get_dependencies();
        if let Some(dep) = deps.first() {
            match dep.source.audio_info() {
                Some(ai) => Ok(ai),
                None => Err("Input node has no audio info".to_string()),
            }
        } else {
            Err("No dependencies and get_audio_info not implemented".to_string())
        }
    }

    /// Request input frames needed for processing frame n
    fn request_input_frames(&self, n: i32, frame_ctx: &FrameContext);

    /// Process frame n and return output frame
    fn process_frame<'core>(
        &mut self,
        n: i32,
        _frame_data: &[u8; 4],
        frame_ctx: &FrameContext,
        core: CoreRef<'core>,
    ) -> Result<Frame<'core>, String>;

    /// Clean up any frame-specific data
    fn cleanup_frame_data(&self, _frame_data: &[u8; 4]) {
        // Default: no cleanup needed
    }

    /// Clean up filter resources
    fn cleanup(&self) {
        // Default: no cleanup needed
    }
}
