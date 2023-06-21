use crate::spa::dict::DictRef;
use crate::wrapper::RawWrapper;
use bitflags::bitflags;
use pipewire_proc_macro::RawWrapper;
use std::ffi::CStr;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct CoreInfoRef {
    #[raw]
    raw: pw_sys::pw_core_info,
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct ChangeMask: u64 {
        const PROPS = pw_sys::PW_CORE_CHANGE_MASK_PROPS as u64;
        const ALL = pw_sys::PW_CORE_CHANGE_MASK_ALL as u64;
    }
}

impl CoreInfoRef {
    pub fn id(&self) -> u32 {
        self.raw.id
    }

    pub fn cookie(&self) -> u32 {
        self.raw.cookie
    }

    pub fn user_name(&self) -> Option<&CStr> {
        unsafe { self.raw.user_name.as_ref().map(|ptr| CStr::from_ptr(ptr)) }
    }

    pub fn host_name(&self) -> Option<&CStr> {
        unsafe { self.raw.host_name.as_ref().map(|ptr| CStr::from_ptr(ptr)) }
    }

    pub fn version(&self) -> Option<&CStr> {
        unsafe { self.raw.version.as_ref().map(|ptr| CStr::from_ptr(ptr)) }
    }

    pub fn change_mask(&self) -> ChangeMask {
        ChangeMask::from_bits_retain(self.raw.change_mask)
    }

    pub fn props(&self) -> &DictRef {
        unsafe { DictRef::from_raw_ptr(self.raw.props) }
    }
}
