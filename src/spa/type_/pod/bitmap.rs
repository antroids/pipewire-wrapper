use std::fmt::{Debug, Formatter};
use std::slice;

use spa_sys::spa_pod;

use pipewire_proc_macro::RawWrapper;

use crate::spa::type_::pod::restricted::{PodHeader, StaticTypePod};
use crate::spa::type_::pod::{BasicTypePod, PodResult, PodValueParser, ReadablePod, SizedPod};
use crate::spa::type_::Type;

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodBitmapRef {
    #[raw]
    raw: spa_sys::spa_pod_bitmap,
}

impl PodBitmapRef {
    fn content_size(&self) -> usize {
        self.raw.pod.size as usize
    }

    unsafe fn content_ptr(&self) -> *const u8 {
        (self as *const Self).offset(1).cast()
    }
}

impl StaticTypePod for PodBitmapRef {
    fn static_type() -> Type {
        Type::BITMAP
    }
}

impl PodHeader for PodBitmapRef {
    fn pod_header(&self) -> &spa_pod {
        &self.raw.pod
    }
}

impl<'a> PodValueParser<*const u8> for &'a PodBitmapRef {
    fn parse(
        content_size: usize,
        header_or_value: *const u8,
    ) -> PodResult<<Self as ReadablePod>::Value> {
        unsafe { Ok(slice::from_raw_parts(header_or_value, content_size)) }
    }
}

impl<'a> ReadablePod for &'a PodBitmapRef {
    type Value = &'a [u8];

    fn value(&self) -> PodResult<Self::Value> {
        unsafe { Self::parse(self.content_size(), self.content_ptr()) }
    }
}

impl Debug for PodBitmapRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("PodBitmapRef")
                .field("pod.type", &self.upcast().type_())
                .field("pod.size", &self.upcast().size())
                .field("value", &self.value())
                .finish()
        }
    }
}
