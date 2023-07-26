/*
 * SPDX-License-Identifier: MIT
 */
use std::pin::Pin;

use pipewire_proc_macro::{interface, proxy_wrapper, RawWrapper};

use crate::core_api::client::events::ClientEvents;
use crate::core_api::core::Core;
use crate::core_api::proxy::{Proxy, ProxyRef};
use crate::core_api::registry::restricted::RegistryBind;
use crate::listeners::{AddListener, Listeners, OwnListeners};
use crate::spa_interface_call;
use crate::wrapper::{RawWrapper, Wrapper};

pub mod events;
pub mod info;

/// Wrapper for the external [pw_sys::pw_client] value.
#[derive(RawWrapper, Debug)]
#[interface(methods=pw_sys::pw_client_methods, interface="Client")]
#[repr(transparent)]
pub struct ClientRef {
    #[raw]
    raw: pw_sys::pw_client,
}

impl<'a> AddListener<'a> for ClientRef {
    type Events = ClientEvents<'a>;

    fn add_listener(&self, events: Pin<Box<Self::Events>>) -> Pin<Box<Self::Events>> {
        unsafe {
            spa_interface_call!(
                self,
                add_listener,
                events.hook().as_raw_ptr(),
                events.as_raw_ptr(),
                &*events as *const _ as *mut _
            )
        };

        events
    }
}

/// Wrapper for the Client proxy, can be obtained from the [crate::core_api::registry::Registry].
#[derive(Clone)]
#[proxy_wrapper(ClientRef)]
pub struct Client<'c> {
    ref_: Proxy<'c>,

    listeners: Listeners<Pin<Box<ClientEvents<'c>>>>,
}

impl<'c> RegistryBind<'c> for Client<'c> {
    fn from_ref(core: &'c Core, ref_: &ProxyRef) -> Self {
        Self {
            ref_: Proxy::from_ref(core, ref_),
            listeners: Listeners::default(),
        }
    }
}

impl<'a> OwnListeners<'a> for Client<'a> {
    fn listeners(
        &self,
    ) -> &Listeners<Pin<Box<<<Self as Wrapper>::RawWrapperType as AddListener<'a>>::Events>>> {
        &self.listeners
    }
}
