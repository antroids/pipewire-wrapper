use std::fmt::{Debug, Formatter};
use std::slice;

use pipewire_proc_macro::RawWrapper;

use crate::spa::type_::pod::{Pod, PodResult, PodSubtype, PodValueParser, ReadablePod};
use crate::spa::type_::Type;

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodBytesRef {
    #[raw]
    raw: spa_sys::spa_pod_bytes,
}

impl Pod for PodBytesRef {
    fn pod_size(&self) -> usize {
        self.upcast().pod_size()
    }
}

impl<'a> PodValueParser<*const u8> for &'a PodBytesRef {
    type To = &'a [u8];

    fn parse(size: u32, value: *const u8) -> PodResult<Self::To> {
        unsafe { Ok(slice::from_raw_parts(value, size as usize)) }
    }
}

impl<'a> ReadablePod for &'a PodBytesRef {
    type Value = &'a [u8];

    fn value(&self) -> PodResult<Self::Value> {
        unsafe { Self::parse(self.upcast().size(), self.upcast().content_ptr()) }
    }
}

impl PodSubtype for PodBytesRef {
    fn static_type() -> Type {
        Type::BYTES
    }
}

impl Debug for PodBytesRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("PodBytesRef")
                .field("pod", &self.upcast())
                .field("value", &self.value())
                .finish()
        }
    }
}
