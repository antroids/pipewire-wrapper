use std::ffi::CStr;
use std::fmt::{Debug, Formatter};
use std::ptr::slice_from_raw_parts;
use std::slice::from_raw_parts;

use bitflags::bitflags;

use pipewire_macro_impl::enum_wrapper;
use pipewire_proc_macro::RawWrapper;

use crate::spa::dict::DictRef;
use crate::spa::param::ParamInfoRef;
use crate::wrapper::RawWrapper;

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct ChangeMask: u64 {
        const INPUT_PORTS = pw_sys::PW_NODE_CHANGE_MASK_INPUT_PORTS as u64;
        const OUTPUT_PORTS = pw_sys::PW_NODE_CHANGE_MASK_OUTPUT_PORTS as u64;
        const STATE = pw_sys::PW_NODE_CHANGE_MASK_STATE as u64;
        const PROPS = pw_sys::PW_NODE_CHANGE_MASK_PROPS as u64;
        const PARAMS = pw_sys::PW_NODE_CHANGE_MASK_PARAMS as u64;
        const ALL = pw_sys::PW_NODE_CHANGE_MASK_ALL as u64;
    }
}

enum_wrapper!(
    NodeState,
    pw_sys::pw_node_state,
    ERROR: pw_sys::pw_node_state_PW_NODE_STATE_ERROR,
    CREATING: pw_sys::pw_node_state_PW_NODE_STATE_CREATING,
    SUSPENDED: pw_sys::pw_node_state_PW_NODE_STATE_SUSPENDED,
    IDLE: pw_sys::pw_node_state_PW_NODE_STATE_IDLE,
    RUNNING: pw_sys::pw_node_state_PW_NODE_STATE_RUNNING,
);

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct NodeInfoRef {
    #[raw]
    raw: pw_sys::pw_node_info,
}

impl NodeInfoRef {
    pub fn id(&self) -> u32 {
        self.raw.id
    }

    pub fn max_input_ports(&self) -> u32 {
        self.raw.max_input_ports
    }

    pub fn max_output_ports(&self) -> u32 {
        self.raw.max_output_ports
    }

    pub fn change_mask(&self) -> ChangeMask {
        ChangeMask::from_bits_retain(self.raw.change_mask)
    }

    pub fn n_input_ports(&self) -> u32 {
        self.raw.n_input_ports
    }

    pub fn n_output_ports(&self) -> u32 {
        self.raw.n_output_ports
    }

    pub fn state(&self) -> NodeState {
        NodeState::from_raw(self.raw.state)
    }

    pub fn error(&self) -> Option<&CStr> {
        unsafe { self.raw.error.as_ref().map(|r| CStr::from_ptr(r)) }
    }

    pub fn props(&self) -> &DictRef {
        unsafe { DictRef::from_raw_ptr(self.raw.props) }
    }

    pub fn params(&self) -> &[ParamInfoRef] {
        unsafe {
            from_raw_parts(
                self.raw.params as *mut ParamInfoRef,
                self.raw.n_params as usize,
            )
        }
    }
}

impl Debug for NodeInfoRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeInfoRef")
            .field("id", &self.id())
            .field("max_input_ports", &self.max_input_ports())
            .field("max_output_ports", &self.max_output_ports())
            .field("change_mask", &self.change_mask())
            .field("n_input_ports", &self.n_input_ports())
            .field("n_output_ports", &self.n_output_ports())
            .field("state", &self.state())
            .field("error", &self.error())
            .field("props", &self.props())
            .field("params", &self.params())
            .finish()
    }
}
