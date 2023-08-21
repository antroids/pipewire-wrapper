/*
 * SPDX-License-Identifier: MIT
 */

//! PipeWire [Node](https://docs.pipewire.org/group__pw__node.html) bindings.
//!
use std::ops::Deref;
use std::pin::Pin;
use std::ptr::null_mut;

use pipewire_wrapper_proc_macro::{interface, proxy_wrapper, RawWrapper};

use crate::core_api::core::Core;
use crate::core_api::node::events::NodeEvents;
use crate::core_api::proxy::{Proxy, ProxyRef};
use crate::core_api::registry::restricted::RegistryBind;
use crate::i32_as_void_result;
use crate::listeners::{AddListener, Listeners, OwnListeners};
use crate::spa::param::ParamType;
use crate::spa::pod::PodRef;
use crate::spa_interface_call;
use crate::wrapper::{RawWrapper, Wrapper};

pub mod events;
pub mod info;

#[derive(RawWrapper, Debug)]
#[interface(methods=pw_sys::pw_node_methods, interface="Node")]
#[repr(transparent)]
pub struct NodeRef {
    #[raw]
    raw: pw_sys::pw_node,
}

impl NodeRef {
    pub fn subscribe_params(&self, param_types: &[ParamType]) -> crate::Result<()> {
        let result = unsafe {
            spa_interface_call!(
                self,
                subscribe_params,
                param_types.as_ptr() as *mut _,
                param_types.len() as u32
            )?
        };
        i32_as_void_result(result)
    }

    pub fn enum_params(
        &self,
        seq: i32,
        id: ParamType,
        start: u32,
        num: u32,
        filter: Option<&PodRef>,
    ) -> crate::Result<()> {
        let result = unsafe {
            spa_interface_call!(
                self,
                enum_params,
                seq,
                *id.as_raw(),
                start,
                num,
                filter.map_or(null_mut(), |f| f as *const _ as *mut _)
            )?
        };
        i32_as_void_result(result)
    }
}

impl<'a> AddListener<'a> for NodeRef {
    type Events = NodeEvents<'a>;

    fn add_listener(&self, events: Pin<Box<Self::Events>>) -> Pin<Box<Self::Events>> {
        unsafe {
            spa_interface_call!(
                self,
                add_listener,
                events.hook().as_raw_ptr(),
                events.as_raw_ptr(),
                &*events as *const _ as *mut _
            )
        };

        events
    }
}

#[derive(Clone, Debug)]
#[proxy_wrapper(NodeRef)]
pub struct Node<'c> {
    ref_: Proxy,

    listeners: Listeners<Pin<Box<NodeEvents<'c>>>>,
}

impl<'c> RegistryBind<'c> for Node<'c> {
    fn from_ref(core: Core, ref_: &ProxyRef) -> Self {
        Self {
            ref_: Proxy::from_ref(core, ref_),
            listeners: Listeners::default(),
        }
    }
}

impl<'a> OwnListeners<'a> for Node<'a> {
    fn listeners(
        &self,
    ) -> &Listeners<Pin<Box<<<Self as Wrapper>::RawWrapperType as AddListener<'a>>::Events>>> {
        &self.listeners
    }
}
