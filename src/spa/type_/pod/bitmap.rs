use std::fmt::{Debug, Formatter};
use std::slice;

use pipewire_proc_macro::RawWrapper;

use crate::spa::type_::pod::{BasicTypePod, PodResult, PodValueParser, ReadablePod, SizedPod};
use crate::spa::type_::Type;

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodBitmapRef {
    #[raw]
    raw: spa_sys::spa_pod_bitmap,
}

impl SizedPod for PodBitmapRef {
    fn pod_size(&self) -> usize {
        self.upcast().pod_size()
    }
}

impl<'a> PodValueParser<*const u8> for &'a PodBitmapRef {
    type To = &'a [u8];

    fn parse(size: u32, value: *const u8) -> PodResult<Self::To> {
        unsafe { Ok(slice::from_raw_parts(value, size as usize)) }
    }
}

impl<'a> ReadablePod for &'a PodBitmapRef {
    type Value = &'a [u8];

    fn value(&self) -> PodResult<Self::Value> {
        unsafe { Self::parse(self.upcast().size(), self.upcast().content_ptr()) }
    }
}

impl BasicTypePod for PodBitmapRef {
    fn static_type() -> Type {
        Type::BITMAP
    }
}

impl Debug for PodBitmapRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("PodBitmapRef")
                .field("pod", &self.upcast())
                .field("value", &self.value())
                .finish()
        }
    }
}
