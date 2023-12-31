/*
 * SPDX-License-Identifier: MIT
 */
use std::ffi::CStr;
use std::fmt::{Debug, Formatter};
use std::pin::Pin;
use std::ptr::NonNull;

use derive_builder::Builder;
use pw_sys::{pw_buffer, pw_stream_control, pw_stream_events, pw_stream_state};
use spa_sys::{spa_command, spa_pod};

use pipewire_wrapper_proc_macro::{RawWrapper, Wrapper};

use crate::events_builder_build;
use crate::spa::interface::Hook;
use crate::spa::io::IOValue;
use crate::spa::pod::object::param_io::IOType;
use crate::spa::pod::PodRef;
use crate::spa::type_::CommandRef;
use crate::stream::buffer::BufferRef;
use crate::stream::control::ControlRef;
use crate::stream::State;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct StreamEventsRef {
    #[raw]
    raw: pw_sys::pw_stream_events,
}

pub type DestroyCallback = Box<dyn FnMut()>;
pub type StateChangedCallback = Box<dyn for<'a> FnMut(State, State, Option<&'a CStr>)>;
pub type ControlInfoCallback = Box<dyn for<'a> FnMut(u32, &'a ControlRef)>;
pub type IOChangedCallback = Box<dyn for<'a> FnMut(IOValue)>;
pub type ParamChangedCallback = Box<dyn for<'a> FnMut(u32, &'a PodRef)>;
pub type AddBufferCallback = Box<dyn for<'a> FnMut(&'a BufferRef)>;
pub type RemoveBufferCallback = Box<dyn for<'a> FnMut(&'a BufferRef)>;
pub type ProcessCallback = Box<dyn FnMut()>;
pub type DrainedCallback = Box<dyn FnMut()>;
pub type CommandCallback = Box<dyn for<'a> FnMut(&'a CommandRef)>;
pub type TriggerDoneCallback = Box<dyn FnMut()>;

#[derive(Wrapper, Builder)]
#[builder(setter(skip, strip_option), build_fn(skip), pattern = "owned")]
pub struct StreamEvents {
    #[raw_wrapper]
    ref_: NonNull<StreamEventsRef>,

    raw: Pin<Box<StreamEventsRef>>,
    hook: Pin<Box<Hook>>,

    #[builder(setter)]
    destroy: Option<DestroyCallback>,
    #[builder(setter)]
    state_changed: Option<StateChangedCallback>,
    #[builder(setter)]
    control_info: Option<ControlInfoCallback>,
    #[builder(setter)]
    io_changed: Option<IOChangedCallback>,
    #[builder(setter)]
    param_changed: Option<ParamChangedCallback>,
    #[builder(setter)]
    add_buffer: Option<AddBufferCallback>,
    #[builder(setter)]
    remove_buffer: Option<RemoveBufferCallback>,
    #[builder(setter)]
    process: Option<ProcessCallback>,
    #[builder(setter)]
    drained: Option<DrainedCallback>,
    #[builder(setter)]
    command: Option<CommandCallback>,
    #[builder(setter)]
    trigger_done: Option<TriggerDoneCallback>,
}

impl StreamEvents {
    unsafe extern "C" fn destroy_call(data: *mut ::std::os::raw::c_void) {
        if let Some(events) = (data as *mut StreamEvents).as_mut() {
            if let Some(callback) = &mut events.destroy {
                callback();
            }
        }
    }

    unsafe extern "C" fn state_changed_call(
        data: *mut ::std::os::raw::c_void,
        old: pw_stream_state,
        state: pw_stream_state,
        error: *const ::std::os::raw::c_char,
    ) {
        if let Some(events) = (data as *mut StreamEvents).as_mut() {
            if let Some(callback) = &mut events.state_changed {
                callback(
                    State::from_raw(old),
                    State::from_raw(state),
                    error.as_ref().map(|e| CStr::from_ptr(e)),
                );
            }
        }
    }

