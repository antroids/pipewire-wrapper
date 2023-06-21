use pw_sys::pw_proxy;
use std::pin::Pin;
use std::ptr::NonNull;
use std::rc::Rc;

use pipewire_macro_impl::spa_interface_call;
use pipewire_proc_macro::{proxied, RawWrapper, Wrapper};

use crate::core_api::core::Core;
use crate::core_api::proxy::ProxyRef;
use crate::core_api::registry::events::RegistryEvents;
use crate::core_api::type_info::TypeInfo;
use crate::wrapper::*;
use crate::{i32_as_void_result, raw_wrapper};

pub mod events;

#[derive(RawWrapper, Debug)]
#[proxied(methods=pw_sys::pw_registry_methods, interface="Registry")]
#[repr(transparent)]
pub struct RegistryRef {
    #[raw]
    raw: pw_sys::pw_registry,
}

impl RegistryRef {
    pub fn add_listener(&self) -> Pin<Box<RegistryEvents>> {
        let events = RegistryEvents::new(self);

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

    pub fn bind(
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
