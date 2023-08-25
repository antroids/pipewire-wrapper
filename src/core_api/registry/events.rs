/*
 * SPDX-License-Identifier: MIT
 */
use std::ffi::CStr;
use std::fmt::{Debug, Formatter};
use std::pin::Pin;
use std::ptr::NonNull;

use bitflags::Flags;
use derive_builder::Builder;
use pw_sys::pw_registry_events;
use spa_sys::spa_dict;

use pipewire_wrapper_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::core::Core;
use crate::core_api::permissions::Permissions;
use crate::core_api::registry::RegistryRef;
use crate::core_api::type_info::TypeInfo;
use crate::events_builder_build;
use crate::spa::dict::DictRef;
use crate::spa::interface::Hook;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct RegistryEventsRef {
    #[raw]
    raw: pw_sys::pw_registry_events,
}

pub type GlobalCallback = Box<dyn for<'a> FnMut(u32, Permissions, TypeInfo<'a>, u32, &'a DictRef)>;
pub type GlobalRemoveCallback = Box<dyn FnMut(u32)>;

#[derive(Wrapper, Builder)]
#[builder(setter(skip, strip_option), build_fn(skip), pattern = "owned")]
pub struct RegistryEvents {
    #[raw_wrapper]
    ref_: NonNull<RegistryEventsRef>,

    raw: Pin<Box<RegistryEventsRef>>,
    hook: Pin<Box<Hook>>,

    #[builder(setter)]
    global: Option<GlobalCallback>,
    #[builder(setter)]
    global_remove: Option<GlobalRemoveCallback>,
}

impl RegistryEvents {
    unsafe extern "C" fn global_call(
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

    unsafe extern "C" fn global_remove_call(object: *mut ::std::os::raw::c_void, id: u32) {
        if let Some(registry_events) = (object as *mut RegistryEvents).as_mut() {
            if let Some(callback) = &mut registry_events.global_remove {
                callback(id);
            }
        }
    }

    pub fn hook(&self) -> &Pin<Box<Hook>> {
        &self.hook
    }

    pub fn version(&self) -> u32 {
        self.raw.raw.version
    }
}

// todo: channel builder

impl RegistryEventsBuilder {
    events_builder_build! {
        RegistryEvents,
        pw_registry_events,
        global => global_call,
        global_remove => global_remove_call,
    }
}

impl Debug for RegistryEvents {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegistryEvents")
            .field("raw", &self.raw)
            .finish()
    }
}
