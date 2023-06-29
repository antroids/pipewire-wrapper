use std::ffi::CStr;
use std::pin::Pin;
use std::ptr::NonNull;

use pw_sys::pw_node_info;
use spa_sys::spa_pod;

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

#[derive(Wrapper)]
pub struct NodeEvents<'p> {
    #[raw_wrapper]
    ref_: NonNull<NodeEventsRef>,

    node: &'p NodeRef,

    raw: Pin<Box<NodeEventsRef>>,
    hook: Pin<Box<Hook>>,

    info: Option<Box<dyn FnMut(&'p NodeInfoRef) + 'p>>,
    param: Option<Box<dyn FnMut(i32, ParamType, u32, u32, &'p PodRef) + 'p>>,
}

impl Drop for NodeEvents<'_> {
    fn drop(&mut self) {
        // handled by hook
    }
}

impl<'p> NodeEvents<'p> {
    pub(crate) fn new(node: &'p NodeRef) -> Pin<Box<Self>> {
        let hook = Hook::new();
        let raw = NodeEventsRef::from_raw(pw_sys::pw_node_events {
            version: 0,
            info: None,
            param: None,
        });
        let mut pinned_raw = Box::into_pin(Box::new(raw));

        Box::into_pin(Box::new(Self {
            ref_: NonNull::new(pinned_raw.as_ptr()).unwrap(),
            node,
            raw: pinned_raw,
            hook,
            info: None,
            param: None,
        }))
    }

    fn info_call(
    ) -> unsafe extern "C" fn(data: *mut ::std::os::raw::c_void, info: *const pw_node_info) {
        unsafe extern "C" fn call(data: *mut ::std::os::raw::c_void, info: *const pw_node_info) {
            if let Some(events) = (data as *mut NodeEvents).as_mut() {
                if let Some(callback) = &mut events.info {
                    callback(NodeInfoRef::from_raw_ptr(info));
                }
            }
        }
        call
    }

    fn param_call() -> unsafe extern "C" fn(
        data: *mut ::std::os::raw::c_void,
        seq: ::std::os::raw::c_int,
        id: u32,
        index: u32,
        next: u32,
        param: *const spa_pod,
    ) {
        unsafe extern "C" fn call(
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
        call
    }

    pub fn set_info(&mut self, info: Option<Box<dyn FnMut(&'p NodeInfoRef) + 'p>>) {
        self.info = info;
        self.raw.raw.info = self.info.as_ref().map(|_| Self::info_call());
    }

    pub fn set_param(
        &mut self,
        param: Option<Box<dyn FnMut(i32, ParamType, u32, u32, &'p PodRef) + 'p>>,
    ) {
        self.param = param;
        self.raw.raw.param = self.param.as_ref().map(|_| Self::param_call());
    }

    pub fn node(&self) -> &'p NodeRef {
        self.node
    }
    pub fn hook(&self) -> &Pin<Box<Hook>> {
        &self.hook
    }
}
