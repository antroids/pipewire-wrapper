use std::ffi::CStr;
use std::fmt::{Debug, Formatter};
use std::pin::Pin;
use std::ptr::NonNull;

use derive_builder::Builder;
use pw_sys::{pw_link_events, pw_link_info};
use spa_sys::spa_pod;

use pipewire_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::link::info::LinkInfoRef;
use crate::core_api::link::LinkRef;
use crate::events_builder_build;
use crate::spa::interface::Hook;
use crate::spa::param::{ParamInfoRef, ParamType};
use crate::spa::pod::PodRef;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct LinkEventsRef {
    #[raw]
    raw: pw_sys::pw_link_events,
}

pub type InfoCallback<'p> = Box<dyn for<'a> FnMut(&'a LinkInfoRef) + 'p>;

#[derive(Wrapper, Builder)]
#[builder(setter(skip, strip_option), build_fn(skip), pattern = "owned")]
pub struct LinkEvents<'p> {
    #[raw_wrapper]
    ref_: NonNull<LinkEventsRef>,

    raw: Pin<Box<LinkEventsRef>>,
    hook: Pin<Box<Hook>>,

    #[builder(setter)]
    info: Option<InfoCallback<'p>>,
}

impl<'p> LinkEvents<'p> {
    unsafe extern "C" fn info_call(data: *mut ::std::os::raw::c_void, info: *const pw_link_info) {
        if let Some(events) = (data as *mut LinkEvents).as_mut() {
            if let Some(callback) = &mut events.info {
                callback(LinkInfoRef::from_raw_ptr(info));
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

impl<'p> LinkEventsBuilder<'p> {
    events_builder_build! {
        LinkEvents<'p>,
        pw_link_events,
        info => info_call,
    }
}

impl Debug for LinkEvents<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LinkEvents")
            .field("raw", &self.raw)
            .finish()
    }
}
