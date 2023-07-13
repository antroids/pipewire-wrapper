use std::collections::HashMap;
use std::ffi::{CStr, CString};

use bitflags::bitflags;

use pipewire_proc_macro::RawWrapper;

use crate::spa::dict::DictRef;
use crate::wrapper::RawWrapper;

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

#[derive(Clone, Debug)]
pub struct CoreInfo {
    id: u32,
    cookie: u32,
    user_name: Option<CString>,
    host_name: Option<CString>,
    version: Option<CString>,
    change_mask: ChangeMask,
    props: HashMap<CString, CString>,
}

impl CoreInfo {
    pub fn from_ref(ref_: &CoreInfoRef) -> Self {
        let raw = ref_.raw;
        Self {
            id: raw.id,
            cookie: raw.cookie,
            user_name: ref_.user_name().map(|s| CString::from(s)),
            host_name: ref_.host_name().map(|s| CString::from(s)),
            version: ref_.version().map(|s| CString::from(s)),
            change_mask: ref_.change_mask(),
            props: ref_.props().into(),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn cookie(&self) -> u32 {
        self.cookie
    }
    pub fn user_name(&self) -> &Option<CString> {
        &self.user_name
    }
    pub fn host_name(&self) -> &Option<CString> {
        &self.host_name
    }
    pub fn version(&self) -> &Option<CString> {
        &self.version
    }
    pub fn change_mask(&self) -> ChangeMask {
        self.change_mask
    }
    pub fn props(&self) -> &HashMap<CString, CString> {
        &self.props
    }
}

impl From<&CoreInfoRef> for CoreInfo {
    fn from(value: &CoreInfoRef) -> Self {
        CoreInfo::from_ref(value)
    }
}
