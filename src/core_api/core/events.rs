use std::ffi::CStr;
use std::os::fd::RawFd;
use std::pin::Pin;
use std::ptr::NonNull;

use derive_builder::Builder;
use pw_sys::{pw_core_events, pw_core_info};

use pipewire_macro_impl::events_builder_build;
use pipewire_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::core::info::CoreInfoRef;
use crate::core_api::core::CoreRef;
use crate::spa::interface::{Hook, HookRef};
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct CoreEventsRef {
    #[raw]
    raw: pw_sys::pw_core_events,
}

#[derive(Wrapper, Builder)]
#[builder(setter(skip, strip_option), build_fn(skip), pattern = "owned")]
pub struct CoreEvents<'c> {
    #[raw_wrapper]
    ref_: NonNull<CoreEventsRef>,

    raw: Pin<Box<CoreEventsRef>>,
    hook: Pin<Box<Hook>>,

    #[builder(setter)]
    info: Option<Box<dyn FnMut(&'c CoreInfoRef) + 'c>>,
    #[builder(setter)]
    done: Option<Box<dyn FnMut(u32, i32) + 'c>>,
    #[builder(setter)]
    ping: Option<Box<dyn FnMut(u32, i32) + 'c>>,
    #[builder(setter)]
    error: Option<Box<dyn FnMut(u32, i32, i32, &CStr) + 'c>>,
    #[builder(setter)]
    remove_id: Option<Box<dyn FnMut(u32) + 'c>>,
    #[builder(setter)]
    bound_id: Option<Box<dyn FnMut(u32, u32) + 'c>>,
    #[builder(setter)]
    add_mem: Option<Box<dyn FnMut(u32, u32, RawFd, u32) + 'c>>,
    #[builder(setter)]
    remove_mem: Option<Box<dyn FnMut(u32) + 'c>>,
}

impl Drop for CoreEvents<'_> {
    fn drop(&mut self) {
        // handled by hook
    }
}

impl<'c> CoreEvents<'c> {
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

impl<'c> CoreEventsBuilder<'c> {
    events_builder_build! {
        CoreEvents<'c>,
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
