use crate::core_api::type_info::TypeInfo;
use crate::wrapper::RawWrapper;
use bitflags::bitflags;
use pipewire_proc_macro::{RawWrapper, Wrapper};
use std::ffi::CStr;
use std::ptr::NonNull;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct FactoryInfoRef {
    #[raw]
    raw: pw_sys::pw_factory_info,
}

// #[derive(Wrapper)]
// pub struct FactoryInfo {
//     #[raw_wrapper]
//     ref_: NonNull<FactoryInfoRef>,
// }

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct ChangeMask: u64 {
        const PROPS = pw_sys::PW_FACTORY_CHANGE_MASK_PROPS as u64;
        const ALL = pw_sys::PW_FACTORY_CHANGE_MASK_ALL as u64;
    }
}

// impl Drop for FactoryInfo {
//     fn drop(&mut self) {
//         unsafe { pw_sys::pw_factory_info_free(self.as_raw_ptr()) }
//     }
// }

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

    //todo props

    //todo update
    //todo merge
}
