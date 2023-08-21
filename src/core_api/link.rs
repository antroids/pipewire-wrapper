/*
 * SPDX-License-Identifier: MIT
 */

//! PipeWire [Link](https://docs.pipewire.org/group__pw__link.html) bindings.
//!
use std::pin::Pin;

use pipewire_wrapper_proc_macro::{interface, proxy_wrapper, RawWrapper};

use crate::wrapper::RawWrapper;
use crate::{
    listeners::{AddListener, Listeners, OwnListeners},
    spa_interface_call,
    wrapper::Wrapper,
};

use super::{
    core::Core,
    proxy::{Proxy, ProxyRef},
    registry::restricted::RegistryBind,
};

use self::events::LinkEvents;

pub mod events;
pub mod info;

#[derive(RawWrapper, Debug)]
#[interface(methods=pw_sys::pw_link_methods, interface="Link")]
#[repr(transparent)]
pub struct LinkRef {
    #[raw]
    raw: pw_sys::pw_link,
}

impl<'a> AddListener<'a> for LinkRef {
    type Events = LinkEvents<'a>;

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

#[derive(Clone, Debug)]
#[proxy_wrapper(LinkRef)]
pub struct Link<'c> {
    ref_: Proxy,

    listeners: Listeners<Pin<Box<LinkEvents<'c>>>>,
}

impl<'c> RegistryBind<'c> for Link<'c> {
    fn from_ref(core: Core, ref_: &ProxyRef) -> Self {
        Self {
            ref_: Proxy::from_ref(core, ref_),
            listeners: Listeners::default(),
        }
    }
}

impl<'a> OwnListeners<'a> for Link<'a> {
    fn listeners(
        &self,
    ) -> &Listeners<Pin<Box<<<Self as Wrapper>::RawWrapperType as AddListener<'a>>::Events>>> {
        &self.listeners
    }
}
