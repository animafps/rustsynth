use rustsynth::{
    core::CoreRef,
    filter::{Filter, FilterDependency, FilterMode, RequestPattern},
    frame::{Frame, FrameContext},
    map::Map,
    node::Node,
    vapoursynth_plugin,
};

#[vapoursynth_plugin]
mod plugin {
    use rustsynth::{ffi, plugin::PluginConfigFlags, vapoursynth_filter, MakeVersion};
    const NAMESPACE: &str = "example";
    const ID: &str = "com.example.invert";
    const NAME: &str = "Example Plugin";
    const PLUGIN_VER: i32 = MakeVersion!(1, 0);
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

        fn request_input_frames(&self, n: i32, frame_ctx: &FrameContext) {
            self.get_dependencies()[0]
                .source
                .request_frame_filter(n, frame_ctx);
        }

        fn process_frame<'core>(
            &mut self,
            n: i32,
            _frame_data: &[u8; 4],
            frame_ctx: &FrameContext,
            core: CoreRef<'core>,
        ) -> Result<Frame<'core>, String> {
            let src = self.input_node.get_frame_filter(n, frame_ctx).unwrap();
            let vf = src.get_video_format().unwrap();
            let height = src.get_height(0);
            let width = src.get_width(0);
            let mut dst = Frame::new_video_frame(&core, width, height, &vf, Some(&src));

            // Actually do the invert operation
            for plane in 0..vf.num_planes {
                let mut srcp = src.get_read_ptr(plane);
                let mut dstp = dst.get_write_ptr(plane);
                let stride = src.get_stride(plane) as usize;
                let h = src.get_height(plane) as usize;
                let w = src.get_width(plane) as usize;

                unsafe {
                    for _y in 0..h {
                        for x in 0..w {
                            *dstp.add(x) = !(*srcp.add(x));
                        }
                        srcp = srcp.add(stride);
                        dstp = dstp.add(stride);
                    }
                }
            }
            Ok(dst)
        }
    }

    // Register all filters in this plugin
    rustsynth::register_filters!(Invert);
}
