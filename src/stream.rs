use std::ffi::{CStr, CString};
use std::pin::Pin;
use std::ptr::{null_mut, NonNull};

use bitflags::{bitflags, Flags};
use spa_sys::spa_pod;

use pipewire_macro_impl::{enum_wrapper, spa_interface_call};
use pipewire_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::core::CoreRef;
use crate::core_api::properties::PropertiesRef;
use crate::core_api::PW_ID_ANY;
use crate::listeners::{AddListener, Listeners};
use crate::spa::dict::DictRef;
use crate::spa::pod::object::param_port_config::Direction;
use crate::spa::pod::PodRef;
use crate::stream::buffer::BufferRef;
use crate::stream::control::ControlRef;
use crate::stream::events::StreamEvents;
use crate::stream::time::TimeRef;
use crate::wrapper::RawWrapper;
use crate::{i32_as_result, i32_as_void_result, new_instance_raw_wrapper, raw_wrapper};

pub mod buffer;
pub mod control;
pub mod events;
pub mod time;

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
    pub struct StreamFlags: u32 {
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

#[derive(RawWrapper, Debug, Clone)]
#[repr(transparent)]
pub struct StreamRef {
    #[raw]
    raw: pw_sys::pw_stream,
}

impl StreamRef {
    pub fn state_as_string<'a>(state: State) -> &'a CStr {
        unsafe { CStr::from_ptr(pw_sys::pw_stream_state_as_string(state.raw)) }
    }

    pub fn get_state_and_error(&self) -> (State, Option<CString>) {
        let mut error_ptr = null_mut();
        unsafe {
            let state = State::from_raw(pw_sys::pw_stream_get_state(self.as_raw_ptr(), error_ptr));
            let error_c_str = error_ptr.as_mut().map(|ptr| CStr::from_ptr(*ptr));
            let error = error_c_str.map(|c_str| CString::from(c_str));
            (state, error)
        }
    }

    pub fn get_state(&self) -> State {
        self.get_state_and_error().0
    }

    pub fn get_error(&self) -> Option<CString> {
        self.get_state_and_error().1
    }

    pub fn get_name(&self) -> Option<&CStr> {
        unsafe {
            pw_sys::pw_stream_get_name(self.as_raw_ptr())
                .as_ref()
                .map(|ptr| CStr::from_ptr(ptr))
        }
    }

    pub fn get_core(&self) -> &CoreRef {
        unsafe { CoreRef::from_raw_ptr(pw_sys::pw_stream_get_core(self.as_raw_ptr())) }
    }

    pub fn get_properties(&self) -> &PropertiesRef {
        unsafe { PropertiesRef::from_raw_ptr(pw_sys::pw_stream_get_properties(self.as_raw_ptr())) }
    }

    pub fn update_properties(&self, properties: &DictRef) -> i32 {
        unsafe { pw_sys::pw_stream_update_properties(self.as_raw_ptr(), properties.as_raw_ptr()) }
    }

    pub fn connect(
        &self,
        direction: Direction,
        flags: StreamFlags,
        params: &[&PodRef],
    ) -> crate::Result<()> {
        let result = unsafe {
            let params_ptr = params as *const [&PodRef] as *mut *const spa_pod;
            pw_sys::pw_stream_connect(
                self.as_raw_ptr(),
                direction.raw,
                PW_ID_ANY,
                flags.bits(),
                params_ptr,
                params.len() as u32,
            )
        };
        i32_as_void_result(result)
    }

    pub fn get_node_id(&self) -> u32 {
        unsafe { pw_sys::pw_stream_get_node_id(self.as_raw_ptr()) }
    }

    pub fn disconnect(&self) -> crate::Result<()> {
        let result = unsafe { pw_sys::pw_stream_disconnect(self.as_raw_ptr()) };
        i32_as_void_result(result)
    }

    pub fn set_error(&self, res: i32, error: &CStr) -> crate::Result<()> {
        let result = unsafe { pw_sys::pw_stream_set_error(self.as_raw_ptr(), res, error.as_ptr()) };
        i32_as_void_result(result)
    }

    pub fn update_params(&self, params: &[&PodRef]) -> crate::Result<()> {
        let result = unsafe {
            let params_ptr = params as *const [&PodRef] as *mut *const spa_pod;
            pw_sys::pw_stream_update_params(self.as_raw_ptr(), params_ptr, params.len() as u32)
        };
        i32_as_void_result(result)
    }

    // pub fn set_param(&self, id: u32, param: &PodRef) -> crate::Result<()> {
    //     let result =
    //         unsafe { pw_sys::pw_stream_set_param(self.as_raw_ptr(), id, param.as_raw_ptr()) };
    //     i32_as_void_result(result)
    // } Since 0.3.70

    pub fn get_control(&self, id: u32) -> Option<&ControlRef> {
        unsafe {
            pw_sys::pw_stream_get_control(self.as_raw_ptr(), id)
                .as_ref()
                .map(|ptr| ControlRef::from_raw_ptr(ptr))
        }
    }

    pub fn set_control(&self, id: u32, values: &mut [f32]) -> crate::Result<()> {
        let result = unsafe {
            pw_sys::pw_stream_set_control(
                self.as_raw_ptr(),
                id,
                values.len() as u32,
                values.as_mut_ptr(),
            )
        };
        i32_as_void_result(result)
    }

    // todo: use get_time_n in new version
    pub fn get_time(&self) -> crate::Result<TimeRef> {
        let time = TimeRef::default();
        let result = unsafe { pw_sys::pw_stream_get_time(self.as_raw_ptr(), time.as_raw_ptr()) };
        i32_as_result(result, time)
    }

    pub fn dequeue_buffer(&self) -> crate::Result<&BufferRef> {
        unsafe { raw_wrapper(pw_sys::pw_stream_dequeue_buffer(self.as_raw_ptr())) }
    }

    pub fn queue_buffer(&self, buffer: &BufferRef) -> crate::Result<()> {
        let result =
            unsafe { pw_sys::pw_stream_queue_buffer(self.as_raw_ptr(), buffer.as_raw_ptr()) };
        i32_as_void_result(result)
    }

    pub fn set_active(&self, active: bool) -> crate::Result<()> {
        let result = unsafe { pw_sys::pw_stream_set_active(self.as_raw_ptr(), active) };
        i32_as_void_result(result)
    }

    pub fn flush(&self, drain: bool) -> crate::Result<()> {
        let result = unsafe { pw_sys::pw_stream_flush(self.as_raw_ptr(), drain) };
        i32_as_void_result(result)
    }

    pub fn is_driving(&self) -> bool {
        unsafe { pw_sys::pw_stream_is_driving(self.as_raw_ptr()) }
    }

    pub fn trigger_process(&self) -> crate::Result<()> {
        let result = unsafe { pw_sys::pw_stream_trigger_process(self.as_raw_ptr()) };
        i32_as_void_result(result)
    }
}

impl<'a> AddListener<'a> for StreamRef {
    type Events = StreamEvents<'a>;

    fn add_listener(&self, events: Pin<Box<Self::Events>>) -> Pin<Box<Self::Events>> {
        unsafe {
            pw_sys::pw_stream_add_listener(
                self.as_raw_ptr(),
                events.hook().as_raw_ptr(),
                events.as_raw_ptr(),
                &*events as *const _ as *mut _,
            )
        };

        events
    }
}

#[derive(Wrapper, Debug, Clone)]
pub struct Stream<'a> {
    #[raw_wrapper]
    ref_: NonNull<StreamRef>,

    listeners: Listeners<Pin<Box<StreamEvents<'a>>>>,
}

impl<'a> Stream<'a> {
    pub fn new(core: &CoreRef, name: &CStr, props: &PropertiesRef) -> crate::Result<Self> {
        unsafe {
            let result =
                pw_sys::pw_stream_new(core.as_raw_ptr(), name.as_ptr(), props.as_raw_ptr());
            let ref_ = new_instance_raw_wrapper(result)?;
            Ok(Self {
                ref_,
                listeners: Default::default(),
            })
        }
    }

    // todo new_simple
}

impl Drop for Stream<'_> {
    fn drop(&mut self) {
        unsafe { pw_sys::pw_stream_destroy(self.as_raw_ptr()) }
    }
}
