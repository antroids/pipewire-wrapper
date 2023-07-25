use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::ptr::NonNull;

use bitflags::bitflags;

use pipewire_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::type_info::TypeInfo;
use crate::spa::dict::DictRef;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct FactoryInfoRef {
    #[raw]
    raw: pw_sys::pw_factory_info,
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct ChangeMask: u64 {
        const PROPS = pw_sys::PW_FACTORY_CHANGE_MASK_PROPS as u64;
        const ALL = pw_sys::PW_FACTORY_CHANGE_MASK_ALL as u64;
    }
}

impl FactoryInfoRef {
    pub fn id(&self) -> u32 {
        self.raw.id
    }

    pub fn name(&self) -> Option<&CStr> {
        unsafe { self.raw.name.as_ref().map(|ptr| CStr::from_ptr(ptr)) }
    }

    pub fn type_(&self) -> Option<TypeInfo> {
        unsafe {
            self.raw
                .type_
                .as_ref()
                .map(|ptr| TypeInfo::from_c_str(CStr::from_ptr(ptr)))
        }
    }

    pub fn version(&self) -> u32 {
        self.raw.version
    }

    pub fn change_mask(&self) -> ChangeMask {
        ChangeMask::from_bits_retain(self.raw.change_mask)
    }

    pub fn props(&self) -> &DictRef {
        unsafe { DictRef::from_raw_ptr(self.raw.props) }
    }
}

#[derive(Clone, Debug)]
pub struct FactoryInfo {
    id: u32,
    name: Option<CString>,
    type_: Option<Box<CStr>>,
    version: u32,
    change_mask: ChangeMask,
    props: HashMap<CString, CString>,
}

impl FactoryInfo {
    pub fn from_ref(ref_: &FactoryInfoRef) -> Self {
        Self {
            id: ref_.id(),
            name: ref_.name().map(CString::from),
            type_: ref_.type_().map(|t| Box::from(t.as_c_str())),
            version: ref_.version(),
            change_mask: ref_.change_mask(),
            props: ref_.props().into(),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn name(&self) -> &Option<CString> {
        &self.name
    }
    pub fn type_(&self) -> Option<TypeInfo> {
        self.type_.as_ref().map(|s| s.as_ref().into())
    }
    pub fn version(&self) -> u32 {
        self.version
    }
    pub fn change_mask(&self) -> ChangeMask {
        self.change_mask
    }
    pub fn props(&self) -> &HashMap<CString, CString> {
        &self.props
    }
}

impl From<&FactoryInfoRef> for FactoryInfo {
    fn from(value: &FactoryInfoRef) -> Self {
        FactoryInfo::from_ref(value)
    }
}
