use std::ffi::c_void;
use std::ptr::addr_of;

use pipewire_proc_macro::RawWrapper;

use crate::spa::type_::pod::restricted::PodSubtype;
use crate::spa::type_::pod::{Pod, PodRef, PodResult, ReadablePod};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
struct PodPointerBodyRef {
    #[raw]
    raw: spa_sys::spa_pod_pointer_body,
}

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct PodPointerRef {
    #[raw]
    raw: spa_sys::spa_pod_pointer,
}

impl PodPointerBodyRef {
    fn pointer_type(&self) -> Type {
        Type::from_raw(self.raw.type_)
    }
}

impl PodPointerRef {
    fn body(&self) -> &PodPointerBodyRef {
        unsafe { PodPointerBodyRef::from_raw_ptr(addr_of!(self.raw.body)) }
    }

    fn pointer_type(&self) -> Type {
        self.body().pointer_type()
    }
}

impl Pod for PodPointerRef {
    fn pod_size(&self) -> usize {
        self.upcast().pod_size()
    }
}

impl PodSubtype for PodPointerRef {
    fn static_type() -> Type {
        Type::POINTER
    }
}

impl<'a> ReadablePod for &'a PodPointerRef {
    type Value = *const c_void; // Can PodRef be used here?

    fn value(&self) -> PodResult<Self::Value> {
        Ok(self.raw.body.value)
    }
}
