use rustsynth_sys as ffi;
use std::{marker::PhantomData, ptr::NonNull};

pub struct Function<'elem> {
    handle: NonNull<ffi::VSFunction>,
    _elem: PhantomData<&'elem ()>,
}

unsafe impl<'core> Send for Function<'core> {}
unsafe impl<'core> Sync for Function<'core> {}

impl<'elem> Function<'elem> {
    pub(crate) unsafe fn ptr(&self) -> *mut ffi::VSFunction {
        self.handle.as_ptr()
    }

    pub(crate) fn from_ptr(ptr: *const ffi::VSFunction) -> Self {
        Self {
            handle: unsafe { NonNull::new_unchecked(ptr as *mut ffi::VSFunction) },
            _elem: PhantomData,
        }
    }
}
