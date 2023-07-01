use std::ffi::{c_char, CStr};
use std::fmt::{Debug, Formatter};

use pipewire_proc_macro::RawWrapper;

use crate::spa::type_::pod::restricted::{PodHeader, StaticTypePod};
use crate::spa::type_::pod::{
    BasicTypePod, PodError, PodResult, PodValueParser, ReadablePod, SizedPod,
};
use crate::spa::type_::Type;

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodStringRef {
    #[raw]
    raw: spa_sys::spa_pod_string,
}

impl PodHeader for PodStringRef {
    fn pod_header(&self) -> &spa_sys::spa_pod {
        &self.raw.pod
    }
}

impl StaticTypePod for PodStringRef {
    fn static_type() -> Type {
        Type::STRING
    }
}

impl PodStringRef {
    fn content_size(&self) -> usize {
        self.raw.pod.size as usize
    }

    unsafe fn content_ptr(&self) -> *const c_char {
        (self as *const Self).offset(1).cast()
    }
}

impl<'a> PodValueParser<*const c_char> for &'a PodStringRef {
    fn parse(
        content_size: usize,
        header_or_value: *const c_char,
    ) -> PodResult<<Self as ReadablePod>::Value> {
        unsafe {
            if *header_or_value.offset((content_size - 1) as isize) != 0 {
                Err(PodError::StringIsNotNullTerminated)
            } else {
                Ok(CStr::from_ptr(header_or_value))
            }
        }
    }
}

impl<'a> PodValueParser<*const u8> for &'a PodStringRef {
    fn parse(
        content_size: usize,
        header_or_value: *const u8,
    ) -> PodResult<<Self as ReadablePod>::Value> {
        Self::parse(content_size, header_or_value as *const c_char)
    }
}

impl<'a> ReadablePod for &'a PodStringRef {
    type Value = &'a CStr;

    fn value(&self) -> PodResult<Self::Value> {
        unsafe { Self::parse(self.content_size(), self.content_ptr()) }
    }
}

impl Debug for PodStringRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("PodStringRef")
                .field("pod.type", &self.upcast().type_())
                .field("pod.size", &self.upcast().size())
                .field("value", &self.value())
                .finish()
        }
    }
}
