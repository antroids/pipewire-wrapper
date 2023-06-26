use std::ffi::{c_char, CStr};
use std::fmt::{Debug, Formatter};

use pipewire_proc_macro::RawWrapper;

use crate::spa::type_::pod::{
    BasicTypePod, PodError, PodResult, PodValueParser, ReadablePod, SizedPod,
};

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodStringRef {
    #[raw]
    raw: spa_sys::spa_pod_string,
}

impl SizedPod for PodStringRef {
    fn size_bytes(&self) -> usize {
        self.upcast().size_bytes()
    }
}

impl<'a> PodValueParser<*const c_char> for &'a PodStringRef {
    type To = &'a CStr;

    fn parse(size: u32, value: *const c_char) -> PodResult<Self::To> {
        unsafe {
            if *value.offset((size - 1) as isize) != 0 {
                Err(PodError::DataIsTooShort)
            } else {
                Ok(CStr::from_ptr(value))
            }
        }
    }
}

impl<'a> PodValueParser<*const u8> for &'a PodStringRef {
    type To = &'a CStr;

    fn parse(size: u32, value: *const u8) -> PodResult<Self::To> {
        Self::parse(size, value as *const c_char)
    }
}

impl<'a> ReadablePod for &'a PodStringRef {
    type Value = &'a CStr;

    fn value(&self) -> PodResult<Self::Value> {
        unsafe {
            Self::parse(
                self.upcast().content_size(),
                self.upcast().content_ptr() as *const c_char,
            )
        }
    }
}

impl BasicTypePod for PodStringRef {}

impl Debug for PodStringRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("PodStringRef")
                .field("pod", &self.upcast())
                .field("value", &self.value())
                .finish()
        }
    }
}
