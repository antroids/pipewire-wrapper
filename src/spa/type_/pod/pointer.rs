use std::ffi::c_void;
use std::io::{Seek, Write};
use std::ptr::addr_of;

use pipewire_proc_macro::RawWrapper;

use crate::spa::type_::pod::restricted::{PodHeader, StaticTypePod};
use crate::spa::type_::pod::{PodRef, PodResult, ReadablePod, SizedPod, WritablePod};
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

impl PodHeader for PodPointerRef {
    fn pod_header(&self) -> &spa_sys::spa_pod {
        &self.raw.pod
    }
}

impl StaticTypePod for PodPointerRef {
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

impl<'a> WritablePod for &'a PodPointerRef {
    fn write_pod<W>(buffer: &mut W, value: &<Self as ReadablePod>::Value) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        todo!()
    }

    fn write_raw_value<W>(buffer: &mut W, value: &<Self as ReadablePod>::Value) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        todo!()
    }
}
