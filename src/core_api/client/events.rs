use core::slice;
use std::pin::Pin;
use std::ptr::NonNull;

use derive_builder::Builder;
use pw_sys::{pw_client_events, pw_client_info};

use pipewire_macro_impl::events_builder_build;
use pipewire_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::client::info::ClientInfoRef;
use crate::core_api::client::ClientRef;
use crate::core_api::permissions::Permissions;
use crate::spa::interface::Hook;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct ClientEventsRef {
    #[raw]
    raw: pw_sys::pw_client_events,
}

#[derive(Wrapper, Builder)]
#[builder(setter(skip, strip_option), build_fn(skip), pattern = "owned")]
pub struct ClientEvents<'f> {
    #[raw_wrapper]
    ref_: NonNull<ClientEventsRef>,

    raw: Pin<Box<ClientEventsRef>>,
    hook: Pin<Box<Hook>>,

    #[builder(setter)]
    info: Option<Box<dyn for<'a> FnMut(&'a ClientInfoRef) + 'f>>,
    #[builder(setter)]
    permissions: Option<Box<dyn for<'a> FnMut(u32, &'a [Permissions]) + 'f>>,
}

impl Drop for ClientEvents<'_> {
    fn drop(&mut self) {
        // handled by hook
    }
}

impl<'f> ClientEvents<'f> {
    unsafe extern "C" fn info_call(data: *mut ::std::os::raw::c_void, info: *const pw_client_info) {
        if let Some(client_events) = (data as *mut ClientEvents).as_mut() {
            if let Some(callback) = &mut client_events.info {
                callback(ClientInfoRef::from_raw_ptr(info));
            }
        }
    }

    unsafe extern "C" fn permissions_call(
        data: *mut ::std::os::raw::c_void,
        index: u32,
        n_permissions: u32,
        permissions: *const pw_sys::pw_permission,
    ) {
        if let Some(client_events) = (data as *mut ClientEvents).as_mut() {
            if let Some(callback) = &mut client_events.permissions {
                callback(
                    index,
                    slice::from_raw_parts(permissions.cast(), n_permissions as usize),
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

impl<'f> ClientEventsBuilder<'f> {
    events_builder_build! {
        ClientEvents<'f>,
        pw_client_events,
        info => info_call,
        permissions => permissions_call,
    }
}
