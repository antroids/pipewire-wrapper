use crate::core_api::port::Direction;
use crate::spa::dict::DictRef;
use crate::spa::param::ParamInfoRef;
use crate::wrapper::RawWrapper;
use bitflags::bitflags;
use pipewire_proc_macro::RawWrapper;
use std::fmt::Debug;
use std::ptr::slice_from_raw_parts;
use std::slice::from_raw_parts;

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct ChangeMask: u64 {
        const PROPS = pw_sys::PW_PORT_CHANGE_MASK_PROPS as u64;
        const PARAMS = pw_sys::PW_PORT_CHANGE_MASK_PARAMS as u64;
        const ALL = pw_sys::PW_PORT_CHANGE_MASK_ALL as u64;
    }
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PortInfoRef {
    #[raw]
    raw: pw_sys::pw_port_info,
}

impl PortInfoRef {
    pub fn id(&self) -> u32 {
        self.raw.id
    }

    pub fn direction(&self) -> Direction {
        Direction::from_raw(self.raw.direction)
    }

    pub fn change_mask(&self) -> ChangeMask {
        ChangeMask::from_bits_retain(self.raw.change_mask)
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
