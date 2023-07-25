use std::pin::Pin;

use pipewire_proc_macro::{interface, proxy_wrapper, RawWrapper};

use crate::core_api::core::Core;
use crate::core_api::factory::events::FactoryEvents;
use crate::core_api::proxy::{Proxy, ProxyRef};
use crate::core_api::registry::restricted::RegistryBind;
use crate::listeners::{AddListener, ListenerId, Listeners, OwnListeners};
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

impl<'a> AddListener<'a> for FactoryRef {
    type Events = FactoryEvents<'a>;

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
pub struct Factory<'c> {
    ref_: Proxy<'c>,

    listeners: Listeners<Pin<Box<FactoryEvents<'c>>>>,
}

impl<'c> RegistryBind<'c> for Factory<'c> {
    fn from_ref(core: &'c Core, ref_: &ProxyRef) -> Self {
        Self {
            ref_: Proxy::from_ref(core, ref_),
            listeners: Listeners::default(),
        }
    }
}

impl<'a> OwnListeners<'a> for Factory<'a> {
    fn listeners(
        &self,
    ) -> &Listeners<Pin<Box<<<Self as Wrapper>::RawWrapperType as AddListener<'a>>::Events>>> {
        &self.listeners
    }
}
