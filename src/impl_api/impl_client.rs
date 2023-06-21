use std::ptr::NonNull;
use std::rc::Rc;

use pipewire_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::client_info::ClientInfoRef;
use crate::core_api::context::Context;
use crate::core_api::core::Core;
use crate::core_api::properties::{Properties, PropertiesRef};
use crate::impl_api::global::GlobalRef;
use crate::impl_api::impl_core::ImplCore;
use crate::impl_api::protocol::Protocol;
use crate::spa::dict::DictRef;
use crate::wrapper::{RawWrapper, Wrapper};
use crate::{i32_as_result, i32_as_void_result, new_instance_raw_wrapper};

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct ImplClientRef {
    #[raw]
    raw: pw_sys::pw_impl_client,
}

#[derive(Wrapper, Debug)]
pub struct ImplClient {
    #[raw_wrapper]
    ref_: NonNull<ImplClientRef>,

    core: std::sync::Arc<ImplCore>,
    protocol: std::sync::Arc<Protocol>,
}

impl Drop for ImplClient {
    fn drop(&mut self) {
        unsafe { pw_sys::pw_impl_client_destroy(self.as_raw_ptr()) }
    }
}

impl ImplClient {
    pub fn new(
        core: &std::sync::Arc<ImplCore>,
        protocol: &std::sync::Arc<Protocol>,
        properties: Properties,
        user_data_size: usize,
    ) -> crate::Result<Self> {
        let ptr = unsafe {
            pw_sys::pw_context_create_client(
                core.as_raw_ptr(),
                protocol.as_raw_ptr(),
                properties.into_raw(),
                user_data_size,
            )
        };
        Ok(Self {
            ref_: new_instance_raw_wrapper(ptr)?,
            core: core.clone(),
            protocol: protocol.clone(),
        })
    }

    pub fn core(&self) -> &std::sync::Arc<ImplCore> {
        &self.core
    }

    pub fn protocol(&self) -> &std::sync::Arc<Protocol> {
        &self.protocol
    }
}

impl ImplClientRef {
    pub fn get_properties(&self) -> &PropertiesRef {
        unsafe {
            PropertiesRef::from_raw_ptr(pw_sys::pw_impl_client_get_properties(self.as_raw_ptr()))
        }
    }

    pub fn update_properties(&self, properties: &DictRef) -> i32 {
        unsafe {
            pw_sys::pw_impl_client_update_properties(self.as_raw_ptr(), properties.as_raw_ptr())
        }
    }

    pub fn get_info(&self) -> &ClientInfoRef {
        unsafe { ClientInfoRef::from_raw_ptr(pw_sys::pw_impl_client_get_info(self.as_raw_ptr())) }
    }

    pub fn register(&self, properties: Properties) -> crate::Result<()> {
        let result =
            unsafe { pw_sys::pw_impl_client_register(self.as_raw_ptr(), properties.into_raw()) };
        i32_as_result(result, ())
    }

    pub unsafe fn get_user_data<T>(&self) -> &mut T {
        let ptr = pw_sys::pw_impl_client_get_user_data(self.as_raw_ptr()) as *mut T;
        &mut *ptr
    }

    pub fn get_global(&self) -> &GlobalRef {
        unsafe { GlobalRef::from_raw_ptr(pw_sys::pw_impl_client_get_global(self.as_raw_ptr())) }
    }

    //todo add_listener
}
