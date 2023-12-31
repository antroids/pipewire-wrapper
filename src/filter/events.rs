/*
 * SPDX-License-Identifier: MIT
 */
use std::ffi::CStr;
use std::fmt::{Debug, Formatter};
use std::pin::Pin;
use std::ptr::NonNull;

use derive_builder::Builder;
use pw_sys::{pw_buffer, pw_filter_events, pw_filter_state};
use spa_sys::{spa_command, spa_pod};

use pipewire_wrapper_proc_macro::{RawWrapper, Wrapper};

use crate::events_builder_build;
use crate::filter::FilterState;
use crate::spa::interface::Hook;
use crate::spa::io::IOPositionRef;
use crate::spa::pod::PodRef;
use crate::spa::type_::CommandRef;
use crate::stream::buffer::BufferRef;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct FilterEventsRef {
    #[raw]
    raw: pw_sys::pw_filter_events,
}

pub type DestroyCallback = Box<dyn FnMut()>;
pub type StateChangedCallback = Box<dyn for<'a> FnMut(FilterState, FilterState, Option<&'a CStr>)>;
pub type IOChangedCallback<T> =
    Box<dyn for<'a> FnMut(Option<&mut T>, u32, *mut ::std::os::raw::c_void, u32)>;
pub type ParamChangedCallback<T> = Box<dyn for<'a> FnMut(Option<&mut T>, u32, &'a PodRef)>;
pub type AddBufferCallback<T> = Box<dyn for<'a> FnMut(&mut T, &'a BufferRef)>;
pub type RemoveBufferCallback<T> = Box<dyn for<'a> FnMut(&mut T, &'a BufferRef)>;
pub type ProcessCallback = Box<dyn for<'a> FnMut(&'a IOPositionRef)>;
pub type DrainedCallback = Box<dyn FnMut()>;
pub type CommandCallback = Box<dyn for<'a> FnMut(&'a CommandRef)>;

#[derive(Wrapper, Builder)]
#[builder(setter(skip, strip_option), build_fn(skip), pattern = "owned")]
pub struct FilterEvents<T> {
    #[raw_wrapper]
    ref_: NonNull<FilterEventsRef>,

    raw: Pin<Box<FilterEventsRef>>,
    hook: Pin<Box<Hook>>,

    #[builder(setter)]
    destroy: Option<DestroyCallback>,
    #[builder(setter)]
    state_changed: Option<StateChangedCallback>,
    #[builder(setter)]
    io_changed: Option<IOChangedCallback<T>>,
    #[builder(setter)]
    param_changed: Option<ParamChangedCallback<T>>,
    #[builder(setter)]
    add_buffer: Option<AddBufferCallback<T>>,
    #[builder(setter)]
    remove_buffer: Option<RemoveBufferCallback<T>>,
    #[builder(setter)]
    process: Option<ProcessCallback>,
    #[builder(setter)]
    drained: Option<DrainedCallback>,
    #[builder(setter)]
    command: Option<CommandCallback>,
}

impl<T> FilterEvents<T> {
    unsafe extern "C" fn destroy_call(data: *mut ::std::os::raw::c_void) {
        if let Some(events) = (data as *mut FilterEvents<T>).as_mut() {
            if let Some(callback) = &mut events.destroy {
                callback();
            }
        }
    }

    unsafe extern "C" fn state_changed_call(
        data: *mut ::std::os::raw::c_void,
        old: pw_filter_state,
        state: pw_filter_state,
        error: *const ::std::os::raw::c_char,
    ) {
        if let Some(events) = (data as *mut FilterEvents<T>).as_mut() {
            if let Some(callback) = &mut events.state_changed {
                callback(
                    FilterState::from_raw(old),
                    FilterState::from_raw(state),
                    error.as_ref().map(|e| CStr::from_ptr(e)),
                );
            }
        }
    }

    unsafe extern "C" fn io_changed_call(
        port_data: *mut ::std::os::raw::c_void,
        data: *mut ::std::os::raw::c_void,
        id: u32,
        area: *mut ::std::os::raw::c_void,
        size: u32,
    ) {
        if let Some(events) = (data as *mut FilterEvents<T>).as_mut() {
            if let Some(callback) = &mut events.io_changed {
                callback((port_data as *mut T).as_mut(), id, area, size);
            }
        }
    }

    unsafe extern "C" fn param_changed_call(
        port_data: *mut ::std::os::raw::c_void,
        data: *mut ::std::os::raw::c_void,
        id: u32,
        param: *const spa_pod,
    ) {
        if let Some(events) = (data as *mut FilterEvents<T>).as_mut() {
            if let Some(callback) = &mut events.param_changed {
                callback(
                    (port_data as *mut T).as_mut(),
                    id,
                    PodRef::from_raw_ptr(param),
                );
            }
        }
    }

    unsafe extern "C" fn add_buffer_call(
        port_data: *mut ::std::os::raw::c_void,
        data: *mut ::std::os::raw::c_void,
        buffer: *mut pw_buffer,
    ) {
        if let Some(events) = (data as *mut FilterEvents<T>).as_mut() {
            if let Some(callback) = &mut events.add_buffer {
                callback(
                    (port_data as *mut T).as_mut().unwrap(),
                    BufferRef::from_raw_ptr(buffer),
                );
            }
        }
    }

    unsafe extern "C" fn remove_buffer_call(
        port_data: *mut ::std::os::raw::c_void,
        data: *mut ::std::os::raw::c_void,
        buffer: *mut pw_buffer,
    ) {
        if let Some(events) = (data as *mut FilterEvents<T>).as_mut() {
            if let Some(callback) = &mut events.remove_buffer {
                callback(
                    (port_data as *mut T).as_mut().unwrap(),
                    BufferRef::from_raw_ptr(buffer),
                );
            }
        }
    }

    unsafe extern "C" fn process_call(
        data: *mut ::std::os::raw::c_void,
        position: *mut spa_sys::spa_io_position,
    ) {
        if let Some(events) = (data as *mut FilterEvents<T>).as_mut() {
            if let Some(callback) = &mut events.process {
                callback(IOPositionRef::from_raw_ptr(position));
            }
        }
    }

    unsafe extern "C" fn drained_call(data: *mut ::std::os::raw::c_void) {
        if let Some(events) = (data as *mut FilterEvents<T>).as_mut() {
            if let Some(callback) = &mut events.drained {
                callback();
            }
        }
    }

    unsafe extern "C" fn command_call(
        data: *mut ::std::os::raw::c_void,
        command: *const spa_command,
    ) {
        if let Some(events) = (data as *mut FilterEvents<T>).as_mut() {
            if let Some(callback) = &mut events.command {
                callback(CommandRef::from_raw_ptr(command));
            }
        }
    }

    pub fn hook(&self) -> &Pin<Box<Hook>> {
        &self.hook
    }
}

impl<T> FilterEventsBuilder<T> {
    events_builder_build! {
        FilterEvents<T>,
        pw_filter_events,
        destroy => destroy_call,
        state_changed => state_changed_call,
        io_changed => io_changed_call,
        param_changed => param_changed_call,
        add_buffer => add_buffer_call,
        remove_buffer => remove_buffer_call,
        process => process_call,
        drained => drained_call,
        command => command_call,
    }
}

impl<T> Debug for FilterEvents<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FilterEvents")
            .field("raw", &self.raw)
            .finish()
    }
}
