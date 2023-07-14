use std::ffi::CStr;

use bitflags::bitflags;

use pipewire_macro_impl::enum_wrapper;
use pipewire_proc_macro::RawWrapper;

pub mod buffer;
pub mod control;
pub mod events;

enum_wrapper!(
    State,
    pw_sys::pw_stream_state,
    ERROR: pw_sys::pw_stream_state_PW_STREAM_STATE_ERROR,
    UNCONNECTED: pw_sys::pw_stream_state_PW_STREAM_STATE_UNCONNECTED,
    CONNECTING: pw_sys::pw_stream_state_PW_STREAM_STATE_CONNECTING,
    PAUSED: pw_sys::pw_stream_state_PW_STREAM_STATE_PAUSED,
    STREAMING: pw_sys::pw_stream_state_PW_STREAM_STATE_STREAMING,
);

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct Flags: u32 {
        const NONE = pw_sys::pw_stream_flags_PW_STREAM_FLAG_NONE;
        const AUTOCONNECT = pw_sys::pw_stream_flags_PW_STREAM_FLAG_AUTOCONNECT;
        const INACTIVE = pw_sys::pw_stream_flags_PW_STREAM_FLAG_INACTIVE;
        const MAP_BUFFERS = pw_sys::pw_stream_flags_PW_STREAM_FLAG_MAP_BUFFERS;
        const DRIVER = pw_sys::pw_stream_flags_PW_STREAM_FLAG_DRIVER;
        const RT_PROCESS = pw_sys::pw_stream_flags_PW_STREAM_FLAG_RT_PROCESS;
        const NO_CONVERT = pw_sys::pw_stream_flags_PW_STREAM_FLAG_NO_CONVERT;
        const EXCLUSIVE = pw_sys::pw_stream_flags_PW_STREAM_FLAG_EXCLUSIVE;
        const DONT_RECONNECT = pw_sys::pw_stream_flags_PW_STREAM_FLAG_DONT_RECONNECT;
        const ALLOC_BUFFERS = pw_sys::pw_stream_flags_PW_STREAM_FLAG_ALLOC_BUFFERS;
        const TRIGGER = pw_sys::pw_stream_flags_PW_STREAM_FLAG_TRIGGER;
        // since 0.3.73 const ASYNC = pw_sys::pw_stream_flags_PW_STREAM_FLAG_ASYNC;
    }
}

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct StreamRef {
    #[raw]
    raw: pw_sys::pw_stream,
}

impl StreamRef {
    pub fn state_as_string<'a>(state: State) -> &'a CStr {
        unsafe { CStr::from_ptr(pw_sys::pw_stream_state_as_string(state.raw)) }
    }
}
