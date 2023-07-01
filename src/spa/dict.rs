use core::slice;
use std::ffi::CStr;
use std::fmt::{Debug, Formatter};
use std::ptr::{NonNull, slice_from_raw_parts};
use std::slice::Iter;

use bitflags::bitflags;

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
}

impl Debug for DictItemRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DictItemRef")
            .field("key", &self.key())
            .field("value", &self.value())
            .finish()
    }
}

impl From<(&CStr, &CStr)> for DictItemRef {
    fn from(key_value: (&CStr, &CStr)) -> Self {
        Self::from_raw(spa_sys::spa_dict_item {
            key: key_value.0.as_ptr(),
            value: key_value.1.as_ptr(),
        })
    }
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct Flags: u32 {
        const SORTED = spa_sys::SPA_DICT_FLAG_SORTED;
    }
}

impl From<&Vec<DictItemRef>> for DictRef {
    fn from(value: &Vec<DictItemRef>) -> Self {
        let flags = Flags::empty().0.bits();
        let n_items = value.len() as u32;
        let items = value.as_ptr() as *const spa_sys::spa_dict_item;

        Self::from_raw(spa_sys::spa_dict {
            flags,
            n_items,
            items,
        })
    }
}

impl From<&Vec<(&CStr, &CStr)>> for DictRef {
    fn from(value: &Vec<(&CStr, &CStr)>) -> Self {
        let items: Vec<DictItemRef> = value.iter().map(|v| DictItemRef::from(*v)).collect();
        DictRef::from(&items)
    }
}

impl Debug for DictRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.iter().map(|item| (item.key(), item.value())))
            .finish()
    }
}
