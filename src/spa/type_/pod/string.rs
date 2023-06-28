use std::ffi::{c_char, CStr};
use std::fmt::{Debug, Formatter};

use pipewire_proc_macro::RawWrapper;

use crate::spa::type_::pod::{Pod, PodError, PodResult, PodSubtype, PodValueParser, ReadablePod};
use crate::spa::type_::Type;

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodStringRef {
    #[raw]
    raw: spa_sys::spa_pod_string,
}

impl Pod for PodStringRef {
    fn pod_size(&self) -> usize {
        self.upcast().pod_size()
    }
}

impl<'a> PodValueParser<*const c_char> for &'a PodStringRef {
    fn parse(size: u32, value: *const c_char) -> PodResult<Self::Value> {
        unsafe {
            if *value.offset((size - 1) as isize) != 0 {
                Err(PodError::StringIsNotNullTerminated)
            } else {
                Ok(CStr::from_ptr(value))
            }
        }
    }
}

impl<'a> PodValueParser<*const u8> for &'a PodStringRef {
    fn parse(size: u32, value: *const u8) -> PodResult<Self::Value> {
        Self::parse(size, value as *const c_char)
    }
}

impl<'a> ReadablePod for &'a PodStringRef {
    type Value = &'a CStr;

    fn value(&self) -> PodResult<Self::Value> {
        unsafe {
            Self::parse(
                self.raw.pod.size,
                self.upcast().content_ptr() as *const c_char,
            )
        }
    }
}

impl PodSubtype for PodStringRef {
    fn static_type() -> Type {
        Type::STRING
    }
}

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
