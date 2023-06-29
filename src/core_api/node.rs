use std::pin::Pin;
use std::ptr::{null_mut, NonNull};

use pipewire_macro_impl::{enum_wrapper, spa_interface_call};
use pipewire_proc_macro::{proxied, RawWrapper, Wrapper};

use crate::core_api::node::events::NodeEvents;
use crate::core_api::type_info::TypeInfo;
use crate::core_api::Pipewire;
use crate::i32_as_void_result;
use crate::spa::param::ParamType;
use crate::spa::type_::pod::PodRef;
use crate::wrapper::RawWrapper;

pub mod events;
pub mod info;

#[derive(RawWrapper, Debug)]
#[proxied(methods=pw_sys::pw_node_methods, interface="Node")]
#[repr(transparent)]
pub struct NodeRef {
    #[raw]
    raw: pw_sys::pw_node,
}

impl NodeRef {
    pub fn add_listener(&self) -> Pin<Box<NodeEvents>> {
        let mut events = NodeEvents::new(self);

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
