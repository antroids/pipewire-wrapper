use std::fmt::{Debug, Formatter};
use std::slice;

use spa_sys::spa_pod;

use pipewire_proc_macro::RawWrapper;

use crate::spa::type_::pod::restricted::{PodHeader, StaticTypePod};
use crate::spa::type_::pod::{BasicTypePod, PodResult, PodValueParser, ReadablePod, SizedPod};
use crate::spa::type_::Type;

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodBytesRef {
    #[raw]
    raw: spa_sys::spa_pod_bytes,
}

impl PodBytesRef {
    fn content_size(&self) -> usize {
        self.raw.pod.size as usize
    }

    unsafe fn content_ptr(&self) -> *const u8 {
        (self as *const Self).offset(1).cast()
    }
}

impl PodHeader for PodBytesRef {
    fn pod_header(&self) -> &spa_pod {
        &self.raw.pod
    }
}

impl StaticTypePod for PodBytesRef {
    fn static_type() -> Type {
        Type::BYTES
    }
}

impl<'a> PodValueParser<*const u8> for &'a PodBytesRef {
    fn parse(
        content_size: usize,
        header_or_value: *const u8,
    ) -> PodResult<<Self as ReadablePod>::Value> {
        unsafe { Ok(slice::from_raw_parts(header_or_value, content_size)) }
    }
}

impl<'a> ReadablePod for &'a PodBytesRef {
    type Value = &'a [u8];

    fn value(&self) -> PodResult<Self::Value> {
        unsafe { Self::parse(self.content_size(), self.content_ptr()) }
    }
}

impl Debug for PodBytesRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("PodBytesRef")
                .field("pod.type", &self.upcast().type_())
                .field("pod.size", &self.upcast().size())
                .field("value", &self.value())
                .finish()
        }
    }
}
