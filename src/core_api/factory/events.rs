/*
 * SPDX-License-Identifier: MIT
 */
use std::fmt::{Debug, Formatter};
use std::pin::Pin;
use std::ptr::NonNull;

use derive_builder::Builder;
use pw_sys::{pw_factory_events, pw_factory_info};

use pipewire_wrapper_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::factory::info::FactoryInfoRef;
use crate::core_api::factory::FactoryRef;
use crate::events_builder_build;
use crate::spa::interface::Hook;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct FactoryEventsRef {
    #[raw]
    raw: pw_sys::pw_factory_events,
}

pub type InfoCallback<'f> = Box<dyn for<'a> FnMut(&'a FactoryInfoRef) + 'f>;

#[derive(Wrapper, Builder)]
#[builder(setter(skip, strip_option), build_fn(skip), pattern = "owned")]
pub struct FactoryEvents<'f> {
    #[raw_wrapper]
    ref_: NonNull<FactoryEventsRef>,

    raw: Pin<Box<FactoryEventsRef>>,
    hook: Pin<Box<Hook>>,

    #[builder(setter)]
    info: Option<InfoCallback<'f>>,
}

impl<'f> FactoryEvents<'f> {
    unsafe extern "C" fn info_call(
        data: *mut ::std::os::raw::c_void,
        info: *const pw_factory_info,
    ) {
        if let Some(factory_events) = (data as *mut FactoryEvents).as_mut() {
            if let Some(callback) = &mut factory_events.info {
                callback(FactoryInfoRef::from_raw_ptr(info));
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

impl<'f> FactoryEventsBuilder<'f> {
    events_builder_build! {
        FactoryEvents<'f>,
        pw_factory_events,
        info => info_call,
    }
}

impl Debug for FactoryEvents<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FactoryEvents")
            .field("raw", &self.raw)
            .finish()
    }
}
