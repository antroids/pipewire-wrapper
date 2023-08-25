/*
 * SPDX-License-Identifier: MIT
 */

//! PipeWire [Factory](https://docs.pipewire.org/group__pw__factory.html) bindings.
//!
use std::pin::Pin;

use pipewire_wrapper_proc_macro::{interface, proxy_wrapper, RawWrapper};

use crate::core_api::core::Core;
use crate::core_api::factory::events::FactoryEvents;
use crate::core_api::proxy::{Proxy, ProxyRef};
use crate::core_api::registry::restricted::RegistryBind;
use crate::listeners::{AddListener, Listeners, OwnListeners};
use crate::spa_interface_call;
use crate::wrapper::{RawWrapper, Wrapper};

pub mod events;
pub mod info;

#[derive(RawWrapper, Debug)]
#[interface(methods=pw_sys::pw_factory_methods, interface="Factory")]
#[repr(transparent)]
pub struct FactoryRef {
    #[raw]
    raw: pw_sys::pw_factory,
}

impl AddListener for FactoryRef {
    type Events = FactoryEvents;

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
#[proxy_wrapper(FactoryRef)]
pub struct Factory {
    ref_: Proxy,

    listeners: Listeners<Pin<Box<FactoryEvents>>>,
}

impl RegistryBind for Factory {
    fn from_ref(core: Core, ref_: &ProxyRef) -> Self {
        Self {
            ref_: Proxy::from_ref(core, ref_),
            listeners: Listeners::default(),
        }
    }
}

impl OwnListeners for Factory {
    fn listeners(
        &self,
    ) -> &Listeners<Pin<Box<<<Self as Wrapper>::RawWrapperType as AddListener>::Events>>> {
        &self.listeners
    }
}
