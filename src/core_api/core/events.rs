use crate::core_api::core::info::CoreInfoRef;
use crate::core_api::core::CoreRef;
use crate::spa::interface::{Hook, HookRef};
use crate::wrapper::RawWrapper;
use pipewire_proc_macro::{RawWrapper, Wrapper};
use pw_sys::pw_core_info;
use std::ffi::CStr;
use std::os::fd::RawFd;
use std::pin::Pin;
use std::ptr::NonNull;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct CoreEventsRef {
    #[raw]
    raw: pw_sys::pw_core_events,
}

#[derive(Wrapper)]
pub struct CoreEvents<'c> {
    #[raw_wrapper]
    ref_: NonNull<CoreEventsRef>,

    core: &'c CoreRef,

    raw: Pin<Box<CoreEventsRef>>,
    hook: Pin<Box<Hook>>,

    info: Option<Box<dyn FnMut(&'c CoreInfoRef) + 'c>>,
    done: Option<Box<dyn FnMut(u32, i32) + 'c>>,
    ping: Option<Box<dyn FnMut(u32, i32) + 'c>>,
    error: Option<Box<dyn FnMut(u32, i32, i32, &CStr) + 'c>>,
    remove_id: Option<Box<dyn FnMut(u32) + 'c>>,
    bound_id: Option<Box<dyn FnMut(u32, u32) + 'c>>,
    add_mem: Option<Box<dyn FnMut(u32, u32, RawFd, u32) + 'c>>,
    remove_mem: Option<Box<dyn FnMut(u32) + 'c>>,
}

impl Drop for CoreEvents<'_> {
    fn drop(&mut self) {
        // handled by hook
    }
}

impl<'c> CoreEvents<'c> {
    pub(crate) fn new(core: &'c CoreRef) -> Pin<Box<Self>> {
        let hook = Hook::new();
        let raw = CoreEventsRef::from_raw(pw_sys::pw_core_events {
            version: 0,
            info: None,
            done: None,
            ping: None,
            error: None,
            remove_id: None,
            bound_id: None,
            add_mem: None,
            remove_mem: None,
        });
        let mut pinned_raw = Box::into_pin(Box::new(raw));

        Box::into_pin(Box::new(Self {
            ref_: NonNull::new(pinned_raw.as_ptr()).unwrap(),
            core,
            raw: pinned_raw,
            hook,
            info: None,
            done: None,
            ping: None,
            error: None,
            remove_id: None,
            bound_id: None,
            add_mem: None,
            remove_mem: None,
        }))
    }

    fn info_call(
    ) -> unsafe extern "C" fn(data: *mut ::std::os::raw::c_void, info: *const pw_core_info) {
        unsafe extern "C" fn call(data: *mut ::std::os::raw::c_void, info: *const pw_core_info) {
            if let Some(core_events) = (data as *mut CoreEvents).as_mut() {
                if let Some(callback) = &mut core_events.info {
                    callback(CoreInfoRef::from_raw_ptr(info));
                }
            }
        }
        call
    }

    fn done_call(
    ) -> unsafe extern "C" fn(data: *mut ::std::os::raw::c_void, id: u32, seq: ::std::os::raw::c_int)
    {
        unsafe extern "C" fn call(
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
        call
    }

    fn ping_call(
    ) -> unsafe extern "C" fn(data: *mut ::std::os::raw::c_void, id: u32, seq: ::std::os::raw::c_int)
    {
        unsafe extern "C" fn call(
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
        call
    }

    fn error_call() -> unsafe extern "C" fn(
        data: *mut ::std::os::raw::c_void,
        id: u32,
        seq: i32,
        res: i32,
        message: *const ::std::os::raw::c_char,
    ) {
        unsafe extern "C" fn call(
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
        call
    }

    fn remove_id_call() -> unsafe extern "C" fn(data: *mut ::std::os::raw::c_void, id: u32) {
        unsafe extern "C" fn call(data: *mut ::std::os::raw::c_void, id: u32) {
            if let Some(core_events) = (data as *mut CoreEvents).as_mut() {
                if let Some(callback) = &mut core_events.remove_id {
                    callback(id);
                }
            }
        }
        call
    }

    fn bound_id_call(
    ) -> unsafe extern "C" fn(data: *mut ::std::os::raw::c_void, id: u32, global_id: u32) {
        unsafe extern "C" fn call(data: *mut ::std::os::raw::c_void, id: u32, global_id: u32) {
            if let Some(core_events) = (data as *mut CoreEvents).as_mut() {
                if let Some(callback) = &mut core_events.bound_id {
                    callback(id, global_id);
                }
            }
        }
        call
    }

    fn add_mem_call() -> unsafe extern "C" fn(
        data: *mut ::std::os::raw::c_void,
        id: u32,
        type_: u32,
        fd: ::std::os::raw::c_int,
        flags: u32,
    ) {
        unsafe extern "C" fn call(
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
        call
    }

    fn remove_mem_call() -> unsafe extern "C" fn(data: *mut ::std::os::raw::c_void, id: u32) {
        unsafe extern "C" fn call(data: *mut ::std::os::raw::c_void, id: u32) {
            if let Some(core_events) = (data as *mut CoreEvents).as_mut() {
                if let Some(callback) = &mut core_events.remove_mem {
                    callback(id);
                }
            }
        }
        call
    }

    pub fn set_info(&mut self, info: Option<Box<dyn FnMut(&'c CoreInfoRef) + 'c>>) {
        self.info = info;
        self.raw.raw.info = self.info.as_ref().map(|_| Self::info_call());
    }

    pub fn set_done(&mut self, done: Option<Box<dyn FnMut(u32, i32) + 'c>>) {
        self.done = done;
        self.raw.raw.done = self.done.as_ref().map(|_| Self::done_call());
    }

    pub fn set_ping(&mut self, ping: Option<Box<dyn FnMut(u32, i32) + 'c>>) {
        self.ping = ping;
        self.raw.raw.ping = self.ping.as_ref().map(|_| Self::ping_call());
    }

    pub fn set_error(&mut self, error: Option<Box<dyn FnMut(u32, i32, i32, &CStr) + 'c>>) {
        self.error = error;
        self.raw.raw.error = self.error.as_ref().map(|_| Self::error_call());
    }

    pub fn set_remove_id(&mut self, remove_id: Option<Box<dyn FnMut(u32) + 'c>>) {
        self.remove_id = remove_id;
        self.raw.raw.remove_id = self.remove_id.as_ref().map(|_| Self::remove_id_call());
    }

    pub fn set_bound_id(&mut self, bound_id: Option<Box<dyn FnMut(u32, u32) + 'c>>) {
        self.bound_id = bound_id;
        self.raw.raw.bound_id = self.bound_id.as_ref().map(|_| Self::bound_id_call());
    }

    pub fn set_add_mem(&mut self, add_mem: Option<Box<dyn FnMut(u32, u32, RawFd, u32) + 'c>>) {
        self.add_mem = add_mem;
        self.raw.raw.add_mem = self.add_mem.as_ref().map(|_| Self::add_mem_call());
    }

    pub fn set_remove_mem(&mut self, remove_mem: Option<Box<dyn FnMut(u32) + 'c>>) {
        self.remove_mem = remove_mem;
        self.raw.raw.remove_mem = self.remove_mem.as_ref().map(|_| Self::remove_mem_call());
    }

    pub fn core(&self) -> &'c CoreRef {
        self.core
    }

    pub fn version(&self) -> u32 {
        self.raw.raw.version
    }

    pub fn hook(&self) -> &Pin<Box<Hook>> {
        &self.hook
    }
}
