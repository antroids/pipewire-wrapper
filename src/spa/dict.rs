/*
 * SPDX-License-Identifier: MIT
 */
use core::slice;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::ptr::{null, null_mut, slice_from_raw_parts, NonNull};
use std::slice::Iter;

use bitflags::bitflags;
use spa_sys::{spa_dict, spa_dict_item};

use pipewire_proc_macro::{RawWrapper, Wrapper};

use crate::wrapper::{RawWrapper, Wrapper};

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct DictRef {
    #[raw]
    raw: spa_sys::spa_dict,
}

pub type DictRefIterator<'a> = std::slice::Iter<'a, DictItemRef>;

impl DictRef {
    pub fn flags(&self) -> Flags {
        Flags::from_bits_retain(self.raw.flags)
    }

    pub fn n_items(&self) -> u32 {
        self.raw.n_items
    }

    pub fn items(&self) -> &[DictItemRef] {
        unsafe {
            slice::from_raw_parts(
                self.raw.items as *const DictItemRef,
                self.raw.n_items as usize,
            )
        }
    }

    pub fn iter(&self) -> DictRefIterator {
        self.items().iter()
    }

    pub(crate) unsafe fn from_items(items: &[DictItemRef], flags: Flags) -> Self {
        Self {
            raw: spa_dict {
                flags: flags.bits(),
                n_items: items.len() as u32,
                items: items.as_ptr().cast(),
            },
        }
    }
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct DictItemRef {
    #[raw]
    raw: spa_sys::spa_dict_item,
}

impl DictItemRef {
    pub fn key(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.raw.key) }
    }

    pub fn value(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.raw.value) }
    }

    pub(crate) unsafe fn from_tuple<'a>(value: &'a (&'a CStr, &'a CStr)) -> Self
    where
        Self: 'a,
    {
        DictItemRef::from_raw(spa_dict_item {
            key: value.0.as_ptr(),
            value: value.1.as_ptr(),
        })
    }
}

impl Debug for DictItemRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DictItemRef")
            .field("key", &self.key())
            .field("value", &self.value())
            .finish()
    }
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct Flags: u32 {
        const SORTED = spa_sys::SPA_DICT_FLAG_SORTED;
    }
}

impl From<&DictRef> for HashMap<CString, CString> {
    fn from(value: &DictRef) -> Self {
        HashMap::from_iter(
            value
                .iter()
                .map(|p| (CString::from(p.key()), CString::from(p.value()))),
        )
    }
}

impl Debug for DictRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.iter().map(|item| (item.key(), item.value())))
            .finish()
    }
}
