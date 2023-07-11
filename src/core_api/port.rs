use std::pin::Pin;
use std::ptr::{null_mut, NonNull};
use std::sync::Arc;

use pipewire_macro_impl::{enum_wrapper, spa_interface_call};
use pipewire_proc_macro::{interface, proxy_wrapper, RawWrapper, Wrapper};

use crate::core_api::core::Core;
use crate::core_api::port::events::PortEvents;
use crate::core_api::proxy::{Proxy, ProxyRef};
use crate::core_api::registry::restricted::RegistryBind;
use crate::core_api::registry::Registry;
use crate::core_api::type_info::TypeInfo;
use crate::core_api::Pipewire;
use crate::i32_as_void_result;
use crate::listeners::{ListenerId, Listeners};
use crate::spa::param::ParamType;
use crate::spa::type_::pod::PodRef;
use crate::wrapper::{RawWrapper, Wrapper};

pub mod events;
pub mod info;

#[derive(RawWrapper, Debug)]
#[interface(methods=pw_sys::pw_port_methods, interface="Port")]
#[repr(transparent)]
pub struct PortRef {
    #[raw]
    raw: pw_sys::pw_port,
}

impl PortRef {
    pub fn add_listener<'a>(&'a self, events: Pin<Box<PortEvents<'a>>>) -> Pin<Box<PortEvents>> {
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

    pub fn subscribe_params(&self, param_types: &[ParamType]) -> crate::Result<()> {
        let result = unsafe {
            spa_interface_call!(
                self,
                subscribe_params,
                param_types.as_ptr() as *mut _,
                param_types.len() as u32
            )?
        };
        i32_as_void_result(result)
    }

    pub fn enum_params(
        &self,
        seq: i32,
        id: ParamType,
        start: u32,
        num: u32,
        filter: Option<&PodRef>,
    ) -> crate::Result<()> {
        let result = unsafe {
            spa_interface_call!(
                self,
                enum_params,
                seq,
                *id.as_raw(),
                start,
                num,
                filter.map_or(null_mut(), |f| f as *const _ as *mut _)
            )?
        };
        i32_as_void_result(result)
    }
}

#[derive(Clone)]
#[proxy_wrapper(PortRef)]
pub struct Port<'c> {
    ref_: Proxy<'c>,

    listeners: Listeners<Pin<Box<PortEvents<'c>>>>,
}

impl<'c> RegistryBind<'c> for Port<'c> {
    fn from_ref(core: &'c Core, ref_: &ProxyRef) -> Self {
        Self {
            ref_: Proxy::from_ref(core, ref_),
            listeners: Listeners::default(),
        }
    }
}

impl<'c> Port<'c> {
    pub fn add_listener(&self, events: Pin<Box<PortEvents<'c>>>) -> ListenerId {
        let raw_wrapper = unsafe { PortRef::from_raw_ptr(self.ref_.as_raw_ptr().cast()) };
        let mut listener = raw_wrapper.add_listener(events);
        self.listeners.add(listener)
    }

    pub fn remove_listener(&'c mut self, id: ListenerId) -> Option<Pin<Box<PortEvents<'c>>>> {
        self.listeners.remove(id)
    }
}
