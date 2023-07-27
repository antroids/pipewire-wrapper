/*
 * SPDX-License-Identifier: MIT
 */
use std::collections::HashMap;
use std::ffi::CString;

use bitflags::{bitflags, Flags};

use pipewire_wrapper_proc_macro::RawWrapper;

use crate::spa::dict::DictRef;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct ClientInfoRef {
    #[raw]
    raw: pw_sys::pw_client_info,
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct ChangeMask: u64 {
        const PROPS = pw_sys::PW_CLIENT_CHANGE_MASK_PROPS as u64;
        const ALL = pw_sys::PW_CLIENT_CHANGE_MASK_ALL as u64;
    }
}

impl ClientInfoRef {
    pub fn id(&self) -> u32 {
        self.raw.id
    }

    pub fn change_mask(&self) -> ChangeMask {
        ChangeMask::from_bits_retain(self.raw.change_mask)
    }

    pub fn props(&self) -> &DictRef {
        unsafe { DictRef::from_raw_ptr(self.raw.props) }
    }
}

#[derive(Clone, Debug)]
pub struct ClientInfo {
    id: u32,
    change_mask: ChangeMask,
    props: HashMap<CString, CString>,
}

impl ClientInfo {
    pub fn from_ref(ref_: &ClientInfoRef) -> Self {
        Self {
            id: ref_.id(),
            change_mask: ref_.change_mask(),
            props: ref_.props().into(),
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
}

impl From<&ClientInfoRef> for ClientInfo {
    fn from(value: &ClientInfoRef) -> Self {
        ClientInfo::from_ref(value)
    }
}
