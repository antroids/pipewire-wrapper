/*
 * SPDX-License-Identifier: MIT
 */
use std::ffi::CStr;
use std::fmt::{Debug, Formatter};
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::mpsc;

use derive_builder::Builder;
use pw_sys::{pw_node_events, pw_node_info};
use spa_sys::spa_pod;

use pipewire_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::loop_;
use crate::core_api::loop_::channel::{Receiver, Sender};
use crate::core_api::node::info::{NodeInfo, NodeInfoRef};
use crate::core_api::node::NodeRef;
use crate::spa::interface::Hook;
use crate::spa::param::{ParamInfoRef, ParamType};
use crate::spa::pod::pod_buf::{AllocatedData, PodBuf};
use crate::spa::pod::PodRef;
use crate::wrapper::RawWrapper;
use crate::{events_builder_build, events_channel_builder};

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct NodeEventsRef {
    #[raw]
    raw: pw_sys::pw_node_events,
}

pub type InfoCallback<'p> = Box<dyn for<'a> FnMut(&'a NodeInfoRef) + 'p>;
pub type ParamCallback<'p> = Box<dyn for<'a> FnMut(i32, ParamType, u32, u32, &'a PodRef) + 'p>;

#[derive(Wrapper, Builder)]
#[builder(setter(skip, strip_option), build_fn(skip), pattern = "owned")]
pub struct NodeEvents<'p> {
    #[raw_wrapper]
    ref_: NonNull<NodeEventsRef>,

    raw: Pin<Box<NodeEventsRef>>,
    hook: Pin<Box<Hook>>,

    #[builder(setter)]
    info: Option<InfoCallback<'p>>,
    #[builder(setter)]
    param: Option<ParamCallback<'p>>,
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

#[derive(Clone, Debug)]
pub enum NodeEventType {
    Info(NodeInfo),
    Param(i32, ParamType, u32, u32, AllocatedData<PodRef>),
}

impl<'p> NodeEventsChannelBuilder<'p> {
    fn info_send(sender: Sender<'p, NodeEventType>) -> InfoCallback<'p> {
        Box::new(move |i| {
            sender.send(NodeEventType::Info(i.into()));
        })
    }

    fn param_send(sender: Sender<'p, NodeEventType>) -> ParamCallback<'p> {
        Box::new(move |seq, type_, index, next, pod| {
            if let Ok(pod) = AllocatedData::from_pod(pod) {
                sender.send(NodeEventType::Param(seq, type_, index, next, pod));
            }
        })
    }
}

events_channel_builder! {
    Node,
    info => info_send,
    param => param_send,
}

impl<'p> NodeEventsBuilder<'p> {
    events_builder_build! {
        NodeEvents<'p>,
        pw_node_events,
        info => info_call,
        param => param_call,
    }
}

impl Debug for NodeEvents<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeEvents")
            .field("raw", &self.raw)
            .finish()
    }
}
