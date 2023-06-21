use std::ffi::CStr;
use std::fmt::{Debug, Display, Formatter};
use std::os::raw::c_char;
use std::ptr::{addr_of, addr_of_mut, NonNull, null};
use std::slice;

use bitflags::bitflags;
use pipewire_proc_macro::{RawWrapper, Wrapper};
use pw_sys::FILE;

use crate::{i32_as_result, i32_as_void_result};
use crate::spa::dict::{DictItemRef, DictRef, DictRefIterator};
use crate::wrapper::RawWrapper;

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PropertiesRef {
    #[raw]
    raw: pw_sys::pw_properties,
}

#[derive(Wrapper, Debug)]
pub struct Properties {
    #[raw_wrapper]
    ref_: NonNull<PropertiesRef>,
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct Flags: u32 {
        const NL = pw_sys::PW_PROPERTIES_FLAG_NL;
        // const RECURSE = pw_sys::PW_PROPERTIES_FLAG_RECURSE;
        // const ENCLOSE = pw_sys::PW_PROPERTIES_FLAG_ENCLOSE;
        // const ARRAY = pw_sys::PW_PROPERTIES_FLAG_ARRAY;
        // const COLORS = pw_sys::PW_PROPERTIES_FLAG_COLORS;
    }
}

impl PropertiesRef {
    pub fn flags(&self) -> Flags {
        Flags::from_bits_retain(self.raw.flags)
    }

    pub fn n_items(&self) -> u32 {
        self.raw.dict.n_items
    }

    pub fn items(&self) -> &[DictItemRef] {
        unsafe {
            slice::from_raw_parts(
                self.raw.dict.items as *const DictItemRef,
                self.raw.dict.n_items as usize,
            )
        }
    }

    pub fn dict(&self) -> &DictRef {
        unsafe { DictRef::from_raw_ptr(&self.raw.dict) }
    }

    pub fn copy(&self) -> NonNull<PropertiesRef> {
        let ptr = unsafe { pw_sys::pw_properties_copy(self.as_raw_ptr()) };
        NonNull::new(ptr as *mut PropertiesRef).unwrap()
    }

    pub fn update_keys(&self, source: &DictRef, keys_to_update: &Vec<&CStr>) -> i32 {
        let mut keys: Vec<*const c_char> =
            keys_to_update.clone().iter().map(|k| k.as_ptr()).collect();
        keys.push(null() as *const c_char);
        unsafe {
            pw_sys::pw_properties_update_keys(self.as_raw_ptr(), source.as_raw_ptr(), keys.as_ptr())
        }
    }

    pub fn update_ignore(&self, source: &DictRef, ignore_keys: &Vec<&CStr>) -> i32 {
        let mut keys: Vec<*const c_char> = ignore_keys.clone().iter().map(|k| k.as_ptr()).collect();
        keys.push(null() as *const c_char);
        unsafe {
            pw_sys::pw_properties_update_ignore(
                self.as_raw_ptr(),
                source.as_raw_ptr(),
                keys.as_ptr(),
            )
        }
    }

    pub fn update(&self, source: &DictRef) -> i32 {
        unsafe { pw_sys::pw_properties_update(self.as_raw_ptr(), source.as_raw_ptr()) }
    }

    pub fn update_string(&self, string: &CStr) -> i32 {
        unsafe {
            pw_sys::pw_properties_update_string(
                self.as_raw_ptr(),
                string.as_ptr(),
                string.to_bytes().len(),
            )
        }
    }

    pub fn add(&self, other: &DictRef) -> i32 {
        unsafe { pw_sys::pw_properties_add(self.as_raw_ptr(), other.as_raw_ptr()) }
    }

    pub fn add_keys(&self, other: &DictRef, keys_to_add: &Vec<&CStr>) -> i32 {
        let mut keys: Vec<*const c_char> = keys_to_add.clone().iter().map(|k| k.as_ptr()).collect();
        keys.push(null() as *const c_char);
        unsafe {
            pw_sys::pw_properties_add_keys(self.as_raw_ptr(), other.as_raw_ptr(), keys.as_ptr())
        }
    }

    pub fn clear(&self) {
        unsafe { pw_sys::pw_properties_clear(self.as_raw_ptr()) }
    }

    pub fn set(&self, key: &CStr, value: &CStr) -> i32 {
        unsafe { pw_sys::pw_properties_set(self.as_raw_ptr(), key.as_ptr(), value.as_ptr()) }
    }

    pub fn remove(&self, key: &CStr) -> i32 {
        unsafe { pw_sys::pw_properties_set(self.as_raw_ptr(), key.as_ptr(), null()) }
    }

    pub fn get(&self, key: &CStr) -> Option<&CStr> {
        unsafe {
            let value = pw_sys::pw_properties_get(self.as_raw_ptr(), key.as_ptr());
            if let Some(value) = value.as_ref() {
                Some(&CStr::from_ptr(value))
            } else {
                None
            }
        }
    }

