use std::ffi::CStr;
use std::pin::Pin;
use std::ptr::NonNull;

use derive_builder::Builder;
use pw_sys::{pw_node_events, pw_node_info};
use spa_sys::spa_pod;

use pipewire_macro_impl::events_builder_build;
use pipewire_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::node::info::NodeInfoRef;
use crate::core_api::node::NodeRef;
use crate::spa::interface::Hook;
use crate::spa::param::{ParamInfoRef, ParamType};
use crate::spa::type_::pod::PodRef;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct NodeEventsRef {
    #[raw]
    raw: pw_sys::pw_node_events,
}

#[derive(Wrapper, Builder)]
#[builder(setter(skip, strip_option), build_fn(skip), pattern = "owned")]
pub struct NodeEvents<'p> {
    #[raw_wrapper]
    ref_: NonNull<NodeEventsRef>,

    raw: Pin<Box<NodeEventsRef>>,
    hook: Pin<Box<Hook>>,

    #[builder(setter)]
    info: Option<Box<dyn for<'a> FnMut(&'a NodeInfoRef) + 'p>>,
    #[builder(setter)]
    param: Option<Box<dyn for<'a> FnMut(i32, ParamType, u32, u32, &'a PodRef) + 'p>>,
}

impl Drop for NodeEvents<'_> {
    fn drop(&mut self) {
        // handled by hook
    }
}

impl<'p> NodeEvents<'p> {
    unsafe extern "C" fn info_call(data: *mut ::std::os::raw::c_void, info: *const pw_node_info) {
        if let Some(events) = (data as *mut NodeEvents).as_mut() {
            if let Some(callback) = &mut events.info {
                callback(NodeInfoRef::from_raw_ptr(info));
            }
        }
    }

    unsafe extern "C" fn param_call(
        data: *mut ::std::os::raw::c_void,
        seq: ::std::os::raw::c_int,
        id: u32,
        index: u32,
        next: u32,
        param: *const spa_pod,
    ) {
        if let Some(events) = (data as *mut NodeEvents).as_mut() {
            if let Some(callback) = &mut events.param {
                callback(
                    seq,
                    ParamType::from_raw(id),
                    index,
                    next,
                    PodRef::from_raw_ptr(param),
                );
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

impl<'p> NodeEventsBuilder<'p> {
    events_builder_build! {
        NodeEvents<'p>,
        pw_node_events,
        info => info_call,
        param => param_call,
    }
}
