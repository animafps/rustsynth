//! Module for interacting with the VapourSynth API
use rustsynth_sys as ffi;
use std::{
    ffi::{c_char, c_int, c_void, CString},
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{
    core::{CoreRef, Plugins},
    plugin::Plugin,
};

use std::mem::MaybeUninit;

/// A wrapper for the VapourSynth API.
///
///
#[derive(Debug, Clone, Copy)]
pub(crate) struct API {
    // Note that this is *const, not *mut.
    handle: NonNull<ffi::VSAPI>,
}

unsafe impl Send for API {}
unsafe impl Sync for API {}

/// A cached API pointer. Note that this is `*const ffi::VSAPI`, not `*mut`.
static RAW_API: AtomicPtr<ffi::VSAPI> = AtomicPtr::new(ptr::null_mut());

// Macros for implementing repetitive functions.
macro_rules! map_get_something {
    ($name:ident, $func:ident, $rv:ty) => {
        #[inline]
        pub(crate) unsafe fn $name(
            self,
            map: &ffi::VSMap,
            key: *const c_char,
            index: i32,
            error: &mut i32,
        ) -> $rv {
            self.handle.as_ref().$func.unwrap()(map, key, index, error)
        }
    };
}

macro_rules! map_set_something {
    ($name:ident, $func:ident, $type:ty) => {
        #[inline]
        pub(crate) unsafe fn $name(
            self,
            map: &mut ffi::VSMap,
            key: *const c_char,
            value: $type,
            append: ffi::VSMapAppendMode,
        ) -> i32 {
            self.handle.as_ref().$func.unwrap()(map, key, value, append as i32)
        }
    };
}

impl API {
    /// Creates and or retrieves the VapourSynth API.
    ///
    /// Returns `None` on error
    #[cfg(all(feature = "vapoursynth-functions"))]
    #[inline]
    pub(crate) fn get() -> Option<Self> {
        // Check if we already have the API.
        let handle = RAW_API.load(Ordering::Relaxed);

        let handle = if handle.is_null() {
            // Attempt retrieving it otherwise.
            let handle =
                unsafe { ffi::getVapourSynthAPI(ffi::VAPOURSYNTH_API_VERSION.try_into().unwrap()) }
                    as *mut ffi::VSAPI;

            if !handle.is_null() {
                // If we successfully retrieved the API, cache it.
                RAW_API.store(handle, Ordering::Relaxed);
            }
            handle
        } else {
            handle
        };

        if handle.is_null() {
            None
        } else {
            Some(Self {
                handle: unsafe { NonNull::new_unchecked(handle) },
            })
        }
    }

    /// Returns the cached API.
    ///
    /// # Safety
    /// This function assumes the cache contains a valid API pointer.
    #[inline]
    pub(crate) unsafe fn get_cached() -> Self {
        Self {
            handle: NonNull::new_unchecked(RAW_API.load(Ordering::Relaxed)),
        }
    }

    pub(crate) unsafe fn create_core(&self, flags: i32) -> *mut ffi::VSCore {
        self.handle.as_ref().createCore.unwrap()(flags)
    }

    /// Creates a vapoursynth map and returns it
    pub(crate) fn create_map(&self) -> *mut ffi::VSMap {
        unsafe { self.handle.as_ref().createMap.unwrap()() }
    }

    pub(crate) unsafe fn free_core(&self, core: *mut ffi::VSCore) {
        self.handle.as_ref().freeCore.unwrap()(core)
    }

    pub(crate) unsafe fn free_func(&self, function: *mut ffi::VSFunction) {
        self.handle.as_ref().freeFunction.unwrap()(function)
    }

    pub(crate) fn plugins<'core>(&self, core: &'core CoreRef<'core>) -> Plugins<'core> {
        Plugins::new(core)
    }

    pub(crate) fn next_plugin<'core>(
        &self,
        plugin: Option<Plugin>,
        core: &CoreRef,
    ) -> Option<Plugin<'core>> {
        unsafe {
            let pluginptr = if let Some(value) = plugin {
                value.ptr()
            } else {
                ptr::null_mut()
            };
            let handle = self.handle.as_ref().getNextPlugin.unwrap()(pluginptr, core.ptr());
            if handle.is_null() {
                None
            } else {
                Some(Plugin::from_ptr(handle))
            }
        }
    }

    pub(crate) fn plugin_by_namespace<'core>(
        &self,
        namespace: &str,
        core: &CoreRef<'core>,
    ) -> Option<Plugin<'core>> {
        unsafe {
            let ns = CString::new(namespace).unwrap();
            let handle =
                self.handle.as_ref().getPluginByNamespace.unwrap()(ns.as_ptr(), core.ptr());
            if handle.is_null() {
                None
            } else {
                Some(Plugin::from_ptr(handle))
            }
        }
    }

    pub(crate) fn plugin_by_id<'core>(
        &self,
        id: &str,
        core: &'core CoreRef<'core>,
    ) -> Option<Plugin<'core>> {
        unsafe {
            let id = CString::new(id).unwrap();
            let handle = self.handle.as_ref().getPluginByID.unwrap()(id.as_ptr(), core.ptr());
            if handle.is_null() {
                None
            } else {
                Some(Plugin::from_ptr(handle))
            }
        }
    }

    pub(crate) fn version(&self) -> i32 {
        unsafe { self.handle.as_ref().getAPIVersion.unwrap()() }
    }

    pub(crate) unsafe fn get_plugin_path(&self, plugin: *mut ffi::VSPlugin) -> *const c_char {
        self.handle.as_ref().getPluginPath.unwrap()(plugin)
    }

    pub(crate) unsafe fn get_plugin_id(&self, plugin: *mut ffi::VSPlugin) -> *const c_char {
        self.handle.as_ref().getPluginID.unwrap()(plugin)
    }

    pub(crate) unsafe fn get_plugin_ns(&self, plugin: *mut ffi::VSPlugin) -> *const c_char {
        self.handle.as_ref().getPluginNamespace.unwrap()(plugin)
    }

    pub(crate) unsafe fn get_plugin_name(&self, plugin: *mut ffi::VSPlugin) -> *const c_char {
        self.handle.as_ref().getPluginName.unwrap()(plugin)
    }

    pub(crate) unsafe fn get_plugin_version(&self, plugin: *mut ffi::VSPlugin) -> i32 {
        self.handle.as_ref().getPluginVersion.unwrap()(plugin)
    }

    pub(crate) unsafe fn get_plugin_function_name(
        &self,
        function: *mut ffi::VSPluginFunction,
    ) -> *const c_char {
        self.handle.as_ref().getPluginFunctionName.unwrap()(function)
    }

    pub(crate) unsafe fn get_plugin_function_by_name(
        &self,
        name: *const c_char,
        plugin: *mut ffi::VSPlugin,
    ) -> *mut ffi::VSPluginFunction {
        self.handle.as_ref().getPluginFunctionByName.unwrap()(name, plugin)
    }

    pub(crate) unsafe fn get_next_plugin_function(
        &self,
        function: *mut ffi::VSPluginFunction,
        plugin: *mut ffi::VSPlugin,
    ) -> *mut ffi::VSPluginFunction {
        self.handle.as_ref().getNextPluginFunction.unwrap()(function, plugin)
    }

    pub(crate) unsafe fn get_plugin_function_arguments(
        &self,
        function: *mut ffi::VSPluginFunction,
    ) -> *const c_char {
        self.handle.as_ref().getPluginFunctionArguments.unwrap()(function)
    }

    pub(crate) unsafe fn get_core_info(&self, core: *mut ffi::VSCore) -> ffi::VSCoreInfo {
        let mut info = MaybeUninit::uninit();
        self.handle.as_ref().getCoreInfo.unwrap()(core, info.as_mut_ptr());
        info.assume_init()
    }

    pub(crate) unsafe fn invoke(
        &self,
        plugin: *mut ffi::VSPlugin,
        name: *const c_char,
        args: &ffi::VSMap,
    ) -> *mut ffi::VSMap {
        self.handle.as_ref().invoke.unwrap()(plugin, name, args)
    }

    pub(crate) unsafe fn clear_map(&self, map: &mut ffi::VSMap) {
        self.handle.as_ref().clearMap.unwrap()(map);
    }

    pub(crate) unsafe fn map_num_elements(&self, map: &ffi::VSMap, key: *const c_char) -> c_int {
        self.handle.as_ref().mapNumElements.unwrap()(map, key)
    }

    #[allow(unused)]
    pub(crate) unsafe fn copy_map(&self, map: *mut ffi::VSMap) -> *mut ffi::VSMap {
        let mut dest = MaybeUninit::uninit();
        self.handle.as_ref().copyMap.unwrap()(map, dest.as_mut_ptr());
        dest.as_mut_ptr()
    }

    pub(crate) unsafe fn map_num_keys(&self, map: &ffi::VSMap) -> c_int {
        self.handle.as_ref().mapNumKeys.unwrap()(map)
    }

    pub(crate) unsafe fn map_get_key(&self, map: &ffi::VSMap, index: c_int) -> *const c_char {
        self.handle.as_ref().mapGetKey.unwrap()(map, index)
    }

    pub(crate) unsafe fn free_map(&self, map: &mut ffi::VSMap) {
        self.handle.as_ref().freeMap.unwrap()(map)
    }

    pub(crate) unsafe fn map_get_type(&self, map: &ffi::VSMap, key: *const c_char) -> c_int {
        self.handle.as_ref().mapGetType.unwrap()(map, key)
    }

    pub(crate) unsafe fn map_get_int_array(
        &self,
        map: &ffi::VSMap,
        key: *const c_char,
        error: &mut i32,
    ) -> *const i64 {
        self.handle.as_ref().mapGetIntArray.unwrap()(map, key, error)
    }

    pub(crate) unsafe fn map_get_float_array(
        &self,
        map: &ffi::VSMap,
        key: *const c_char,
        error: &mut i32,
    ) -> *const f64 {
        self.handle.as_ref().mapGetFloatArray.unwrap()(map, key, error)
    }

    pub(crate) unsafe fn map_set_int_array(
        &self,
        map: *mut ffi::VSMap,
        key: *const c_char,
        int_array: *const i64,
        size: i32,
    ) -> i32 {
        self.handle.as_ref().mapSetIntArray.unwrap()(map, key, int_array, size)
    }

    pub(crate) unsafe fn map_set_float_array(
        &self,
        map: *mut ffi::VSMap,
        key: *const c_char,
        array: *const f64,
        size: i32,
    ) -> i32 {
        self.handle.as_ref().mapSetFloatArray.unwrap()(map, key, array, size)
    }

    pub(crate) unsafe fn get_node_type(&self, node: *mut ffi::VSNode) -> i32 {
        self.handle.as_ref().getNodeType.unwrap()(node)
    }

    pub(crate) unsafe fn get_video_info(&self, node: *mut ffi::VSNode) -> *const ffi::VSVideoInfo {
        self.handle.as_ref().getVideoInfo.unwrap()(node)
    }

    pub(crate) unsafe fn get_audio_info(&self, node: *mut ffi::VSNode) -> *const ffi::VSAudioInfo {
        self.handle.as_ref().getAudioInfo.unwrap()(node)
    }

    pub(crate) unsafe fn set_cache_mode(&self, node: *mut ffi::VSNode, mode: i32) {
        self.handle.as_ref().setCacheMode.unwrap()(node, mode)
    }

    pub(crate) unsafe fn set_cache_options(
        &self,
        node: *mut ffi::VSNode,
        fixed_size: i32,
        max_size: i32,
        max_history_size: i32,
    ) {
        self.handle.as_ref().setCacheOptions.unwrap()(node, fixed_size, max_size, max_history_size)
    }

    pub(crate) unsafe fn free_node(&self, node: *mut ffi::VSNode) {
        self.handle.as_ref().freeNode.unwrap()(node)
    }

    pub(crate) unsafe fn free_frame(&self, frame: *const ffi::VSFrame) {
        self.handle.as_ref().freeFrame.unwrap()(frame)
    }

    #[allow(unused)]
    pub(crate) unsafe fn copy_frame(
        &self,
        frame: &ffi::VSFrame,
        core: *mut ffi::VSCore,
    ) -> *const ffi::VSFrame {
        self.handle.as_ref().copyFrame.unwrap()(frame, core)
    }

    pub(crate) unsafe fn map_get_data_type_hint(
        &self,
        map: *mut ffi::VSMap,
        key: *const c_char,
        index: i32,
    ) -> i32 {
        let mut dest = MaybeUninit::uninit();
        self.handle.as_ref().mapGetDataTypeHint.unwrap()(map, key, index, dest.as_mut_ptr())
    }

    pub(crate) unsafe fn map_get_data_size(
        &self,
        map: &ffi::VSMap,
        key: *const c_char,
        index: i32,
        error: &mut i32,
    ) -> i32 {
        self.handle.as_ref().mapGetDataSize.unwrap()(map, key, index, error)
    }

    #[allow(unused)]
    pub(crate) unsafe fn map_set_empty(&self, map: *mut ffi::VSMap, key: *const c_char) -> i32 {
        self.handle.as_ref().mapSetEmpty.unwrap()(map, key, 0)
    }

    pub(crate) unsafe fn map_get_error(&self, map: &ffi::VSMap) -> *const c_char {
        self.handle.as_ref().mapGetError.unwrap()(map)
    }

    pub(crate) unsafe fn map_set_error(&self, map: &mut ffi::VSMap, error: *const c_char) {
        self.handle.as_ref().mapSetError.unwrap()(map, error)
    }

    pub(crate) unsafe fn set_thread_count(&self, core: *mut ffi::VSCore, count: i32) -> i32 {
        self.handle.as_ref().setThreadCount.unwrap()(count, core)
    }

    pub(crate) unsafe fn map_delete_key(&self, map: &mut ffi::VSMap, key: *const c_char) -> c_int {
        self.handle.as_ref().mapDeleteKey.unwrap()(map, key)
    }

    pub(crate) unsafe fn map_set_data(
        &self,
        map: &mut ffi::VSMap,
        key: *const c_char,
        value: &[u8],
        data_type: ffi::VSDataTypeHint,
        append: ffi::VSMapAppendMode,
    ) -> i32 {
        let length = value.len();
        assert!(length <= i32::max_value() as usize);
        let length = length as i32;

        self.handle.as_ref().mapSetData.unwrap()(
            map,
            key,
            value.as_ptr() as _,
            length,
            data_type as i32,
            append as i32,
        )
    }

    pub(crate) unsafe fn get_frame_width(&self, frame: &ffi::VSFrame, plane: i32) -> i32 {
        self.handle.as_ref().getFrameWidth.unwrap()(frame, plane)
    }

    pub(crate) unsafe fn get_frame_height(&self, frame: &ffi::VSFrame, plane: i32) -> i32 {
        self.handle.as_ref().getFrameHeight.unwrap()(frame, plane)
    }

    pub(crate) unsafe fn get_frame_length(&self, frame: &ffi::VSFrame) -> i32 {
        self.handle.as_ref().getFrameLength.unwrap()(frame)
    }

    pub(crate) unsafe fn get_frame_stride(&self, frame: &ffi::VSFrame, plane: i32) -> isize {
        self.handle.as_ref().getStride.unwrap()(frame, plane)
    }

    pub(crate) unsafe fn get_video_frame_format(
        &self,
        frame: &ffi::VSFrame,
    ) -> *const ffi::VSVideoFormat {
        self.handle.as_ref().getVideoFrameFormat.unwrap()(frame)
    }

    /// Creates a new frame, optionally copying the properties attached to another frame. The new
    /// frame contains uninitialised memory.
    ///
    /// # Safety
    /// The caller must ensure all pointers are valid and that the uninitialized plane data of the
    /// returned frame is handled carefully.
    #[inline]
    pub(crate) unsafe fn new_video_frame(
        self,
        format: *const ffi::VSVideoFormat,
        width: i32,
        height: i32,
        prop_src: *const ffi::VSFrame,
        core: *mut ffi::VSCore,
    ) -> *mut ffi::VSFrame {
        self.handle.as_ref().newVideoFrame.unwrap()(format, width, height, prop_src, core)
    }

    /// Creates a new video frame from the planes of existing frames, optionally copying the properties attached to another frame.
    #[inline]
    pub(crate) unsafe fn new_video_frame2(
        self,
        format: *const ffi::VSVideoFormat,
        width: i32,
        height: i32,
        planesrc: *mut *const ffi::VSFrame,
        planes: *const i32,
        prop_src: *const ffi::VSFrame,
        core: *mut ffi::VSCore,
    ) -> *mut ffi::VSFrame {
        self.handle.as_ref().newVideoFrame2.unwrap()(
            format, width, height, planesrc, planes, prop_src, core,
        )
    }

    /// Creates a new frame, optionally copying the properties attached to another frame. The new
    /// frame contains uninitialised memory.
    ///
    /// # Safety
    /// The caller must ensure all pointers are valid and that the uninitialized plane data of the
    /// returned frame is handled carefully.
    #[inline]
    pub(crate) unsafe fn new_audio_frame(
        self,
        format: *const ffi::VSAudioFormat,
        prop_src: *const ffi::VSFrame,
        num_samples: i32,
        core: *mut ffi::VSCore,
    ) -> *mut ffi::VSFrame {
        self.handle.as_ref().newAudioFrame.unwrap()(format, num_samples, prop_src, core)
    }

    /// Creates a new audio frame from the channels of existing frames, optionally copying the properties attached to another frame.
    #[inline]
    pub(crate) unsafe fn new_audio_frame2(
        self,
        format: *const ffi::VSAudioFormat,
        num_samples: i32,
        channel_src: *mut *const ffi::VSFrame,
        channels: *const i32,
        prop_src: *const ffi::VSFrame,
        core: *mut ffi::VSCore,
    ) -> *mut ffi::VSFrame {
        self.handle.as_ref().newAudioFrame2.unwrap()(
            format,
            num_samples,
            channel_src,
            channels,
            prop_src,
            core,
        )
    }

    pub(crate) unsafe fn clone_node(&self, node: *mut ffi::VSNode) -> *mut ffi::VSNode {
        self.handle.as_ref().addNodeRef.unwrap()(node)
    }

    pub(crate) unsafe fn get_frame_write_ptr(
        &self,
        frame: *mut ffi::VSFrame,
        plane: i32,
    ) -> *mut u8 {
        self.handle.as_ref().getWritePtr.unwrap()(frame, plane)
    }

    pub(crate) unsafe fn get_frame_read_ptr(&self, frame: &ffi::VSFrame, plane: i32) -> *const u8 {
        self.handle.as_ref().getReadPtr.unwrap()(frame, plane)
    }

    pub(crate) unsafe fn get_frame(
        &self,
        n: i32,
        node: *mut ffi::VSNode,
        err_msg: &mut [c_char],
    ) -> *const ffi::VSFrame {
        let len = err_msg.len();
        assert!(len <= i32::max_value() as usize);
        let len = len as i32;
        self.handle.as_ref().getFrame.unwrap()(n, node, err_msg.as_mut_ptr(), len)
    }

    #[allow(unused)]
    pub(crate) unsafe fn get_frame_props_ro(&self, frame: &ffi::VSFrame) -> *const ffi::VSMap {
        self.handle.as_ref().getFramePropertiesRO.unwrap()(frame)
    }

    #[allow(unused)]
    pub(crate) unsafe fn get_frame_props_rw(&self, frame: *mut ffi::VSFrame) -> *mut ffi::VSMap {
        self.handle.as_ref().getFramePropertiesRW.unwrap()(frame)
    }

    #[allow(non_snake_case)]
    pub(crate) unsafe fn get_frame_async(
        &self,
        n: i32,
        node: *mut ffi::VSNode,
        callback: Option<
            unsafe extern "C" fn(
                userData: *mut c_void,
                f: *const ffi::VSFrame,
                n: c_int,
                node: *mut ffi::VSNode,
                errorMsg: *const c_char,
            ),
        >,
        user_data: *mut c_void,
    ) {
        self.handle.as_ref().getFrameAsync.unwrap()(n, node, callback, user_data)
    }

    pub(crate) unsafe fn request_frame_filter(
        &self,
        n: i32,
        node: *mut ffi::VSNode,
        frame_ctx: *mut ffi::VSFrameContext,
    ) {
        self.handle.as_ref().requestFrameFilter.unwrap()(n, node, frame_ctx)
    }

    pub(crate) unsafe fn get_frame_filter(
        &self,
        n: i32,
        node: *mut ffi::VSNode,
        frame_ctx: *mut ffi::VSFrameContext,
    ) -> *const ffi::VSFrame {
        self.handle.as_ref().getFrameFilter.unwrap()(n, node, frame_ctx)
    }

    pub(crate) unsafe fn clone_func(&self, func: *mut ffi::VSFunction) -> *mut ffi::VSFunction {
        self.handle.as_ref().addFunctionRef.unwrap()(func)
    }

    pub(crate) unsafe fn create_func(
        &self,
        func: Option<
            unsafe extern "C" fn(
                *const ffi::VSMap,
                *mut ffi::VSMap,
                *mut c_void,
                *mut ffi::VSCore,
                *const ffi::VSAPI,
            ),
        >,
        user_data: *mut c_void,
        free_callback: Option<unsafe extern "C" fn(*mut c_void)>,
        core: *mut ffi::VSCore,
    ) -> *mut ffi::VSFunction {
        self.handle.as_ref().createFunction.unwrap()(func, user_data, free_callback, core)
    }

    pub(crate) unsafe fn call_func(
        &self,
        function: *mut ffi::VSFunction,
        in_map: &ffi::VSMap,
        out_map: &mut ffi::VSMap,
    ) {
        self.handle.as_ref().callFunction.unwrap()(function, in_map, out_map)
    }

    #[allow(unused)]
    pub(crate) unsafe fn create_video_filter(
        &self,
        out: *mut ffi::VSMap,
        name: *const ::std::os::raw::c_char,
        vi: *const ffi::VSVideoInfo,
        get_frame: ffi::VSFilterGetFrame,
        free: ffi::VSFilterFree,
        filter_mode: i32,
        dependencies: *const ffi::VSFilterDependency,
        num_deps: i32,
        instance_data: *mut ::std::os::raw::c_void,
        core: *mut ffi::VSCore,
    ) {
        self.handle.as_ref().createVideoFilter.unwrap()(
            out,
            name,
            vi,
            get_frame,
            free,
            filter_mode,
            dependencies,
            num_deps,
            instance_data,
            core,
        );
    }

    #[allow(unused)]
    pub(crate) unsafe fn create_video_filter2(
        &self,
        name: *const ::std::os::raw::c_char,
        vi: *const ffi::VSVideoInfo,
        get_frame: ffi::VSFilterGetFrame,
        free: ffi::VSFilterFree,
        filter_mode: i32,
        dependencies: *const ffi::VSFilterDependency,
        num_deps: i32,
        instance_data: *mut ::std::os::raw::c_void,
        core: *mut ffi::VSCore,
    ) -> *mut ffi::VSNode {
        self.handle.as_ref().createVideoFilter2.unwrap()(
            name,
            vi,
            get_frame,
            free,
            filter_mode,
            dependencies,
            num_deps,
            instance_data,
            core,
        )
    }

    #[allow(unused)]
    pub(crate) unsafe fn create_audio_filter2(
        &self,
        name: *const ::std::os::raw::c_char,
        ai: *const ffi::VSAudioInfo,
        get_frame: ffi::VSFilterGetFrame,
        free: ffi::VSFilterFree,
        filter_mode: i32,
        dependencies: *const ffi::VSFilterDependency,
        num_deps: i32,
        instance_data: *mut ::std::os::raw::c_void,
        core: *mut ffi::VSCore,
    ) -> *mut ffi::VSNode {
        self.handle.as_ref().createAudioFilter2.unwrap()(
            name,
            ai,
            get_frame,
            free,
            filter_mode,
            dependencies,
            num_deps,
            instance_data,
            core,
        )
    }

    pub(crate) fn get_audio_frame_format(
        &self,
        frame: *const ffi::VSFrame,
    ) -> *const ffi::VSAudioFormat {
        unsafe { self.handle.as_ref().getAudioFrameFormat.unwrap()(frame) }
    }

    pub(crate) fn get_audio_format_name(
        &self,
        format: *const ffi::VSAudioFormat,
    ) -> Option<String> {
        let buf: *mut i8 = std::ptr::null_mut();
        let result = unsafe { self.handle.as_ref().getAudioFormatName.unwrap()(format, buf) };
        if result == 0 {
            None
        } else {
            Some(unsafe { CString::from_raw(buf).to_string_lossy().into_owned() })
        }
    }

    pub(crate) fn cache_frame(
        &self,
        frame: *const ffi::VSFrame,
        n: i32,
        frame_ctx: *mut ffi::VSFrameContext,
    ) {
        unsafe { self.handle.as_ref().cacheFrame.unwrap()(frame, n, frame_ctx) }
    }

    pub(crate) fn set_linear_filter(&self, node: *mut ffi::VSNode) -> i32 {
        unsafe { self.handle.as_ref().setLinearFilter.unwrap()(node) }
    }

    pub(crate) fn get_frame_type(&self, frame: *const ffi::VSFrame) -> i32 {
        unsafe { self.handle.as_ref().getFrameType.unwrap()(frame) }
    }

    pub(crate) fn release_frame_early(
        &self,
        node: *mut ffi::VSNode,
        n: i32,
        frame_ctx: *mut ffi::VSFrameContext,
    ) {
        unsafe { self.handle.as_ref().releaseFrameEarly.unwrap()(node, n, frame_ctx) }
    }

    pub(crate) fn set_max_cache_size(&self, core: *mut ffi::VSCore, size: i64) -> i64 {
        unsafe { self.handle.as_ref().setMaxCacheSize.unwrap()(size, core) }
    }

    map_get_something!(map_get_int, mapGetInt, i64);
    map_get_something!(map_get_float, mapGetFloat, f64);
    map_get_something!(map_get_data, mapGetData, *const c_char);
    map_get_something!(map_get_node, mapGetNode, *mut ffi::VSNode);
    map_get_something!(map_get_frame, mapGetFrame, *const ffi::VSFrame);
    map_get_something!(map_get_func, mapGetFunction, *mut ffi::VSFunction);

    map_set_something!(map_set_int, mapSetInt, i64);
    map_set_something!(map_set_float, mapSetFloat, f64);
    map_set_something!(map_set_node, mapSetNode, *mut ffi::VSNode);
    map_set_something!(map_set_frame, mapSetFrame, *const ffi::VSFrame);
    map_set_something!(map_set_func, mapSetFunction, *mut ffi::VSFunction);
}

/// Initialize the global API pointer (for use in derive macros)
#[inline]
pub unsafe fn init_api(vsapi: *const ffi::VSAPI) {
    RAW_API.store(vsapi as *mut ffi::VSAPI, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let api = super::API::get().unwrap();
        let version = api.version();
        // VapourSynth API version should be a reasonable value
        // Version format is major << 16 | minor
        assert!(version >= 262144); // At least API version 4.0
        assert!(version < 327680); // Less than API version 5.0
    }
}
