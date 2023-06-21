use crate::core_api::proxy::ProxyRef;
use crate::spa::interface::Hook;
use crate::wrapper::RawWrapper;
use pipewire_proc_macro::{RawWrapper, Wrapper};
use std::ffi::CStr;
use std::pin::Pin;
use std::ptr::NonNull;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct ProxyEventsRef {
    #[raw]
    raw: pw_sys::pw_proxy_events,
}

#[derive(Wrapper)]
pub struct ProxyEvents<'p> {
    #[raw_wrapper]
    ref_: NonNull<ProxyEventsRef>,

    proxy: &'p ProxyRef,

    raw: Pin<Box<ProxyEventsRef>>,
    hook: Pin<Box<Hook>>,

    destroy: Option<Box<dyn FnMut() + 'p>>,
    bound: Option<Box<dyn FnMut(u32) + 'p>>,
    removed: Option<Box<dyn FnMut() + 'p>>,
    done: Option<Box<dyn FnMut(i32) + 'p>>,
    error: Option<Box<dyn FnMut(i32, i32, &CStr) + 'p>>,
}

impl Drop for ProxyEvents<'_> {
    fn drop(&mut self) {
        // handled by hook
    }
}

impl<'p> ProxyEvents<'p> {
    pub(crate) fn new(proxy: &'p ProxyRef) -> Pin<Box<Self>> {
        let hook = Hook::new();
        let raw = ProxyEventsRef::from_raw(pw_sys::pw_proxy_events {
            version: 0,
            destroy: None,
            bound: None,
            removed: None,
            done: None,
            error: None,
        });
        let mut pinned_raw = Box::into_pin(Box::new(raw));

        Box::into_pin(Box::new(Self {
            ref_: NonNull::new(pinned_raw.as_ptr()).unwrap(),
            proxy,
            raw: pinned_raw,
            hook,
            destroy: None,
            bound: None,
            removed: None,
            done: None,
            error: None,
        }))
    }

    fn destroy_call() -> unsafe extern "C" fn(data: *mut ::std::os::raw::c_void) {
        unsafe extern "C" fn call(data: *mut ::std::os::raw::c_void) {
            if let Some(events) = (data as *mut ProxyEvents).as_mut() {
                if let Some(callback) = &mut events.destroy {
                    callback();
                }
            }
        }
        call
    }

    fn bound_call() -> unsafe extern "C" fn(data: *mut ::std::os::raw::c_void, global_id: u32) {
        unsafe extern "C" fn call(data: *mut ::std::os::raw::c_void, global_id: u32) {
            if let Some(events) = (data as *mut ProxyEvents).as_mut() {
                if let Some(callback) = &mut events.bound {
                    callback(global_id);
                }
            }
        }
        call
    }

    fn removed_call() -> unsafe extern "C" fn(data: *mut ::std::os::raw::c_void) {
        unsafe extern "C" fn call(data: *mut ::std::os::raw::c_void) {
            if let Some(events) = (data as *mut ProxyEvents).as_mut() {
                if let Some(callback) = &mut events.removed {
                    callback();
                }
            }
        }
        call
    }

    fn done_call() -> unsafe extern "C" fn(data: *mut ::std::os::raw::c_void, seq: i32) {
        unsafe extern "C" fn call(data: *mut ::std::os::raw::c_void, seq: i32) {
            if let Some(events) = (data as *mut ProxyEvents).as_mut() {
                if let Some(callback) = &mut events.done {
                    callback(seq);
                }
            }
        }
        call
    }

    fn error_call() -> unsafe extern "C" fn(
        data: *mut ::std::os::raw::c_void,
        seq: i32,
        res: i32,
        message: *const ::std::os::raw::c_char,
    ) {
        unsafe extern "C" fn call(
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
        call
    }

    pub fn set_destroy(&mut self, destroy: Option<Box<dyn FnMut() + 'p>>) {
        self.destroy = destroy;
        self.raw.raw.destroy = self.destroy.as_ref().map(|_| Self::destroy_call());
    }

    pub fn set_bound(&mut self, bound: Option<Box<dyn FnMut(u32) + 'p>>) {
        self.bound = bound;
        self.raw.raw.bound = self.bound.as_ref().map(|_| Self::bound_call());
    }

    pub fn set_removed(&mut self, removed: Option<Box<dyn FnMut() + 'p>>) {
        self.removed = removed;
        self.raw.raw.removed = self.removed.as_ref().map(|_| Self::removed_call());
    }

    pub fn set_done(&mut self, done: Option<Box<dyn FnMut(i32) + 'p>>) {
        self.done = done;
        self.raw.raw.done = self.done.as_ref().map(|_| Self::done_call());
    }

    pub fn set_error(&mut self, error: Option<Box<dyn FnMut(i32, i32, &CStr) + 'p>>) {
        self.error = error;
        self.raw.raw.error = self.error.as_ref().map(|_| Self::error_call());
    }

    pub fn proxy(&self) -> &'p ProxyRef {
        self.proxy
    }
    pub fn hook(&self) -> &Pin<Box<Hook>> {
        &self.hook
    }
}