    unsafe extern "C" fn control_info_call(
        data: *mut ::std::os::raw::c_void,
        id: u32,
        control: *const pw_stream_control,
    ) {
        if !control.is_null() {
            if let Some(events) = (data as *mut StreamEvents).as_mut() {
                if let Some(callback) = &mut events.control_info {
                    callback(id, ControlRef::from_raw_ptr(control));
                }
            }
        }
    }

    unsafe extern "C" fn io_changed_call(
        data: *mut ::std::os::raw::c_void,
        id: u32,
        area: *mut ::std::os::raw::c_void,
        size: u32,
    ) {
        if !area.is_null() {
            if let Some(events) = (data as *mut StreamEvents).as_mut() {
                if let Some(callback) = &mut events.io_changed {
                    callback(IOValue::from_type_and_ptr(IOType::from(id), area));
                }
            }
        }
    }

    unsafe extern "C" fn param_changed_call(
        data: *mut ::std::os::raw::c_void,
        id: u32,
        param: *const spa_pod,
    ) {
        if !param.is_null() {
            if let Some(events) = (data as *mut StreamEvents).as_mut() {
                if let Some(callback) = &mut events.param_changed {
                    callback(id, PodRef::from_raw_ptr(param));
                }
            }
        }
    }

    unsafe extern "C" fn add_buffer_call(
        data: *mut ::std::os::raw::c_void,
        buffer: *mut pw_buffer,
    ) {
        if !buffer.is_null() {
            if let Some(events) = (data as *mut StreamEvents).as_mut() {
                if let Some(callback) = &mut events.add_buffer {
                    callback(BufferRef::from_raw_ptr(buffer));
                }
            }
        }
    }

    unsafe extern "C" fn remove_buffer_call(
        data: *mut ::std::os::raw::c_void,
        buffer: *mut pw_buffer,
    ) {
        if !buffer.is_null() {
            if let Some(events) = (data as *mut StreamEvents).as_mut() {
                if let Some(callback) = &mut events.remove_buffer {
                    callback(BufferRef::from_raw_ptr(buffer));
                }
            }
        }
    }

    unsafe extern "C" fn process_call(data: *mut ::std::os::raw::c_void) {
        if let Some(events) = (data as *mut StreamEvents).as_mut() {
            if let Some(callback) = &mut events.process {
                callback();
            }
        }
    }

    unsafe extern "C" fn drained_call(data: *mut ::std::os::raw::c_void) {
        if let Some(events) = (data as *mut StreamEvents).as_mut() {
            if let Some(callback) = &mut events.drained {
                callback();
            }
        }
    }

    unsafe extern "C" fn command_call(
        data: *mut ::std::os::raw::c_void,
        command: *const spa_command,
    ) {
        if !command.is_null() {
            if let Some(events) = (data as *mut StreamEvents).as_mut() {
                if let Some(callback) = &mut events.command {
                    callback(CommandRef::from_raw_ptr(command));
                }
            }
        }
    }

    unsafe extern "C" fn trigger_done_call(data: *mut ::std::os::raw::c_void) {
        if let Some(events) = (data as *mut StreamEvents).as_mut() {
            if let Some(callback) = &mut events.trigger_done {
                callback();
            }
        }
    }

    pub fn hook(&self) -> &Pin<Box<Hook>> {
        &self.hook
    }
}

impl StreamEventsBuilder {
    events_builder_build! {
        StreamEvents,
        pw_stream_events,
        destroy => destroy_call,
        state_changed => state_changed_call,
        control_info => control_info_call,
        io_changed => io_changed_call,
        param_changed => param_changed_call,
        add_buffer => add_buffer_call,
        remove_buffer => remove_buffer_call,
        process => process_call,
        drained => drained_call,
        command => command_call,
        trigger_done => trigger_done_call,
    }
}

impl Debug for StreamEvents {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamEvents")
            .field("raw", &self.raw)
            .finish()
    }
}
