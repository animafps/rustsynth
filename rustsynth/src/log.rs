use log::{debug, error, info, warn};
use rustsynth_sys as ffi;
use std::{
    ffi::{c_char, c_void, CStr},
    ptr::NonNull,
};

#[derive(Debug, Clone, Copy)]
pub enum MessageType {
    Debug = 0,
    Information = 1,
    Warning = 2,
    Critical = 3,
    Fatal = 4,
}

impl From<i32> for MessageType {
    fn from(value: i32) -> Self {
        match value {
            0 => MessageType::Debug,
            1 => MessageType::Information,
            2 => MessageType::Warning,
            3 => MessageType::Critical,
            4 => MessageType::Fatal,
            _ => MessageType::Debug, // fallback
        }
    }
}

impl Into<i32> for MessageType {
    fn into(self) -> i32 {
        match self {
            MessageType::Debug => 0,
            MessageType::Information => 1,
            MessageType::Warning => 2,
            MessageType::Critical => 3,
            MessageType::Fatal => 4,
        }
    }
}

pub struct LogHandle {
    /// mut
    handle: NonNull<ffi::VSLogHandle>,
    _handler: Box<dyn LogHandler>,
}

impl LogHandle {
    pub fn from_ptr(ptr: *mut ffi::VSLogHandle, handler: Box<dyn LogHandler>) -> Self {
        Self {
            handle: unsafe { NonNull::new_unchecked(ptr) },
            _handler: handler,
        }
    }
    pub fn as_ptr(&self) -> *mut ffi::VSLogHandle {
        self.handle.as_ptr()
    }
}

pub trait LogHandler: Send + Sync {
    fn handle(&self, msg_type: MessageType, msg: &str);
}

// C callback function that bridges to Rust LogHandler
pub(crate) unsafe extern "C" fn log_handler_callback(
    msg_type: i32,
    msg: *const c_char,
    userdata: *mut c_void,
) {
    if userdata.is_null() || msg.is_null() {
        return;
    }

    let handler = *(userdata as *const &dyn LogHandler);
    let message_type = MessageType::from(msg_type);

    if let Ok(message_str) = CStr::from_ptr(msg).to_str() {
        handler.handle(message_type, message_str);
    }
}

/// LogHandler Implementation using [`log`](https://github.com/rust-lang/log)
pub struct LogRS {}

impl LogHandler for LogRS {
    fn handle(&self, msg_type: MessageType, msg: &str) {
        match msg_type {
            MessageType::Debug => debug!("{}", msg),
            MessageType::Information => info!("{}", msg),
            MessageType::Warning => warn!("{}", msg),
            MessageType::Critical => error!("{}", msg),
            MessageType::Fatal => error!("{}", msg),
        }
    }
}
