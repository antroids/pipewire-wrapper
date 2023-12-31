/*
 * SPDX-License-Identifier: MIT
 */
use std::ffi::CStr;
use std::fmt::{Debug, Formatter};
use std::pin::Pin;
use std::ptr::NonNull;

use derive_builder::Builder;
use pw_sys::pw_proxy_events;

use pipewire_wrapper_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::proxy::ProxyRef;
use crate::events_builder_build;
use crate::spa::interface::Hook;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct ProxyEventsRef {
    #[raw]
    raw: pw_sys::pw_proxy_events,
}

pub type DestroyCallback = Box<dyn FnMut()>;
pub type BoundCallback = Box<dyn FnMut(u32)>;
pub type RemovedCallback = Box<dyn FnMut()>;
pub type DoneCallback = Box<dyn FnMut(i32)>;
pub type ErrorCallback = Box<dyn for<'a> FnMut(i32, i32, &'a CStr)>;

#[derive(Wrapper, Builder)]
#[builder(setter(skip, strip_option), build_fn(skip), pattern = "owned")]
pub struct ProxyEvents {
    #[raw_wrapper]
    ref_: NonNull<ProxyEventsRef>,

    raw: Pin<Box<ProxyEventsRef>>,
    hook: Pin<Box<Hook>>,

    #[builder(setter)]
    destroy: Option<DestroyCallback>,
    #[builder(setter)]
    bound: Option<BoundCallback>,
    #[builder(setter)]
    removed: Option<RemovedCallback>,
    #[builder(setter)]
    done: Option<DoneCallback>,
    #[builder(setter)]
    error: Option<ErrorCallback>,
}

impl ProxyEvents {
    unsafe extern "C" fn destroy_call(data: *mut ::std::os::raw::c_void) {
        if let Some(events) = (data as *mut ProxyEvents).as_mut() {
            if let Some(callback) = &mut events.destroy {
                callback();
            }
        }
    }

    unsafe extern "C" fn bound_call(data: *mut ::std::os::raw::c_void, global_id: u32) {
        if let Some(events) = (data as *mut ProxyEvents).as_mut() {
            if let Some(callback) = &mut events.bound {
                callback(global_id);
            }
        }
    }

    unsafe extern "C" fn removed_call(data: *mut ::std::os::raw::c_void) {
        if let Some(events) = (data as *mut ProxyEvents).as_mut() {
            if let Some(callback) = &mut events.removed {
                callback();
            }
        }
    }

    unsafe extern "C" fn done_call(data: *mut ::std::os::raw::c_void, seq: i32) {
        if let Some(events) = (data as *mut ProxyEvents).as_mut() {
            if let Some(callback) = &mut events.done {
                callback(seq);
            }
        }
    }

    unsafe extern "C" fn error_call(
        data: *mut ::std::os::raw::c_void,
        seq: i32,
        res: i32,
        message: *const ::std::os::raw::c_char,
    ) {
        if let Some(events) = (data as *mut ProxyEvents).as_mut() {
            if let Some(callback) = &mut events.error {
                callback(seq, res, CStr::from_ptr(message));
            }
        }
    }

    pub fn hook(&self) -> &Pin<Box<Hook>> {
        &self.hook
    }

    pub fn version(&self) -> u32 {
        self.raw.raw.version
    }
}

// todo: channel builder

impl ProxyEventsBuilder {
    events_builder_build! {
        ProxyEvents,
        pw_proxy_events,
        destroy => destroy_call,
        bound => bound_call,
        removed => removed_call,
        done => done_call,
        error => error_call,
    }
}

impl Debug for ProxyEvents {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProxyEvents")
            .field("raw", &self.raw)
            .finish()
    }
}
