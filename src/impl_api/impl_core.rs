/*
 * SPDX-License-Identifier: MIT
 */
use std::ptr::NonNull;
use std::rc::Rc;

use pipewire_wrapper_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::context::Context;
use crate::core_api::core::info::CoreInfoRef;
use crate::core_api::properties::{Properties, PropertiesRef};
use crate::impl_api::global::GlobalRef;
use crate::spa::dict::DictRef;
use crate::wrapper::{RawWrapper, Wrapper};
use crate::{i32_as_result, new_instance_raw_wrapper};

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct ImplCoreRef {
    #[raw]
    raw: pw_sys::pw_impl_core,
}

#[derive(Wrapper, Debug)]
pub struct ImplCore {
    #[raw_wrapper]
    ref_: NonNull<ImplCoreRef>,

    context: std::sync::Arc<Context>,
}

impl Drop for ImplCore {
    fn drop(&mut self) {
        unsafe { pw_sys::pw_impl_core_destroy(self.as_raw_ptr()) }
    }
}

impl ImplCore {
    pub fn new(context: &std::sync::Arc<Context>, properties: Properties) -> crate::Result<Self> {
        let ptr = unsafe {
            pw_sys::pw_context_create_core(context.as_raw_ptr(), properties.into_raw(), 0)
        };
        Ok(Self {
            ref_: new_instance_raw_wrapper(ptr)?,
            context: context.clone(),
        })
    }

    pub fn context(&self) -> &std::sync::Arc<Context> {
        &self.context
    }
}

impl ImplCoreRef {
    pub fn get_default_core(context: &std::sync::Arc<Context>) -> &Self {
        unsafe { Self::from_raw_ptr(pw_sys::pw_context_get_default_core(context.as_raw_ptr())) }
    }

    pub fn get_properties(&self) -> &PropertiesRef {
        unsafe {
            PropertiesRef::from_raw_ptr(pw_sys::pw_impl_core_get_properties(self.as_raw_ptr()))
        }
    }

    pub fn update_properties(&self, properties: &DictRef) -> i32 {
        unsafe {
            pw_sys::pw_impl_core_update_properties(self.as_raw_ptr(), properties.as_raw_ptr())
        }
    }

    pub fn get_info(&self) -> &CoreInfoRef {
        unsafe { CoreInfoRef::from_raw_ptr(pw_sys::pw_impl_core_get_info(self.as_raw_ptr())) }
    }

    pub fn register(&self, properties: Properties) -> crate::Result<()> {
        let result =
            unsafe { pw_sys::pw_impl_core_register(self.as_raw_ptr(), properties.into_raw()) };
        i32_as_result(result, ())
    }

    unsafe fn get_user_data<T>(&self) -> Option<&mut T> {
        let ptr = pw_sys::pw_impl_core_get_user_data(self.as_raw_ptr()) as *mut T;
        ptr.as_mut()
    }

    pub fn get_global(&self) -> &GlobalRef {
        unsafe { GlobalRef::from_raw_ptr(pw_sys::pw_impl_core_get_global(self.as_raw_ptr())) }
    }

    //todo add_listener
}
