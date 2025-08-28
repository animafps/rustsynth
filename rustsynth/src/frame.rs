use std::{marker::PhantomData, ops::Deref, ptr::NonNull};

use rustsynth_sys as ffi;

use crate::{
    api::API,
    core::CoreRef,
    format::{AudioFormat, VideoFormat},
    map::{MapRef, MapRefMut},
};

/// Chroma sample position in YUV formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromaLocation {
    Left = 0,
    Center = 1,
    TopLeft = 2,
    Top = 3,
    BottomLeft = 4,
    Bottom = 5,
}

/// Full or limited range (PC/TV range)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorRange {
    Full = 0,
    Limited = 1,
}

/// If the frame is composed of two independent fields (interlaced)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldBased {
    Progressive = 0,
    BottomFieldFirst = 1,
    TopFieldFirst = 2,
}

/// Which field was used to generate this frame
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Bottom = 0,
    Top = 1,
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transfer {
    Unknown(u32),
}

// One frame of a clip.
// This type is intended to be publicly used only in reference form.
#[derive(Debug)]
pub struct Frame<'core> {
    // The actual mutability of this depends on whether it's accessed via `&Frame` or `&mut Frame`.
    handle: NonNull<ffi::VSFrame>,
    _owner: PhantomData<&'core ()>,
}

unsafe impl<'core> Send for Frame<'core> {}
unsafe impl<'core> Sync for Frame<'core> {}

impl<'core> Drop for Frame<'core> {
    fn drop(&mut self) {
        unsafe { API::get_cached().free_frame(self.handle.as_ptr()) }
    }
}

/// Represents a reference to the obscure object
#[derive(Debug)]
pub struct FrameContext {
    handle: *mut ffi::VSFrameContext,
}

impl FrameContext {
    #[inline]
    #[allow(unused)]
    pub fn from_ptr(ptr: *mut ffi::VSFrameContext) -> Self {
        Self { handle: ptr }
    }

    #[inline]
    pub(crate) fn ptr(&self) -> *mut ffi::VSFrameContext {
        self.handle
    }
}

