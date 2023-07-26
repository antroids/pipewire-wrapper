/*
 * SPDX-License-Identifier: MIT
 */
use std::fmt::{Debug, Formatter};
use std::io::{Seek, Write};

use pipewire_proc_macro::RawWrapper;

use crate::spa::pod::iterator::PodIterator;
use crate::spa::pod::restricted::{PodHeader, PodRawValue, StaticTypePod};
use crate::spa::pod::{BasicTypePod, PodRef, PodResult, PodValue, SizedPod, WritePod, WriteValue};
use crate::spa::type_::Type;

use super::restricted::{write_align_padding, write_count_size, write_header};

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodStructRef {
    #[raw]
    raw: spa_sys::spa_pod_struct,
}

impl StaticTypePod for PodStructRef {
    fn static_type() -> Type {
        Type::STRUCT
    }
}

impl PodHeader for PodStructRef {
    fn pod_header(&self) -> &spa_sys::spa_pod {
        &self.raw.pod
    }
}

impl<'a> PodRawValue for &'a PodStructRef {
    type RawValue = spa_sys::spa_pod;

    fn raw_value_ptr(&self) -> *const Self::RawValue {
        unsafe { (&self.raw.pod as *const spa_sys::spa_pod).offset(1) }
    }

    fn parse_raw_value(ptr: *const Self::RawValue, size: usize) -> PodResult<Self::Value> {
        Ok(PodIterator::new(ptr.cast(), size))
    }
}

impl<'a> PodValue for &'a PodStructRef {
    type Value = PodIterator<'a, PodRef>;
    fn value(&self) -> PodResult<Self::Value> {
        Self::parse_raw_value(self.raw_value_ptr(), self.pod_header().size as usize)
    }
}

impl<'a> WritePod for &'a PodStructRef {
    fn write_pod<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<()>
    where
        W: Write + Seek,
    {
        let iterator_content = unsafe { value.as_bytes() };
        write_header(
            buffer,
            iterator_content.len() as u32,
            PodStructRef::static_type(),
        )?;
        buffer.write_all(iterator_content)?;
        write_align_padding(buffer)
    }
}

impl<'a> WriteValue for &'a PodStructRef {
    fn write_raw_value<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<()>
    where
        W: Write + Seek,
    {
        let iterator_content = unsafe { value.as_bytes() };
        buffer.write_all(iterator_content)?;
        Ok(())
    }
}

impl Debug for PodStructRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodStructRef")
            .field("pod.type", &self.pod_type())
            .field("pod.size", &self.pod_size())
            .field(
                "value",
                &self
                    .value()
                    .map(|v| v.map(|p| p.downcast()).collect::<Vec<_>>()),
            )
            .finish()
    }
}
