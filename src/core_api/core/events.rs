/*
 * SPDX-License-Identifier: MIT
 */
use std::ffi::CStr;
use std::fmt::{Debug, Formatter};
use std::os::fd::RawFd;
use std::pin::Pin;
use std::ptr::NonNull;

use derive_builder::Builder;
use pw_sys::{pw_core_events, pw_core_info};

use pipewire_wrapper_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::core::info::CoreInfoRef;
use crate::core_api::core::CoreRef;
use crate::events_builder_build;
use crate::spa::interface::{Hook, HookRef};
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct CoreEventsRef {
    #[raw]
    raw: pw_sys::pw_core_events,
}

pub type InfoCallback = Box<dyn for<'a> FnMut(&'a CoreInfoRef)>;
pub type DoneCallback = Box<dyn FnMut(u32, i32)>;
pub type PingCallback = Box<dyn FnMut(u32, i32)>;
pub type ErrorCallback = Box<dyn FnMut(u32, i32, i32, &CStr)>;
pub type RemoveIdCallback = Box<dyn FnMut(u32)>;
pub type BoundIdCallback = Box<dyn FnMut(u32, u32)>;
pub type AddMemCallback = Box<dyn FnMut(u32, u32, RawFd, u32)>;
pub type RemoveMemCallback = Box<dyn FnMut(u32)>;

#[derive(Wrapper, Builder)]
#[builder(setter(skip, strip_option), build_fn(skip), pattern = "owned")]
pub struct CoreEvents {
    #[raw_wrapper]
    ref_: NonNull<CoreEventsRef>,

    raw: Pin<Box<CoreEventsRef>>,
    hook: Pin<Box<Hook>>,

    #[builder(setter)]
    info: Option<InfoCallback>,
    #[builder(setter)]
    done: Option<DoneCallback>,
    #[builder(setter)]
    ping: Option<PingCallback>,
    #[builder(setter)]
    error: Option<ErrorCallback>,
    #[builder(setter)]
    remove_id: Option<RemoveIdCallback>,
    #[builder(setter)]
    bound_id: Option<BoundIdCallback>,
    #[builder(setter)]
    add_mem: Option<AddMemCallback>,
    #[builder(setter)]
    remove_mem: Option<RemoveMemCallback>,
}

impl CoreEvents {
    unsafe extern "C" fn info_call(data: *mut ::std::os::raw::c_void, info: *const pw_core_info) {
        if let Some(core_events) = (data as *mut CoreEvents).as_mut() {
            if let Some(callback) = &mut core_events.info {
                callback(CoreInfoRef::from_raw_ptr(info));
            }
        }
    }

    unsafe extern "C" fn done_call(
        data: *mut ::std::os::raw::c_void,
        id: u32,
        seq: ::std::os::raw::c_int,
    ) {
        if let Some(core_events) = (data as *mut CoreEvents).as_mut() {
            if let Some(callback) = &mut core_events.done {
                callback(id, seq);
            }
        }
    }

    unsafe extern "C" fn ping_call(
        data: *mut ::std::os::raw::c_void,
        id: u32,
        seq: ::std::os::raw::c_int,
    ) {
        if let Some(core_events) = (data as *mut CoreEvents).as_mut() {
            if let Some(callback) = &mut core_events.ping {
                callback(id, seq);
            }
        }
    }

    unsafe extern "C" fn error_call(
        data: *mut ::std::os::raw::c_void,
        id: u32,
        seq: i32,
        res: i32,
        message: *const ::std::os::raw::c_char,
    ) {
        if let Some(events) = (data as *mut CoreEvents).as_mut() {
            if let Some(callback) = &mut events.error {
                callback(id, seq, res, CStr::from_ptr(message));
            }
        }
    }

    unsafe extern "C" fn remove_id_call(data: *mut ::std::os::raw::c_void, id: u32) {
        if let Some(core_events) = (data as *mut CoreEvents).as_mut() {
            if let Some(callback) = &mut core_events.remove_id {
                callback(id);
            }
        }
    }

    unsafe extern "C" fn bound_id_call(data: *mut ::std::os::raw::c_void, id: u32, global_id: u32) {
        if let Some(core_events) = (data as *mut CoreEvents).as_mut() {
            if let Some(callback) = &mut core_events.bound_id {
                callback(id, global_id);
            }
        }
    }

    unsafe extern "C" fn add_mem_call(
        data: *mut ::std::os::raw::c_void,
        id: u32,
        type_: u32,
        fd: ::std::os::raw::c_int,
        flags: u32,
    ) {
        if let Some(core_events) = (data as *mut CoreEvents).as_mut() {
            if let Some(callback) = &mut core_events.add_mem {
                callback(id, type_, fd, flags);
            }
        }
    }

    unsafe extern "C" fn remove_mem_call(data: *mut ::std::os::raw::c_void, id: u32) {
        if let Some(core_events) = (data as *mut CoreEvents).as_mut() {
            if let Some(callback) = &mut core_events.remove_mem {
                callback(id);
            }
        }
    }

    pub fn version(&self) -> u32 {
        self.raw.raw.version
    }

    pub fn hook(&self) -> &Pin<Box<Hook>> {
        &self.hook
    }
}

// todo: channel builder

impl CoreEventsBuilder {
    events_builder_build! {
        CoreEvents,
        pw_core_events,
        info => info_call,
        done => done_call,
        ping => ping_call,
        error => error_call,
        remove_id => remove_id_call,
        bound_id => bound_id_call,
        add_mem => add_mem_call,
        remove_mem => remove_mem_call,
    }
}

impl Debug for CoreEvents {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoreEvents")
            .field("raw", &self.raw)
            .finish()
    }
}