    pub fn fetch_u32(&self, key: &CStr) -> crate::Result<u32> {
        let mut value = 0u32;
        let result = unsafe {
            pw_sys::pw_properties_fetch_uint32(self.as_raw_ptr(), key.as_ptr(), addr_of_mut!(value))
        };
        i32_as_result(result, value)
    }

    pub fn fetch_i32(&self, key: &CStr) -> crate::Result<i32> {
        let mut value = 0i32;
        let result = unsafe {
            pw_sys::pw_properties_fetch_int32(self.as_raw_ptr(), key.as_ptr(), addr_of_mut!(value))
        };
        i32_as_result(result, value)
    }

    pub fn fetch_u64(&self, key: &CStr, value: &u64) -> crate::Result<u64> {
        let mut value = 0u64;
        let result = unsafe {
            pw_sys::pw_properties_fetch_uint64(self.as_raw_ptr(), key.as_ptr(), addr_of_mut!(value))
        };
        i32_as_result(result, value)
    }

    pub fn fetch_i64(&self, key: &CStr) -> crate::Result<i64> {
        let mut value = 0i64;
        let result = unsafe {
            pw_sys::pw_properties_fetch_int64(self.as_raw_ptr(), key.as_ptr(), addr_of_mut!(value))
        };
        i32_as_result(result, value)
    }

    pub fn fetch_bool(&self, key: &CStr) -> crate::Result<bool> {
        let mut value = false;
        let result = unsafe {
            pw_sys::pw_properties_fetch_bool(self.as_raw_ptr(), key.as_ptr(), addr_of_mut!(value))
        };
        i32_as_result(result, value)
    }

    pub fn get_u32(&self, key: &CStr, default: u32) -> u32 {
        let mut value = default;
        let result = unsafe {
            pw_sys::pw_properties_fetch_uint32(self.as_raw_ptr(), key.as_ptr(), addr_of_mut!(value))
        };
        value
    }

    pub fn get_i32(&self, key: &CStr, default: i32) -> i32 {
        let mut value = default;
        let result = unsafe {
            pw_sys::pw_properties_fetch_int32(self.as_raw_ptr(), key.as_ptr(), addr_of_mut!(value))
        };
        value
    }

    pub fn get_u64(&self, key: &CStr, default: u64) -> u64 {
        let mut value = default;
        let result = unsafe {
            pw_sys::pw_properties_fetch_uint64(self.as_raw_ptr(), key.as_ptr(), addr_of_mut!(value))
        };
        value
    }

    pub fn get_i64(&self, key: &CStr, default: i64) -> i64 {
        let mut value = default;
        let result = unsafe {
            pw_sys::pw_properties_fetch_int64(self.as_raw_ptr(), key.as_ptr(), addr_of_mut!(value))
        };
        value
    }

    pub fn get_bool(&self, key: &CStr, default: bool) -> bool {
        let mut value = default;
        let result = unsafe {
            pw_sys::pw_properties_fetch_bool(self.as_raw_ptr(), key.as_ptr(), addr_of_mut!(value))
        };
        value
    }

    pub fn serialize(&self, file: &FILE) -> i32 {
        unsafe {
            pw_sys::pw_properties_serialize_dict(
                file as *const FILE as *mut FILE,
                addr_of!(self.raw.dict),
                self.raw.flags,
            )
        }
    }

    pub fn iter(&self) -> PropertiesIterator {
        PropertiesIterator {
            dict_iterator: self.dict().iter(),
        }
    }
}

impl Properties {
    pub fn new(values: &Vec<(&CStr, &CStr)>) -> Self {
        values.into()
    }
}

impl From<DictRef> for Properties {
    fn from(value: DictRef) -> Self {
        let ptr = unsafe { pw_sys::pw_properties_new_dict(value.as_raw_ptr()) };
        Self {
            ref_: NonNull::new(ptr as *mut PropertiesRef).unwrap(),
        }
    }
}

impl From<&Vec<(&CStr, &CStr)>> for Properties {
    fn from(value: &Vec<(&CStr, &CStr)>) -> Self {
        let dict: DictRef = value.into();
        dict.into()
    }
}

impl From<&CStr> for Properties {
    fn from(value: &CStr) -> Self {
        let ptr = unsafe { pw_sys::pw_properties_new_string(value.as_ptr()) };
        Self {
            ref_: NonNull::new(ptr as *mut PropertiesRef).unwrap(),
        }
    }
}

impl Clone for Properties {
    fn clone(&self) -> Self {
        Self { ref_: self.copy() }
    }
}

impl Drop for Properties {
    fn drop(&mut self) {
        unsafe { pw_sys::pw_properties_free(self.as_raw_ptr()) }
    }
}

impl Default for Properties {
    fn default() -> Self {
        (&Vec::<(&CStr, &CStr)>::default()).into()
    }
}

pub struct PropertiesIterator<'a> {
    dict_iterator: DictRefIterator<'a>,
}

impl<'a> Iterator for PropertiesIterator<'a> {
    type Item = (&'a CStr, &'a CStr);

    fn next(&mut self) -> Option<Self::Item> {
        self.dict_iterator
            .next()
            .map(|item| (item.key(), item.value()))
    }
}

impl Debug for PropertiesRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}
