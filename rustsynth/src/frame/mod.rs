mod enums;

use std::{ffi::CStr, marker::PhantomData, ops::Deref, ptr::NonNull};

use rustsynth_sys as ffi;

use crate::{
    api::API,
    core::CoreRef,
    format::{AudioFormat, MediaType, VideoFormat},
    map::{MapRef, MapRefMut, MapResult},
};

// One frame of a clip.
// This type is intended to be publicly used only in reference form.
#[derive(Debug, Clone)]
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

const CHROMA_LOCATION_KEY: &CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"_ChromaLocation\0") };
const COLOR_RANGE_KEY: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"_ColorRange\0") };
const PRIMARIES_KEY: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"_Primaries\0") };
const MATRIX_KEY: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"_Matrix\0") };
const TRANSFER_KEY: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"_Transfer\0") };
const FIELD_BASED_KEY: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"_FieldBased\0") };
const ABSOLUTE_TIME_KEY: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"_AbsoluteTime\0") };
const DURATION_NUM_KEY: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"_DurationNum\0") };
const DURATION_DEN_KEY: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"_DurationDen\0") };
const COMBED_KEY: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"_Combed\0") };
const FIELD_KEY: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"_Field\0") };
const PICT_TYPE_KEY: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"_PictType\0") };
const SAR_NUM_KEY: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"_SARNum\0") };
const SAR_DEN_KEY: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"_SARDen\0") };
const SCENE_CHANGE_NEXT_KEY: &CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"_SceneChangeNext\0") };
const SCENE_CHANGE_PREV_KEY: &CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"_SceneChangePrev\0") };
const ALPHA_KEY: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"_Alpha\0") };

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
            let mut planesrcptr: [*const ffi::VSFrame; T] = [std::ptr::null(); T];
            for (i, frame) in planesrc.iter().enumerate() {
                planesrcptr[i] = frame.as_ptr();
            }
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
        Frame::from_ptr(ptr)
    }

    /// Creates a new audio frame, optionally copying the properties attached to another frame. It is a fatal error to pass invalid arguments to this function
    pub fn new_audio_frame(
        core: &CoreRef,
        length: i32,
        format: &AudioFormat,
        prop_src: Option<&Frame<'_>>,
    ) -> Self {
        let ptr = unsafe {
            API::get_cached().new_audio_frame(
                &format.as_ptr() as *const ffi::VSAudioFormat,
                prop_src.map_or(std::ptr::null(), |f| f.as_ptr()),
                length,
                core.ptr(),
            )
        };
        Frame::from_ptr(ptr)
    }

    /// Creates a new audio frame, optionally copying the properties attached to another frame. It is a fatal error to pass invalid arguments to this function.
    ///
    /// See also [Frame::new_video_frame_from_existing_planes]
    pub fn new_audio_frame_from_existing_channels<const T: usize>(
        core: &CoreRef,
        num_samples: i32,
        format: &AudioFormat,
        channelsrc: &mut [&Frame<'_>; T],
        channels: &[i32; T],
        propsrc: Option<&Frame<'_>>,
    ) -> Self {
        let ptr = unsafe {
            let mut channelsrcptr: [*const ffi::VSFrame; T] = [std::ptr::null(); T];
            for (i, frame) in channelsrc.iter().enumerate() {
                channelsrcptr[i] = frame.as_ptr();
            }
            API::get_cached().new_audio_frame2(
                &format.as_ptr() as *const ffi::VSAudioFormat,
                num_samples,
                channelsrcptr.as_mut_ptr(),
                channels.as_ptr(),
                propsrc.map_or(std::ptr::null(), |f| f.as_ptr()),
                core.ptr(),
            )
        };
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
        unsafe {
            self.properties()
                .get_int_raw_unchecked(CHROMA_LOCATION_KEY, 0)
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
    }

    /// Get color range (full or limited)
    pub fn color_range(&self) -> Option<ColorRange> {
        unsafe {
            self.properties()
                .get_int_raw_unchecked(COLOR_RANGE_KEY, 0)
                .ok()
                .and_then(|val| match val {
                    0 => Some(ColorRange::Full),
                    1 => Some(ColorRange::Limited),
                    _ => None,
                })
        }
    }

    /// Get color primaries as specified in ITU-T H.273 Table 2
    pub fn primaries(&self) -> ColorPrimaries {
        let res = unsafe {
            self.properties()
                .get_int_raw_unchecked(PRIMARIES_KEY, 0)
                .unwrap_or(2)
        };
        ColorPrimaries::from(res)
    }

    /// Get matrix coefficients as specified in ITU-T H.273 Table 4
    pub fn matrix(&self) -> MatrixCoefficients {
        let res = unsafe {
            self.properties()
                .get_int_raw_unchecked(MATRIX_KEY, 0)
                .unwrap_or(2)
        };
        MatrixCoefficients::from(res)
    }

    /// Get transfer characteristics as specified in ITU-T H.273 Table 3
    pub fn transfer(&self) -> TransferCharacteristics {
        let res = unsafe { self.properties().get_int_raw_unchecked(TRANSFER_KEY, 2) }.unwrap_or(0);
        TransferCharacteristics::from(res)
    }

    /// Get field based information (interlaced)
    pub fn field_based(&self) -> Option<FieldBased> {
        unsafe {
            self.properties()
                .get_int_raw_unchecked(FIELD_BASED_KEY, 0)
                .ok()
                .and_then(|val| match val {
                    0 => Some(FieldBased::Progressive),
                    1 => Some(FieldBased::BottomFieldFirst),
                    2 => Some(FieldBased::TopFieldFirst),
                    _ => None,
                })
        }
    }

    /// Get absolute timestamp in seconds
    pub fn absolute_time(&self) -> Option<f64> {
        unsafe {
            self.properties()
                .get_float_raw_unchecked(ABSOLUTE_TIME_KEY, 0)
                .ok()
        }
    }

    /// Get frame duration as a rational number (numerator, denominator)
    pub fn duration(&self) -> Option<(i64, i64)> {
        let num = unsafe {
            self.properties()
                .get_int_raw_unchecked(DURATION_NUM_KEY, 0)
                .ok()?
        };
        let den = unsafe {
            self.properties()
                .get_int_raw_unchecked(DURATION_DEN_KEY, 0)
                .ok()?
        };
        Some((num, den))
    }

    /// Get whether the frame needs postprocessing
    pub fn combed(&self) -> Option<bool> {
        unsafe {
            self.properties()
                .get_int_raw_unchecked(COMBED_KEY, 0)
                .ok()
                .map(|val| val != 0)
        }
    }

    /// Get which field was used to generate this frame
    pub fn field(&self) -> Option<Field> {
        unsafe {
            self.properties()
                .get_int_raw_unchecked(FIELD_KEY, 0)
                .ok()
                .and_then(|val| match val {
                    0 => Some(Field::Bottom),
                    1 => Some(Field::Top),
                    _ => None,
                })
        }
    }

    /// Get picture type (single character describing frame type)
    pub fn picture_type(&self) -> Option<String> {
        unsafe {
            self.properties()
                .get_string_raw_unchecked(PICT_TYPE_KEY, 0)
                .ok()
        }
    }

    /// Get pixel (sample) aspect ratio as a rational number (numerator, denominator)
    pub fn sample_aspect_ratio(&self) -> Option<(i64, i64)> {
        let num = unsafe {
            self.properties()
                .get_int_raw_unchecked(SAR_NUM_KEY, 0)
                .ok()?
        };
        let den = unsafe {
            self.properties()
                .get_int_raw_unchecked(SAR_DEN_KEY, 0)
                .ok()?
        };
        Some((num, den))
    }

    /// Get whether this frame is the last frame of the current scene
    pub fn scene_change_next(&self) -> Option<bool> {
        unsafe {
            self.properties()
                .get_int_raw_unchecked(SCENE_CHANGE_NEXT_KEY, 0)
                .ok()
                .map(|val| val != 0)
        }
    }

    /// Get whether this frame starts a new scene
    pub fn scene_change_prev(&self) -> Option<bool> {
        unsafe {
            self.properties()
                .get_int_raw_unchecked(SCENE_CHANGE_PREV_KEY, 0)
                .ok()
                .map(|val| val != 0)
        }
    }

    /// Get alpha channel frame attached to this frame
    pub fn alpha(&self) -> Option<Frame<'core>> {
        unsafe { self.properties().get_frame_raw_unchecked(ALPHA_KEY, 0).ok() }
    }

    // Standard frame property setters (for owned frames only)

    /// Set chroma sample position in YUV formats
    pub fn set_chroma_location(
        &mut self,
        location: ChromaLocation,
    ) -> MapResult<()> {
        unsafe {
            self.properties_mut()
                .set_int_raw_unchecked(CHROMA_LOCATION_KEY, location as i64);
        }
        Ok(())
    }

    /// Set color range (full or limited)
    pub fn set_color_range(&mut self, range: ColorRange) -> MapResult<()> {
        unsafe {
            self.properties_mut()
                .set_int_raw_unchecked(COLOR_RANGE_KEY, range as i64);
        }
        Ok(())
    }

    /// Set color primaries as specified in ITU-T H.273 Table 2
    pub fn set_primaries(&mut self, primaries: ColorPrimaries) -> MapResult<()> {
        unsafe {
            self.properties_mut()
                .set_int_raw_unchecked(PRIMARIES_KEY, primaries as i64);
        }
        Ok(())
    }

    /// Set matrix coefficients as specified in ITU-T H.273 Table 4
    pub fn set_matrix(&mut self, matrix: MatrixCoefficients) -> MapResult<()> {
        unsafe {
            self.properties_mut()
                .set_int_raw_unchecked(MATRIX_KEY, matrix as i64);
        }
        Ok(())
    }

    /// Set transfer characteristics as specified in ITU-T H.273 Table 3
    pub fn set_transfer(
        &mut self,
        transfer: TransferCharacteristics,
    ) -> MapResult<()> {
        unsafe {
            self.properties_mut()
                .set_int_raw_unchecked(TRANSFER_KEY, transfer as i64);
        }
        Ok(())
    }

    /// Set field based information (interlaced)
    pub fn set_field_based(&mut self, field_based: FieldBased) -> MapResult<()> {
        unsafe {
            self.properties_mut()
                .set_int_raw_unchecked(FIELD_BASED_KEY, field_based as i64);
        }
        Ok(())
    }

    /// Set absolute timestamp in seconds (should only be set by source filter)
    pub fn set_absolute_time(&mut self, time: f64) -> MapResult<()> {
        unsafe {
            self.properties_mut()
                .set_float_raw_unchecked(ABSOLUTE_TIME_KEY, time);
        }
        Ok(())
    }

    /// Set frame duration as a rational number (numerator, denominator)
    pub fn set_duration(&mut self, num: i64, den: i64) -> MapResult<()> {
        unsafe {
            self.properties_mut()
                .set_int_raw_unchecked(DURATION_NUM_KEY, num);
            self.properties_mut()
                .set_int_raw_unchecked(DURATION_DEN_KEY, den);
        }
        Ok(())
    }

    /// Set whether the frame needs postprocessing
    pub fn set_combed(&mut self, combed: bool) -> MapResult<()> {
        unsafe {
            self.properties_mut()
                .set_int_raw_unchecked(COMBED_KEY, if combed { 1 } else { 0 });
        }
        Ok(())
    }

    /// Set which field was used to generate this frame
    pub fn set_field(&mut self, field: Field) -> MapResult<()> {
        unsafe {
            self.properties_mut()
                .set_int_raw_unchecked(FIELD_KEY, field as i64);
        }
        Ok(())
    }

    /// Set picture type (single character describing frame type)
    pub fn set_picture_type(&mut self, pic_type: &str) -> MapResult<()> {
        unsafe {
            self.properties_mut()
                .set_string_raw_unchecked(PICT_TYPE_KEY, pic_type);
        }
        Ok(())
    }

    /// Set pixel (sample) aspect ratio as a rational number (numerator, denominator)
    pub fn set_sample_aspect_ratio(&mut self, num: i64, den: i64) -> MapResult<()> {
        unsafe {
            self.properties_mut()
                .set_int_raw_unchecked(SAR_NUM_KEY, num);
            self.properties_mut()
                .set_int_raw_unchecked(SAR_DEN_KEY, den);
        }
        Ok(())
    }

    /// Set whether this frame is the last frame of the current scene
    pub fn set_scene_change_next(&mut self, scene_change: bool) -> MapResult<()> {
        unsafe {
            self.properties_mut()
                .set_int_raw_unchecked(SCENE_CHANGE_NEXT_KEY, if scene_change { 1 } else { 0 });
        }
        Ok(())
    }

    /// Set whether this frame starts a new scene
    pub fn set_scene_change_prev(&mut self, scene_change: bool) -> MapResult<()> {
        unsafe {
            self.properties_mut()
                .set_int_raw_unchecked(SCENE_CHANGE_PREV_KEY, if scene_change { 1 } else { 0 });
        }
        Ok(())
    }

    /// Set alpha channel frame for this frame
    pub fn set_alpha(&mut self, alpha_frame: &Frame<'core>) -> MapResult<()> {
        unsafe {
            self.properties_mut()
                .set_frame_raw_unchecked(ALPHA_KEY, alpha_frame);
        }
        Ok(())
    }

    pub fn get_frame_type(&self) -> MediaType {
        MediaType::from_ffi(unsafe { API::get_cached().get_frame_type(self.handle.as_ref()) })
    }

    /// Pushes a not requested frame into the cache. This is useful for (source) filters that greatly benefit from completely linear access and producing all output in linear order.
    /// This function may only be used in filters that were created with setLinearFilter.
    /// Only use inside a filter’s “getframe” function.
    pub fn cache_frame(&self, n: i32, frame_ctxt: &FrameContext) {
        unsafe { API::get_cached().cache_frame(self.handle.as_ref(), n, frame_ctxt.ptr()) }
    }

    /// RAII fn that provides access to all planes of a video frame
    pub fn with_planes<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Plane]) -> R,
    {
        let num_planes = self.get_video_format().map_or(0, |vf| vf.num_planes);
        let mut planes = Vec::with_capacity(num_planes as usize);
        for i in 0..num_planes {
            let plane = Plane {
                data: self.get_read_ptr(i),
                stride: self.get_stride(i),
                width: self.get_width(i),
                height: self.get_height(i),
            };
            planes.push(plane);
        }
        f(&planes)
    }

    /// RAII fn that provides mutable access to all planes of a video frame (only for owned frames)
    pub fn map_pixels<T, F>(&mut self, plane: i32, mut f: F)
    where
        F: FnMut(&mut [T]),
    {
        let ptr = self.get_write_ptr(plane) as *mut T;
        let stride = self.get_stride(plane) as isize / std::mem::size_of::<T>() as isize;
        let width = self.get_width(plane) as isize;
        let height = self.get_height(plane) as isize;
        unsafe {
            for row in 0..height {
                let row_ptr = ptr.offset(row * stride);
                let slice = std::slice::from_raw_parts_mut(row_ptr, width as usize);
                f(slice);
            }
        }
    }
}

impl<'core> Deref for Frame<'core> {
    type Target = ffi::VSFrame;

    fn deref(&self) -> &Self::Target {
        unsafe { self.handle.as_ref() }
    }
}
pub struct Plane {
    pub data: *const u8,
    pub stride: isize,
    pub width: i32,
    pub height: i32,
}

pub use enums::{
    ChromaLocation, ColorPrimaries, ColorRange, Field, FieldBased, MatrixCoefficients,
    TransferCharacteristics,
};
