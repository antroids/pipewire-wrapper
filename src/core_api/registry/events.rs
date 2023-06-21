use std::ffi::CStr;
use std::pin::Pin;
use std::ptr::NonNull;

use pipewire_proc_macro::{RawWrapper, Wrapper};
use spa_sys::spa_dict;

use crate::core_api::core::Core;
use crate::core_api::permissions::Permissions;
use crate::core_api::registry::RegistryRef;
use crate::core_api::type_info::TypeInfo;
use crate::spa::dict::DictRef;
use crate::spa::interface::Hook;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct RegistryEventsRef {
    #[raw]
    raw: pw_sys::pw_registry_events,
}

#[derive(Wrapper)]
#[repr(C)]
pub struct RegistryEvents<'r> {
    #[raw_wrapper]
    ref_: NonNull<RegistryEventsRef>,

    registry: &'r RegistryRef,

    raw: Pin<Box<RegistryEventsRef>>,
    hook: Pin<Box<Hook>>,

    global: Option<Box<dyn FnMut(u32, Permissions, TypeInfo<'r>, u32, &'r DictRef) + 'r>>,
    global_remove: Option<Box<dyn FnMut(u32) + 'r>>,
}

impl Drop for RegistryEvents<'_> {
    fn drop(&mut self) {
        // handled by hook
    }
}

impl<'r> RegistryEvents<'r> {
    pub(crate) fn new(registry: &'r RegistryRef) -> Pin<Box<Self>> {
        let raw = RegistryEventsRef::from_raw(pw_sys::pw_registry_events {
            version: 0,
            global: None,
            global_remove: None,
        });
        let pinned_raw = Box::into_pin(Box::new(raw));

        Box::into_pin(Box::new(Self {
            ref_: NonNull::new(pinned_raw.as_ptr()).unwrap(),
            registry,
            raw: pinned_raw,
            hook: Hook::new(),
            global: None,
            global_remove: None,
        }))
    }

    fn global_call() -> unsafe extern "C" fn(
        object: *mut ::std::os::raw::c_void,
        id: u32,
        permissions: u32,
        type_: *const ::std::os::raw::c_char,
        version: u32,
        props: *const spa_dict,
    ) {
        unsafe extern "C" fn call(
            object: *mut ::std::os::raw::c_void,
            id: u32,
            permissions: u32,
            type_: *const ::std::os::raw::c_char,
            version: u32,
            props: *const spa_dict,
        ) {
            if let Some(registry_events) = (object as *mut RegistryEvents).as_mut() {
                if let Some(callback) = &mut registry_events.global {
                    callback(
                        id,
                        Permissions::from_bits_retain(permissions),
                        TypeInfo::from_c_str(CStr::from_ptr(type_)),
                        version,
                        DictRef::from_raw_ptr(props),
                    );
                }
            }
        }
        call
    }

    fn global_remove_call() -> unsafe extern "C" fn(object: *mut ::std::os::raw::c_void, id: u32) {
        unsafe extern "C" fn call(object: *mut ::std::os::raw::c_void, id: u32) {
            if let Some(registry_events) = (object as *mut RegistryEvents).as_mut() {
                if let Some(callback) = &mut registry_events.global_remove {
                    callback(id);
                }
            }
        }
        call
    }

    pub fn set_global(
        &mut self,
        global: Option<Box<dyn FnMut(u32, Permissions, TypeInfo<'r>, u32, &'r DictRef) + 'r>>,
    ) {
        self.global = global;
        self.raw.raw.global = self.global.as_ref().map(|_| Self::global_call());
    }

    pub fn set_global_remove(&mut self, global_remove: Option<Box<dyn FnMut(u32) + 'r>>) {
        self.global_remove = global_remove;
        self.raw.raw.global_remove = self
            .global_remove
            .as_ref()
            .map(|_| Self::global_remove_call());
    }

    pub fn hook(&self) -> &Pin<Box<Hook>> {
        &self.hook
    }
}
