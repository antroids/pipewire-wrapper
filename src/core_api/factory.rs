use std::pin::Pin;

use pipewire_macro_impl::spa_interface_call;
use pipewire_proc_macro::{interface, proxy_wrapper, RawWrapper};

use crate::core_api::core::Core;
use crate::core_api::factory::events::FactoryEvents;
use crate::core_api::proxy::{Proxy, ProxyRef};
use crate::core_api::registry::restricted::RegistryBind;
use crate::listeners::{ListenerId, Listeners};
use crate::wrapper::RawWrapper;

pub mod events;
pub mod info;

#[derive(RawWrapper, Debug)]
#[interface(methods=pw_sys::pw_factory_methods, interface="Factory")]
#[repr(transparent)]
pub struct FactoryRef {
    #[raw]
    raw: pw_sys::pw_factory,
}

impl FactoryRef {
    #[must_use]
    pub fn add_listener<'a>(
        &'a self,
        events: Pin<Box<FactoryEvents<'a>>>,
    ) -> Pin<Box<FactoryEvents>> {
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

#[derive(Clone)]
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

impl<'c> Factory<'c> {
    pub fn add_listener(&self, events: Pin<Box<FactoryEvents<'c>>>) -> ListenerId {
        let raw_wrapper = unsafe { FactoryRef::from_raw_ptr(self.ref_.as_raw_ptr().cast()) };
        let mut listener = raw_wrapper.add_listener(events);
        self.listeners.add(listener)
    }

    pub fn remove_listener(&'c mut self, id: ListenerId) -> Option<Pin<Box<FactoryEvents<'c>>>> {
        self.listeners.remove(id)
    }
}
