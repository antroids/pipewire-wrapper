use crate::core_api::port::info::PortInfoRef;
use crate::core_api::port::PortRef;
use crate::spa::interface::Hook;
use crate::spa::param::{ParamInfoRef, ParamType};
use crate::spa::pod::PodRef;
use crate::wrapper::RawWrapper;
use pipewire_proc_macro::{RawWrapper, Wrapper};
use pw_sys::pw_port_info;
use spa_sys::spa_pod;
use std::ffi::CStr;
use std::pin::Pin;
use std::ptr::NonNull;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct PortEventsRef {
    #[raw]
    raw: pw_sys::pw_port_events,
}

#[derive(Wrapper)]
pub struct PortEvents<'p> {
    #[raw_wrapper]
    ref_: NonNull<PortEventsRef>,

    port: &'p PortRef,

    raw: Pin<Box<PortEventsRef>>,
    hook: Pin<Box<Hook>>,

    info: Option<Box<dyn FnMut(&'p PortInfoRef) + 'p>>,
    param: Option<Box<dyn FnMut(i32, ParamType, u32, u32, &'p PodRef) + 'p>>,
}

impl Drop for PortEvents<'_> {
    fn drop(&mut self) {
        // handled by hook
    }
}

impl<'p> PortEvents<'p> {
    pub(crate) fn new(port: &'p PortRef) -> Pin<Box<Self>> {
        let hook = Hook::new();
        let raw = PortEventsRef::from_raw(pw_sys::pw_port_events {
            version: 0,
            info: None,
            param: None,
        });
        let mut pinned_raw = Box::into_pin(Box::new(raw));

        Box::into_pin(Box::new(Self {
            ref_: NonNull::new(pinned_raw.as_ptr()).unwrap(),
            port,
            raw: pinned_raw,
            hook,
            info: None,
            param: None,
        }))
    }

    fn info_call(
    ) -> unsafe extern "C" fn(data: *mut ::std::os::raw::c_void, info: *const pw_port_info) {
        unsafe extern "C" fn call(data: *mut ::std::os::raw::c_void, info: *const pw_port_info) {
            if let Some(events) = (data as *mut PortEvents).as_mut() {
                if let Some(callback) = &mut events.info {
                    callback(PortInfoRef::from_raw_ptr(info));
                }
            }
        }
        call
    }

    fn param_call() -> unsafe extern "C" fn(
        data: *mut ::std::os::raw::c_void,
        seq: ::std::os::raw::c_int,
        id: u32,
        index: u32,
        next: u32,
        param: *const spa_pod,
    ) {
        unsafe extern "C" fn call(
            data: *mut ::std::os::raw::c_void,
            seq: ::std::os::raw::c_int,
            id: u32,
            index: u32,
            next: u32,
            param: *const spa_pod,
        ) {
            if let Some(events) = (data as *mut PortEvents).as_mut() {
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
        call
    }

    pub fn set_info(&mut self, info: Option<Box<dyn FnMut(&'p PortInfoRef) + 'p>>) {
        self.info = info;
        self.raw.raw.info = self.info.as_ref().map(|_| Self::info_call());
    }

    pub fn set_param(
        &mut self,
        param: Option<Box<dyn FnMut(i32, ParamType, u32, u32, &'p PodRef) + 'p>>,
    ) {
        self.param = param;
        self.raw.raw.param = self.param.as_ref().map(|_| Self::param_call());
    }

    pub fn port(&self) -> &'p PortRef {
        self.port
    }
    pub fn hook(&self) -> &Pin<Box<Hook>> {
        &self.hook
    }
}
