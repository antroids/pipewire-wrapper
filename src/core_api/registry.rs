use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use pw_sys::pw_proxy;

use pipewire_macro_impl::spa_interface_call;
use pipewire_proc_macro::{interface, proxy_wrapper, RawWrapper, Wrapper};

use crate::core_api::core::Core;
use crate::core_api::proxy::Proxied;
use crate::core_api::proxy::{Proxy, ProxyRef};
use crate::core_api::registry::events::RegistryEvents;
use crate::core_api::registry::restricted::RegistryBind;
use crate::core_api::type_info::TypeInfo;
use crate::listeners::{ListenerId, Listeners};
use crate::wrapper::*;
use crate::{i32_as_void_result, raw_wrapper};

pub mod events;

#[derive(RawWrapper, Debug)]
#[interface(methods=pw_sys::pw_registry_methods, interface="Registry")]
#[repr(transparent)]
pub struct RegistryRef {
    #[raw]
    raw: pw_sys::pw_registry,
}

impl RegistryRef {
    pub fn add_listener<'a>(
        &'a self,
        events: Pin<Box<RegistryEvents<'a>>>,
    ) -> Pin<Box<RegistryEvents>> {
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

    pub(crate) fn bind(
        &self,
        id: u32,
        type_info: TypeInfo,
        version: u32,
        use_data_size: usize,
    ) -> crate::Result<&ProxyRef> {
        let result = unsafe {
            spa_interface_call!(self, bind, id, type_info.as_ptr(), version, use_data_size)?
        };
        raw_wrapper(result as *mut pw_proxy)
    }

    pub fn destroy(&self, id: u32) -> crate::Result<()> {
        let result = spa_interface_call!(self, destroy, id)?;
        i32_as_void_result(result)
    }
}

#[derive(Clone)]
#[proxy_wrapper(RegistryRef)]
pub struct Registry<'c> {
    ref_: Proxy<'c>,

    listeners: Listeners<Pin<Box<RegistryEvents<'c>>>>,
}

impl<'c> RegistryBind<'c> for Registry<'c> {
    fn from_ref(core: &'c Core, ref_: &ProxyRef) -> Self {
        Self {
            ref_: Proxy::from_ref(core, ref_),
            listeners: Listeners::default(),
        }
    }
}

impl<'c> Registry<'c> {
    pub fn add_listener(&self, events: Pin<Box<RegistryEvents<'c>>>) -> ListenerId {
        let raw_wrapper = unsafe { RegistryRef::from_raw_ptr(self.ref_.as_raw_ptr().cast()) };
        let mut listener = raw_wrapper.add_listener(events);
        self.listeners.add(listener)
    }

    pub fn remove_listener(&'c mut self, id: ListenerId) -> Option<Pin<Box<RegistryEvents<'c>>>> {
        self.listeners.remove(id)
    }
}

impl<'c> Registry<'c> {
    pub fn bind_proxy<T>(&self, id: u32, version: u32, user_data_size: usize) -> crate::Result<T>
    where
        T: RegistryBind<'c>,
        <T as Wrapper>::RawWrapperType: Proxied,
    {
        let type_info = T::RawWrapperType::type_info();
        let ref_ = self.bind(id, type_info, version, user_data_size)?;
        Ok(T::from_ref(&self.ref_.core(), ref_))
    }
}

pub(crate) mod restricted {
    use std::sync::Arc;

    use crate::core_api::proxy::{Proxied, ProxyRef};
    use crate::wrapper::Wrapper;

    pub trait RegistryBind<'c>
    where
        Self: Wrapper,
        Self::RawWrapperType: Proxied,
    {
        fn from_ref(core: &'c crate::core_api::core::Core, ref_: &ProxyRef) -> Self;
    }
}
