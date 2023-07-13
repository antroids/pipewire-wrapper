use std::fmt::{Debug, Formatter};
use std::pin::Pin;
use std::ptr::NonNull;
use std::rc::Rc;

use derive_builder::Builder;
use pw_sys::{pw_context_events, pw_global, pw_impl_client};

use pipewire_macro_impl::events_builder_build;
use pipewire_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::context::{Context, ContextRef};
use crate::impl_api::global::GlobalRef;
use crate::impl_api::impl_client::ImplClientRef;
use crate::spa::interface::Hook;
use crate::wrapper::{RawWrapper, Wrapper};

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct ContextEventsRef {
    #[raw]
    raw: pw_sys::pw_context_events,
}

#[derive(Wrapper, Builder)]
#[builder(setter(skip, strip_option), build_fn(skip), pattern = "owned")]
pub struct ContextEvents<'c> {
    #[raw_wrapper]
    ref_: NonNull<ContextEventsRef>,

    raw: Pin<Box<ContextEventsRef>>,
    hook: Pin<Box<Hook>>,

    #[builder(setter)]
    destroy: Option<Box<dyn FnMut() + 'c>>,
    #[builder(setter)]
    free: Option<Box<dyn FnMut() + 'c>>,
    #[builder(setter)]
    check_access: Option<Box<dyn for<'a> FnMut(&'a ImplClientRef) + 'c>>,
    #[builder(setter)]
    global_added: Option<Box<dyn for<'a> FnMut(&'a GlobalRef) + 'c>>,
    #[builder(setter)]
    global_removed: Option<Box<dyn for<'a> FnMut(&'a GlobalRef) + 'c>>,
}

impl<'c> ContextEvents<'c> {
    unsafe extern "C" fn destroy_call(data: *mut ::std::os::raw::c_void) {
        if let Some(context_events) = (data as *mut ContextEvents).as_mut() {
            if let Some(callback) = &mut context_events.destroy {
                callback();
            }
        }
    }

    unsafe extern "C" fn free_call(data: *mut ::std::os::raw::c_void) {
        if let Some(context_events) = (data as *mut ContextEvents).as_mut() {
            if let Some(callback) = &mut context_events.free {
                callback();
            }
        }
    }

    unsafe extern "C" fn check_access_call(
        data: *mut ::std::os::raw::c_void,
        client: *mut pw_impl_client,
    ) {
        if let Some(context_events) = (data as *mut ContextEvents).as_mut() {
            if let Some(callback) = &mut context_events.check_access {
                callback(ImplClientRef::from_raw_ptr(client));
            }
        }
    }

    unsafe extern "C" fn global_added_call(
        data: *mut ::std::os::raw::c_void,
        global: *mut pw_global,
    ) {
        if let Some(context_events) = (data as *mut ContextEvents).as_mut() {
            if let Some(callback) = &mut context_events.global_added {
                callback(GlobalRef::from_raw_ptr(global));
            }
        }
    }

    unsafe extern "C" fn global_removed_call(
        data: *mut ::std::os::raw::c_void,
        global: *mut pw_global,
    ) {
        if let Some(context_events) = (data as *mut ContextEvents).as_mut() {
            if let Some(callback) = &mut context_events.global_removed {
                callback(GlobalRef::from_raw_ptr(global));
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

// todo: channel builder

impl<'c> ContextEventsBuilder<'c> {
    events_builder_build! {
        ContextEvents<'c>,
        pw_context_events,
        destroy => destroy_call,
        free => free_call,
        check_access => check_access_call,
        global_added => global_added_call,
        global_removed => global_removed_call,
    }
}
