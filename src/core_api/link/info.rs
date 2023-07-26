use std::{ffi::CStr, fmt::Debug};

use bitflags::bitflags;
use pipewire_proc_macro::RawWrapper;

use crate::{
    enum_wrapper,
    spa::{dict::DictRef, pod::PodRef},
    wrapper::RawWrapper,
};

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct LinkInfoRef {
    #[raw]
    raw: pw_sys::pw_link_info,
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct ChangeMask: u64 {
        const STATE = pw_sys::PW_LINK_CHANGE_MASK_STATE as u64;
        const FORMAT = pw_sys::PW_LINK_CHANGE_MASK_FORMAT as u64;
        const PROPS = pw_sys::PW_LINK_CHANGE_MASK_PROPS as u64;
        const ALL = pw_sys::PW_LINK_CHANGE_MASK_ALL as u64;
    }
}

enum_wrapper!(
    LinkState,
    pw_sys::pw_link_state,
    ERROR: pw_sys::pw_link_state_PW_LINK_STATE_ERROR,
    UNLINKED: pw_sys::pw_link_state_PW_LINK_STATE_UNLINKED,
    INIT: pw_sys::pw_link_state_PW_LINK_STATE_INIT,
    NEGOTIATING: pw_sys::pw_link_state_PW_LINK_STATE_NEGOTIATING,
    ALLOCATING: pw_sys::pw_link_state_PW_LINK_STATE_ALLOCATING,
    PAUSED: pw_sys::pw_link_state_PW_LINK_STATE_PAUSED,
    ACTIVE: pw_sys::pw_link_state_PW_LINK_STATE_ACTIVE,
);

impl LinkInfoRef {
    pub fn id(&self) -> u32 {
        self.raw.id
    }

    pub fn output_node_id(&self) -> u32 {
        self.raw.output_node_id
    }

    pub fn output_port_id(&self) -> u32 {
        self.raw.output_port_id
    }

    pub fn input_node_id(&self) -> u32 {
        self.raw.input_node_id
    }

    pub fn input_port_id(&self) -> u32 {
        self.raw.input_port_id
    }

    pub fn change_mask(&self) -> ChangeMask {
        ChangeMask::from_bits_retain(self.raw.change_mask)
    }

    pub fn state(&self) -> LinkState {
        LinkState::from_raw(self.raw.state)
    }

    pub fn error(&self) -> Option<&CStr> {
        unsafe { self.raw.error.as_ref().map(|s| CStr::from_ptr(s)) }
    }

    pub fn format(&self) -> &PodRef {
        unsafe { PodRef::from_raw_ptr(self.raw.format) }
    }

    pub fn props(&self) -> &DictRef {
        unsafe { DictRef::from_raw_ptr(self.raw.props) }
    }
}

impl Debug for LinkInfoRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LinkInfoRef")
            .field("id", &self.raw.id)
            .field("output_node_id", &self.raw.output_node_id)
            .field("output_port_id", &self.raw.output_port_id)
            .field("input_node_id", &self.raw.input_node_id)
            .field("input_port_id", &self.raw.input_port_id)
            .field("change_mask", &self.raw.change_mask)
            .field("state", &self.raw.state)
            .field("error", &self.raw.error)
            .field("format", &self.raw.format)
            .field("props", &self.raw.props)
            .finish()
    }
}
