use std::ffi::CStr;
use std::marker::PhantomData;

use bitflags::bitflags;

use pipewire_macro_impl::enum_wrapper;
use pipewire_proc_macro::RawWrapper;

pub mod events;

/// Wrapper for the external [pw_sys::pw_core] value.
/// The filter object provides a convenient way to implement processing filters.
#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct FilterRef<T> {
    #[raw]
    raw: pw_sys::pw_filter,
    phantom: PhantomData<T>,
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct FilterFlags: u32 {
        const NONE = pw_sys::pw_filter_flags_PW_FILTER_FLAG_NONE;
        const INACTIVE = pw_sys::pw_filter_flags_PW_FILTER_FLAG_INACTIVE;
        const DRIVER = pw_sys::pw_filter_flags_PW_FILTER_FLAG_DRIVER;
        const RT_PROCESS = pw_sys::pw_filter_flags_PW_FILTER_FLAG_RT_PROCESS;
        const CUSTOM_LATENCY = pw_sys::pw_filter_flags_PW_FILTER_FLAG_CUSTOM_LATENCY;
    }
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct PortFlags: u32 {
        const NONE = pw_sys::pw_filter_port_flags_PW_FILTER_PORT_FLAG_NONE;
        const ALLOC_BUFFERS = pw_sys::pw_filter_port_flags_PW_FILTER_PORT_FLAG_ALLOC_BUFFERS;
        const MAP_BUFFERS = pw_sys::pw_filter_port_flags_PW_FILTER_PORT_FLAG_MAP_BUFFERS;
    }
}

enum_wrapper!(
    FilterState,
    pw_sys::pw_filter_state,
    ERROR: pw_sys::pw_filter_state_PW_FILTER_STATE_ERROR,
    UNCONNECTED: pw_sys::pw_filter_state_PW_FILTER_STATE_UNCONNECTED,
    CONNECTING: pw_sys::pw_filter_state_PW_FILTER_STATE_CONNECTING,
    PAUSED: pw_sys::pw_filter_state_PW_FILTER_STATE_PAUSED,
    STREAMING: pw_sys::pw_filter_state_PW_FILTER_STATE_STREAMING,
);

impl FilterState {
    pub fn state_as_string(&self) -> &CStr {
        unsafe { CStr::from_ptr(pw_sys::pw_filter_state_as_string(self.raw)) }
    }
}

impl<T> FilterRef<T> {}

// impl<'a> AddListener<'a> for FilterRef {
//     type Events = FilterEvents<'a>;
//
//     fn add_listener(&self, events: Pin<Box<Self::Events>>) -> Pin<Box<Self::Events>> {
//         unsafe {
//             pw_sys::pw_filter_add_listener(
//                 self.as_raw_ptr(),
//                 events.hook().as_raw_ptr(),
//                 events.as_raw_ptr(),
//                 &*events as *const _ as *mut _,
//             )
//         };
//
//         events
//     }
// }
