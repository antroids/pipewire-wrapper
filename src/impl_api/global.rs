/*
 * SPDX-License-Identifier: MIT
 */
use std::ffi::{c_char, CStr, CString};
use std::fmt::{Debug, Formatter};
use std::ptr::{null, NonNull};
use std::rc::Rc;

use bitflags::Flags;

use pipewire_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::context::{Context, ContextRef};
use crate::core_api::permissions::Permissions;
use crate::core_api::properties::PropertiesRef;
use crate::core_api::type_info::TypeInfo;
use crate::error::Error;
use crate::i32_as_void_result;
use crate::impl_api::impl_client::{ImplClient, ImplClientRef};
use crate::spa::dict::DictRef;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct GlobalRef {
    #[raw]
    raw: pw_sys::pw_global,
}

#[derive(Wrapper)]
pub struct Global {
    #[raw_wrapper]
    ref_: NonNull<GlobalRef>,

    context: std::sync::Arc<Context>,
    bind_func: Box<dyn FnMut() -> crate::Result<i32>>,
}

impl Debug for Global {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Global")
            .field("id", &self.get_id())
            .field("type", &self.get_type())
            .finish()
    }
}

impl Drop for Global {
    fn drop(&mut self) {
        unsafe { pw_sys::pw_global_destroy(self.as_raw_ptr()) }
    }
}

// todo new

impl GlobalRef {
    pub fn register(&self) -> crate::Result<()> {
        let result = unsafe { pw_sys::pw_global_register(self.as_raw_ptr()) };
        i32_as_void_result(result)
    }

    //todo add_listener

    pub fn get_permissions(&self, client: &ImplClient) -> Permissions {
        let permission =
            unsafe { pw_sys::pw_global_get_permissions(self.as_raw_ptr(), client.as_raw_ptr()) };
        Permissions::from_bits_retain(permission)
    }

    pub fn get_context(&self) -> &ContextRef {
        unsafe { ContextRef::from_raw_ptr(pw_sys::pw_global_get_context(self.as_raw_ptr())) }
    }

    pub fn get_type(&self) -> TypeInfo {
        unsafe {
            let ptr = pw_sys::pw_global_get_type(self.as_raw_ptr());
            TypeInfo::from_c_str(CStr::from_ptr(ptr))
        }
    }

    pub fn is_type(&self, type_info: &TypeInfo) -> bool {
        let type_string = CStr::from_bytes_with_nul(type_info.full_type_bytes()).unwrap();
        unsafe { pw_sys::pw_global_is_type(self.as_raw_ptr(), type_string.as_ptr()) }
    }

    pub fn get_version(&self) -> u32 {
        unsafe { pw_sys::pw_global_get_version(self.as_raw_ptr()) }
    }

    pub fn get_properties(&self) -> &PropertiesRef {
        unsafe { PropertiesRef::from_raw_ptr(pw_sys::pw_global_get_properties(self.as_raw_ptr())) }
    }

    pub fn update_keys(&self, source: &DictRef, keys_to_update: &[&CStr]) -> i32 {
        let mut keys: Vec<*const c_char> = keys_to_update
            .to_owned()
            .iter()
            .map(|k| k.as_ptr())
            .collect();
        keys.push(null() as *const c_char);
        unsafe {
            pw_sys::pw_global_update_keys(self.as_raw_ptr(), source.as_raw_ptr(), keys.as_ptr())
        }
    }

    //todo get_object

    pub fn get_id(&self) -> u32 {
        unsafe { pw_sys::pw_global_get_id(self.as_raw_ptr()) }
    }

    pub fn get_serial(&self) -> u64 {
        unsafe { pw_sys::pw_global_get_serial(self.as_raw_ptr()) }
    }

    //todo add_resource
    //todo for_each_resource
    //todo bind

    pub fn update_permissions(
        &self,
        client: &ImplClientRef,
        old: Permissions,
        new: Permissions,
    ) -> crate::Result<()> {
        let result = unsafe {
            pw_sys::pw_global_update_permissions(
                self.as_raw_ptr(),
                client.as_raw_ptr(),
                old.bits(),
                new.bits(),
            )
        };
        i32_as_void_result(result)
    }
}
