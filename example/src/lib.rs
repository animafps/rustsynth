use rustsynth::{
    core::CoreRef,
    filter::{
        traits::{Filter},
        FilterDependency, FilterMode, RequestPattern,
    },
    frame::{Frame, FrameContext},
    map::Map,
    node::Node,
};
use rustsynth_derive::vapoursynth_plugin;

#[vapoursynth_plugin]
mod plugin {
    use rustsynth::{ffi, plugin::PluginConfigFlags, MakeVersion};
    use rustsynth_derive::vapoursynth_filter;
    const NAMESPACE: &'static str = "example";
    const ID: &'static str = "com.example.invert";
    const NAME: &'static str = "Example Plugin";
    const PLUGIN_VER: i32 = MakeVersion!(1,0);
    const API_VER: i32 = ffi::VAPOURSYNTH_API_VERSION;
    const FLAGS: i32 = PluginConfigFlags::NONE.bits();

    #[vapoursynth_filter(video)]
    struct Invert {
        input_node: Node,
    }

    // Just implement the trait methods and the macro handles all C FFI
    impl Filter for Invert {
        const NAME: &'static str = "Invert";
        const ARGS: &'static str = "clip:vnode;";
        const RETURNTYPE: &'static str = "clip:vnode;";
        const MODE: FilterMode = FilterMode::Parallel;

        fn from_args(args: &Map, _core: &CoreRef) -> Result<Self, String> {
            let input_node = args.get_node("clip")?;
            Ok(Self { input_node })
        }

        fn get_dependencies(&self) -> Vec<FilterDependency> {
            vec![FilterDependency {
                source: self.input_node.clone(),
                request_pattern: RequestPattern::StrictSpatial,
            }]
        }

        fn request_input_frames(&self, n: i32, frame_ctx: FrameContext) {
            self.get_dependencies()[0]
                .source
                .request_frame_filter(n, &frame_ctx);
        }

        fn process_frame<'core>(
            &mut self,
            n: i32,
            _frame_data: &[u8; 4],
            frame_ctx: FrameContext,
            core: CoreRef<'core>,
        ) -> Result<Frame<'core>, String> {
            let src = self.input_node.get_frame_filter(n, &frame_ctx).unwrap();
            // simple pass through
            Ok(src)
        }
    }

    // Register all filters in this plugin
    rustsynth::register_filters!(Invert);
}