impl<'core> Frame<'core> {
    #[inline]
    pub fn from_ptr(ptr: *const ffi::VSFrame) -> Self {
        Self {
            handle: unsafe { NonNull::new_unchecked(ptr as *mut ffi::VSFrame) },
            _owner: PhantomData,
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const ffi::VSFrame {
        self.handle.as_ptr()
    }

    /// Returns the height of a plane of a given frame, in pixels. The height depends on the plane number because of the possible chroma subsampling.
    #[inline]
    pub fn get_height(&self, plane: i32) -> i32 {
        unsafe { API::get_cached().get_frame_height(self.handle.as_ref(), plane) }
    }

    /// Returns the width of a plane of a given frame, in pixels. The width depends on the plane number because of the possible chroma subsampling.
    #[inline]
    pub fn get_width(&self, plane: i32) -> i32 {
        unsafe { API::get_cached().get_frame_width(self.handle.as_ref(), plane) }
    }

    #[inline]
    pub fn get_length(&self) -> i32 {
        unsafe { API::get_cached().get_frame_length(self.handle.as_ref()) }
    }

    /// Returns the distance in bytes between two consecutive lines of a plane of a frame. The stride is always positive.
    ///
    /// Passing an invalid plane number will cause a fatal error.
    #[inline]
    pub fn get_stride(&self, plane: i32) -> isize {
        unsafe { API::get_cached().get_frame_stride(self.handle.as_ref(), plane) }
    }

    #[inline]
    pub fn get_video_format(&self) -> Option<VideoFormat> {
        let ptr = unsafe { API::get_cached().get_video_frame_format(self.handle.as_ref()) };
        if ptr.is_null() {
            None
        } else {
            Some(VideoFormat::from_ptr(ptr))
        }
    }

    pub fn get_audio_format(&self) -> Option<AudioFormat> {
        let ptr = unsafe { API::get_cached().get_audio_frame_format(self.handle.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { AudioFormat::from_ptr(ptr) })
        }
    }

    /// Creates a new video frame, optionally copying the properties attached to another frame.
    pub fn new_video_frame(
        core: &CoreRef,
        width: i32,
        height: i32,
        format: &VideoFormat,
        prop_src: Option<&Frame<'_>>,
    ) -> Self {
        let ptr = unsafe {
            API::get_cached().new_video_frame(
                &format.as_ptr() as *const ffi::VSVideoFormat,
                width,
                height,
                prop_src.map_or(std::ptr::null(), |f| f.as_ptr()),
                core.ptr(),
            )
        };
        if ptr.is_null() {
            panic!("Failed to create new video frame");
        }
        Frame::from_ptr(ptr)
    }

    /// Creates a new video frame from the planes of existing frames, optionally copying the properties attached to another frame
    pub fn new_video_frame_from_existing_planes<const T: usize>(
        core: &CoreRef,
        width: i32,
        height: i32,
        format: VideoFormat,
        planesrc: &mut [&Frame<'_>; T],
        planes: &[i32; T],
        propsrc: Option<&Frame<'_>>,
    ) -> Self {
        let ptr = unsafe {
            let mut planesrcptr: Vec<_> = planesrc.iter().map(|f| f.as_ptr()).collect();
            API::get_cached().new_video_frame2(
                &format.as_ptr() as *const ffi::VSVideoFormat,
                width,
                height,
                planesrcptr.as_mut_ptr(),
                planes.as_ptr(),
                propsrc.map_or(std::ptr::null(), |f| f.as_ptr()),
                core.ptr(),
            )
        };
        if ptr.is_null() {
            panic!("Failed to create new video frame from existing planes");
        }
        Frame::from_ptr(ptr)
    }

    /// Get read-only access to plane data
    #[inline]
    pub fn get_read_ptr(&self, plane: i32) -> *const u8 {
        unsafe { API::get_cached().get_frame_read_ptr(self.handle.as_ref(), plane) }
    }

    /// Get mutable access to plane data (only for owned frames)
    #[inline]
    pub fn get_write_ptr(&mut self, plane: i32) -> *mut u8 {
        unsafe { API::get_cached().get_frame_write_ptr(self.handle.as_ptr(), plane) }
    }

    /// Get read-only access to frame properties
    #[inline]
    pub fn properties(&self) -> MapRef<'_, 'core> {
        let map_ptr = unsafe { API::get_cached().get_frame_props_ro(self.handle.as_ref()) };
        unsafe { MapRef::from_ptr(map_ptr) }
    }

    /// Get read-write access to frame properties (only for owned frames)
    #[inline]
    pub fn properties_mut(&mut self) -> MapRefMut<'_, 'core> {
        let map_ptr = unsafe { API::get_cached().get_frame_props_rw(self.handle.as_ptr()) };
        unsafe { MapRefMut::from_ptr(map_ptr) }
    }

    // Standard frame property getters

    /// Get chroma sample position in YUV formats
    pub fn chroma_location(&self) -> Option<ChromaLocation> {
        self.properties()
            .get_int("_ChromaLocation")
            .ok()
            .and_then(|val| match val {
                0 => Some(ChromaLocation::Left),
                1 => Some(ChromaLocation::Center),
                2 => Some(ChromaLocation::TopLeft),
                3 => Some(ChromaLocation::Top),
                4 => Some(ChromaLocation::BottomLeft),
                5 => Some(ChromaLocation::Bottom),
                _ => None,
            })
    }

    /// Get color range (full or limited)
    pub fn color_range(&self) -> Option<ColorRange> {
        self.properties()
            .get_int("_ColorRange")
            .ok()
            .and_then(|val| match val {
                0 => Some(ColorRange::Full),
                1 => Some(ColorRange::Limited),
                _ => None,
            })
    }

    /// Get color primaries as specified in ITU-T H.273 Table 2
    pub fn primaries(&self) -> Option<i64> {
        self.properties().get_int("_Primaries").ok()
    }

    /// Get matrix coefficients as specified in ITU-T H.273 Table 4
    pub fn matrix(&self) -> Option<i64> {
        self.properties().get_int("_Matrix").ok()
    }

    /// Get transfer characteristics as specified in ITU-T H.273 Table 3
    pub fn transfer(&self) -> Option<i64> {
        self.properties().get_int("_Transfer").ok()
    }

    /// Get field based information (interlaced)
    pub fn field_based(&self) -> Option<FieldBased> {
        self.properties()
            .get_int("_FieldBased")
            .ok()
            .and_then(|val| match val {
                0 => Some(FieldBased::Progressive),
                1 => Some(FieldBased::BottomFieldFirst),
                2 => Some(FieldBased::TopFieldFirst),
                _ => None,
            })
    }

    /// Get absolute timestamp in seconds
    pub fn absolute_time(&self) -> Option<f64> {
        self.properties().get_float("_AbsoluteTime").ok()
    }

    /// Get frame duration as a rational number (numerator, denominator)
    pub fn duration(&self) -> Option<(i64, i64)> {
        let num = self.properties().get_int("_DurationNum").ok()?;
        let den = self.properties().get_int("_DurationDen").ok()?;
        Some((num, den))
    }

    /// Get whether the frame needs postprocessing
    pub fn combed(&self) -> Option<bool> {
        self.properties()
            .get_int("_Combed")
            .ok()
            .map(|val| val != 0)
    }

    /// Get which field was used to generate this frame
    pub fn field(&self) -> Option<Field> {
        self.properties()
            .get_int("_Field")
            .ok()
            .and_then(|val| match val {
                0 => Some(Field::Bottom),
                1 => Some(Field::Top),
                _ => None,
            })
    }

    /// Get picture type (single character describing frame type)
    pub fn picture_type(&self) -> Option<String> {
        self.properties().get_string("_PictType").ok()
    }

    /// Get pixel (sample) aspect ratio as a rational number (numerator, denominator)
    pub fn sample_aspect_ratio(&self) -> Option<(i64, i64)> {
        let num = self.properties().get_int("_SARNum").ok()?;
        let den = self.properties().get_int("_SARDen").ok()?;
        Some((num, den))
    }

    /// Get whether this frame is the last frame of the current scene
    pub fn scene_change_next(&self) -> Option<bool> {
        self.properties()
            .get_int("_SceneChangeNext")
            .ok()
            .map(|val| val != 0)
    }

    /// Get whether this frame starts a new scene
    pub fn scene_change_prev(&self) -> Option<bool> {
        self.properties()
            .get_int("_SceneChangePrev")
            .ok()
            .map(|val| val != 0)
    }

    /// Get alpha channel frame attached to this frame
    pub fn alpha(&self) -> Option<Frame<'core>> {
        self.properties().get_frame("_Alpha").ok()
    }

    // Standard frame property setters (for owned frames only)

    /// Set chroma sample position in YUV formats
    pub fn set_chroma_location(
        &mut self,
        location: ChromaLocation,
    ) -> Result<(), crate::map::Error> {
        self.properties_mut()
            .set_int("_ChromaLocation", location as i64)
    }

    /// Set color range (full or limited)
    pub fn set_color_range(&mut self, range: ColorRange) -> Result<(), crate::map::Error> {
        self.properties_mut().set_int("_ColorRange", range as i64)
    }

    /// Set color primaries as specified in ITU-T H.273 Table 2
    pub fn set_primaries(&mut self, primaries: i64) -> Result<(), crate::map::Error> {
        self.properties_mut().set_int("_Primaries", primaries)
    }

    /// Set matrix coefficients as specified in ITU-T H.273 Table 4
    pub fn set_matrix(&mut self, matrix: i64) -> Result<(), crate::map::Error> {
        self.properties_mut().set_int("_Matrix", matrix)
    }

    /// Set transfer characteristics as specified in ITU-T H.273 Table 3
    pub fn set_transfer(&mut self, transfer: i64) -> Result<(), crate::map::Error> {
        self.properties_mut().set_int("_Transfer", transfer)
    }

    /// Set field based information (interlaced)
    pub fn set_field_based(&mut self, field_based: FieldBased) -> Result<(), crate::map::Error> {
        self.properties_mut()
            .set_int("_FieldBased", field_based as i64)
    }

    /// Set absolute timestamp in seconds (should only be set by source filter)
    pub fn set_absolute_time(&mut self, time: f64) -> Result<(), crate::map::Error> {
        self.properties_mut().set_float("_AbsoluteTime", time)
    }

    /// Set frame duration as a rational number (numerator, denominator)
    pub fn set_duration(&mut self, num: i64, den: i64) -> Result<(), crate::map::Error> {
        self.properties_mut().set_int("_DurationNum", num)?;
        self.properties_mut().set_int("_DurationDen", den)
    }

    /// Set whether the frame needs postprocessing
    pub fn set_combed(&mut self, combed: bool) -> Result<(), crate::map::Error> {
        self.properties_mut()
            .set_int("_Combed", if combed { 1 } else { 0 })
    }

    /// Set which field was used to generate this frame
    pub fn set_field(&mut self, field: Field) -> Result<(), crate::map::Error> {
        self.properties_mut().set_int("_Field", field as i64)
    }

    /// Set picture type (single character describing frame type)
    pub fn set_picture_type(&mut self, pic_type: &str) -> Result<(), crate::map::Error> {
        self.properties_mut()
            .set_string(&"_PictType".to_string(), pic_type)
    }

    /// Set pixel (sample) aspect ratio as a rational number (numerator, denominator)
    pub fn set_sample_aspect_ratio(&mut self, num: i64, den: i64) -> Result<(), crate::map::Error> {
        self.properties_mut().set_int("_SARNum", num)?;
        self.properties_mut().set_int("_SARDen", den)
    }

    /// Set whether this frame is the last frame of the current scene
    pub fn set_scene_change_next(&mut self, scene_change: bool) -> Result<(), crate::map::Error> {
        self.properties_mut()
            .set_int("_SceneChangeNext", if scene_change { 1 } else { 0 })
    }

    /// Set whether this frame starts a new scene
    pub fn set_scene_change_prev(&mut self, scene_change: bool) -> Result<(), crate::map::Error> {
        self.properties_mut()
            .set_int("_SceneChangePrev", if scene_change { 1 } else { 0 })
    }

    /// Set alpha channel frame for this frame
    pub fn set_alpha(&mut self, alpha_frame: &Frame<'core>) -> Result<(), crate::map::Error> {
        self.properties_mut().set_frame("_Alpha", alpha_frame)
    }
}

impl<'core> Deref for Frame<'core> {
    type Target = ffi::VSFrame;

    fn deref(&self) -> &Self::Target {
        unsafe { self.handle.as_ref() }
    }
}
