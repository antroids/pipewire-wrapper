/*
 * SPDX-License-Identifier: MIT
 */
use std::ffi::CStr;
use std::fmt::{Debug, Formatter};
use std::pin::Pin;
use std::ptr::NonNull;

use derive_builder::Builder;
use pw_sys::{pw_device_events, pw_device_info};
use spa_sys::spa_pod;

use pipewire_wrapper_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::device::info::DeviceInfoRef;
use crate::core_api::device::DeviceRef;
use crate::events_builder_build;
use crate::spa::interface::Hook;
use crate::spa::param::{ParamInfoRef, ParamType};
use crate::spa::pod::PodRef;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct DeviceEventsRef {
    #[raw]
    raw: pw_sys::pw_device_events,
}

pub type InfoCallback = Box<dyn for<'a> FnMut(&'a DeviceInfoRef)>;
pub type ParamCallback = Box<dyn for<'a> FnMut(i32, ParamType, u32, u32, &'a PodRef)>;

#[derive(Wrapper, Builder)]
#[builder(setter(skip, strip_option), build_fn(skip), pattern = "owned")]
pub struct DeviceEvents {
    #[raw_wrapper]
    ref_: NonNull<DeviceEventsRef>,

    raw: Pin<Box<DeviceEventsRef>>,
    hook: Pin<Box<Hook>>,

    #[builder(setter)]
    info: Option<InfoCallback>,
    #[builder(setter)]
    param: Option<ParamCallback>,
}

impl DeviceEvents {
    unsafe extern "C" fn info_call(data: *mut ::std::os::raw::c_void, info: *const pw_device_info) {
        if let Some(events) = (data as *mut DeviceEvents).as_mut() {
            if let Some(callback) = &mut events.info {
                callback(DeviceInfoRef::from_raw_ptr(info));
            }
        }
    }

    unsafe extern "C" fn param_call(
        data: *mut ::std::os::raw::c_void,
        seq: ::std::os::raw::c_int,
        id: u32,
        index: u32,
        next: u32,
        param: *const spa_pod,
    ) {
        if let Some(events) = (data as *mut DeviceEvents).as_mut() {
            if let Some(callback) = &mut events.param {
                callback(
                    seq,
                    ParamType::from_raw(id),
                    index,
                    next,
                    PodRef::from_raw_ptr(param),
                );
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

impl DeviceEventsBuilder {
    events_builder_build! {
        DeviceEvents,
        pw_device_events,
        info => info_call,
        param => param_call,
    }
}

impl Debug for DeviceEvents {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeviceEvents")
            .field("raw", &self.raw)
            .finish()
    }
}
