/*
 * SPDX-License-Identifier: MIT
 */
use std::ffi::c_void;
use std::io::{Seek, Write};
use std::ptr::addr_of;

use pipewire_proc_macro::RawWrapper;

use crate::spa::pod::restricted::{PodHeader, PodRawValue};
use crate::spa::pod::{PodRef, PodResult, PodValue, SizedPod, WritePod, WriteValue};
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

    fn static_type() -> Type {
        Type::POINTER
    }
}

impl<'a> PodRawValue for &'a PodPointerRef {
    // Can PodRef be used here?
    type RawValue = spa_sys::spa_pod_pointer_body;

    fn raw_value_ptr(&self) -> *const Self::RawValue {
        &self.raw.body
    }

    fn parse_raw_value(ptr: *const Self::RawValue, _size: usize) -> PodResult<Self::Value> {
        unsafe { Ok((*ptr).value) }
    }
}

impl<'a> PodValue for &'a PodPointerRef {
    type Value = *const c_void;
    fn value(&self) -> PodResult<Self::Value> {
        Self::parse_raw_value(self.raw_value_ptr(), self.pod_header().size as usize)
    }
}

impl<'a> WritePod for &'a PodPointerRef {
    fn write_pod<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<()>
    where
        W: Write + Seek,
    {
        todo!()
    }
}

impl<'a> WriteValue for &'a PodPointerRef {
    fn write_raw_value<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<()>
    where
        W: Write + Seek,
    {
        todo!()
    }
}
