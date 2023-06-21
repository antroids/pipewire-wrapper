use crate::wrapper::RawWrapper;
use pipewire_macro_impl::{enum_wrapper, spa_interface_call};
use pipewire_proc_macro::{proxied, RawWrapper, Wrapper};
use std::pin::Pin;
use std::ptr::{null_mut, NonNull};

use crate::core_api::port::events::PortEvents;
use crate::core_api::type_info::TypeInfo;
use crate::core_api::Pipewire;
use crate::i32_as_void_result;
use crate::spa::param::ParamType;
use crate::spa::type_::pod::PodRef;

pub mod events;
pub mod info;

enum_wrapper!(
    Direction,
    spa_sys::spa_direction,
    INPUT: spa_sys::SPA_DIRECTION_INPUT,
    OUTPUT: spa_sys::SPA_DIRECTION_OUTPUT,
);

#[derive(RawWrapper, Debug)]
#[proxied(methods=pw_sys::pw_port_methods, interface="Port")]
#[repr(transparent)]
pub struct PortRef {
    #[raw]
    raw: pw_sys::pw_port,
}

impl PortRef {
    pub fn add_listener(&self) -> Pin<Box<PortEvents>> {
        let mut events = PortEvents::new(self);

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
