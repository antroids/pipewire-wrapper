use std::fmt::{Debug, Formatter};
use std::pin::Pin;
use std::ptr::NonNull;
use std::rc::Rc;

use pipewire_proc_macro::{RawWrapper, Wrapper};
use pw_sys::{pw_global, pw_impl_client};

use crate::core_api::context::{Context, ContextRef};
use crate::impl_api::global::GlobalRef;
use crate::impl_api::impl_client::ImplClientRef;
use crate::spa::interface::Hook;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct ContextEventsRef {
    #[raw]
    raw: pw_sys::pw_context_events,
}

#[derive(Wrapper)]
pub struct ContextEvents<'c> {
    #[raw_wrapper]
    ref_: NonNull<ContextEventsRef>,

    context: &'c ContextRef,

    raw: Pin<Box<ContextEventsRef>>,
    hook: Pin<Box<Hook>>,

    destroy: Option<Box<dyn FnMut() + 'c>>,
    free: Option<Box<dyn FnMut() + 'c>>,
    check_access: Option<Box<dyn FnMut(&'c ImplClientRef) + 'c>>,
    global_added: Option<Box<dyn FnMut(&'c GlobalRef) + 'c>>,
    global_removed: Option<Box<dyn FnMut(&'c GlobalRef) + 'c>>,
}

impl<'c> ContextEvents<'c> {
    pub(crate) fn new(context: &'c ContextRef) -> Pin<Box<Self>> {
        let hook = Hook::new();
        let raw = ContextEventsRef::from_raw(pw_sys::pw_context_events {
            version: 0,
            destroy: None,
            free: None,
            check_access: None,
            global_added: None,
            global_removed: None,
        });
        let mut pinned_raw = Box::into_pin(Box::new(raw));

        Box::into_pin(Box::new(Self {
            ref_: NonNull::new(pinned_raw.as_ptr()).unwrap(),
            context,
            raw: pinned_raw,
            hook,
            destroy: None,
            free: None,
            check_access: None,
            global_added: None,
            global_removed: None,
        }))
    }

    fn destroy_call() -> unsafe extern "C" fn(data: *mut ::std::os::raw::c_void) {
        unsafe extern "C" fn call(data: *mut ::std::os::raw::c_void) {
            if let Some(context_events) = (data as *mut ContextEvents).as_mut() {
                if let Some(callback) = &mut context_events.destroy {
                    callback();
                }
            }
        }
        call
    }

    fn free_call() -> unsafe extern "C" fn(data: *mut ::std::os::raw::c_void) {
        unsafe extern "C" fn call(data: *mut ::std::os::raw::c_void) {
            if let Some(context_events) = (data as *mut ContextEvents).as_mut() {
                if let Some(callback) = &mut context_events.free {
                    callback();
                }
            }
        }
        call
    }

    fn check_access_call(
    ) -> unsafe extern "C" fn(data: *mut ::std::os::raw::c_void, client: *mut pw_impl_client) {
        unsafe extern "C" fn call(data: *mut ::std::os::raw::c_void, client: *mut pw_impl_client) {
            if let Some(context_events) = (data as *mut ContextEvents).as_mut() {
                if let Some(callback) = &mut context_events.check_access {
                    callback(ImplClientRef::from_raw_ptr(client));
                }
            }
        }
        call
    }

    fn global_added_call(
    ) -> unsafe extern "C" fn(data: *mut ::std::os::raw::c_void, global: *mut pw_global) {
        unsafe extern "C" fn call(data: *mut ::std::os::raw::c_void, global: *mut pw_global) {
            if let Some(context_events) = (data as *mut ContextEvents).as_mut() {
                if let Some(callback) = &mut context_events.global_added {
                    callback(GlobalRef::from_raw_ptr(global));
                }
            }
        }
        call
    }

    fn global_removed_call(
    ) -> unsafe extern "C" fn(data: *mut ::std::os::raw::c_void, global: *mut pw_global) {
        unsafe extern "C" fn call(data: *mut ::std::os::raw::c_void, global: *mut pw_global) {
            if let Some(context_events) = (data as *mut ContextEvents).as_mut() {
                if let Some(callback) = &mut context_events.global_removed {
                    callback(GlobalRef::from_raw_ptr(global));
                }
            }
        }
        call
    }

    pub fn set_destroy(&mut self, destroy: Option<Box<dyn FnMut() + 'c>>) {
        self.destroy = destroy;
        self.raw.raw.destroy = self.destroy.as_ref().map(|_| Self::destroy_call());
    }
    pub fn set_free(&mut self, free: Option<Box<dyn FnMut() + 'c>>) {
        self.free = free;
        self.raw.raw.free = self.free.as_ref().map(|_| Self::free_call());
    }
    pub fn set_check_access(
        &mut self,
        check_access: Option<Box<dyn FnMut(&'c ImplClientRef) + 'c>>,
    ) {
        self.check_access = check_access;
        self.raw.raw.check_access = self
            .check_access
            .as_ref()
            .map(|_| Self::check_access_call());
    }
    pub fn set_global_added(&mut self, global_added: Option<Box<dyn FnMut(&'c GlobalRef) + 'c>>) {
        self.global_added = global_added;
        self.raw.raw.global_added = self
            .global_added
            .as_ref()
            .map(|_| Self::global_added_call());
    }
    pub fn set_global_removed(
        &mut self,
        global_removed: Option<Box<dyn FnMut(&'c GlobalRef) + 'c>>,
    ) {
        self.global_removed = global_removed;
        self.raw.raw.global_removed = self
            .global_removed
            .as_ref()
            .map(|_| Self::global_removed_call());
    }

    pub fn context(&self) -> &ContextRef {
        &self.context
    }

    pub fn hook(&self) -> &Hook {
        &self.hook
    }
}

impl Drop for ContextEvents<'_> {
    fn drop(&mut self) {
        // handled by hook
    }
}

impl Debug for ContextEvents<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextEvents").finish()
    }
}
