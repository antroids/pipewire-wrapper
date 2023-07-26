/*
 * SPDX-License-Identifier: MIT
 */
use std::collections::HashMap;
use std::ffi::CString;
use std::fmt::{Debug, Formatter};
use std::slice;

use bitflags::{bitflags, Flags};

use pipewire_proc_macro::RawWrapper;

use crate::spa::dict::DictRef;
use crate::spa::param::{ParamInfo, ParamInfoRef};
use crate::wrapper::RawWrapper;

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct DeviceInfoRef {
    #[raw]
    raw: pw_sys::pw_device_info,
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct ChangeMask: u64 {
        const PROPS = pw_sys::PW_DEVICE_CHANGE_MASK_PROPS as u64;
        const PARAMS = pw_sys::PW_DEVICE_CHANGE_MASK_PARAMS as u64;
        const ALL = pw_sys::PW_DEVICE_CHANGE_MASK_ALL as u64;
    }
}

impl DeviceInfoRef {
    pub fn id(&self) -> u32 {
        self.raw.id
    }

    pub fn change_mask(&self) -> ChangeMask {
        ChangeMask::from_bits_retain(self.raw.change_mask)
    }

    pub fn props(&self) -> &DictRef {
        unsafe { DictRef::from_raw_ptr(self.raw.props) }
    }

    pub fn params(&self) -> &[ParamInfoRef] {
        unsafe {
            slice::from_raw_parts(
                self.raw.params as *mut ParamInfoRef,
                self.raw.n_params as usize,
            )
        }
    }
}

impl Debug for DeviceInfoRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeviceInfoRef")
            .field("id", &self.id())
            .field("change_mask", &self.change_mask())
            .field("props", &self.props())
            .field("params", &self.params())
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct DeviceInfo {
    id: u32,
    change_mask: ChangeMask,
    props: HashMap<CString, CString>,
    params: Vec<ParamInfo>,
}

impl DeviceInfo {
    pub fn from_ref(ref_: &DeviceInfoRef) -> Self {
        Self {
            id: ref_.id(),
            change_mask: ref_.change_mask(),
            props: ref_.props().into(),
            params: ref_.params().iter().map(ParamInfo::from_ref).collect(),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn change_mask(&self) -> ChangeMask {
        self.change_mask
    }
    pub fn props(&self) -> &HashMap<CString, CString> {
        &self.props
    }
    pub fn params(&self) -> &Vec<ParamInfo> {
        &self.params
    }
}

impl From<&DeviceInfoRef> for DeviceInfo {
    fn from(value: &DeviceInfoRef) -> Self {
        DeviceInfo::from_ref(value)
    }
}
