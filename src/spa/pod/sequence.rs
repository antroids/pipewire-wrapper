/*
 * SPDX-License-Identifier: MIT
 */
use std::io::{Seek, Write};
use std::ptr::addr_of;

use pipewire_proc_macro::RawWrapper;

use crate::spa::pod::control::PodControlRef;
use crate::spa::pod::iterator::PodIterator;
use crate::spa::pod::restricted::{PodHeader, PodRawValue, WritePod};
use crate::spa::pod::{BasicType, PodResult, PodValue, SizedPod};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

use super::restricted::{write_align_padding, write_header};

#[derive(RawWrapper)]
#[repr(transparent)]
struct PodSequenceBodyRef {
    #[raw]
    raw: spa_sys::spa_pod_sequence_body,
}

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct PodSequenceRef {
    #[raw]
    raw: spa_sys::spa_pod_sequence,
}

impl PodSequenceRef {
    fn body(&self) -> &PodSequenceBodyRef {
        unsafe { PodSequenceBodyRef::from_raw_ptr(addr_of!(self.raw.body)) }
    }
}

impl PodHeader for PodSequenceRef {
    fn pod_header(&self) -> &spa_sys::spa_pod {
        &self.raw.pod
    }

    fn static_type() -> Type {
        Type::SEQUENCE
    }
}

impl<'a> PodRawValue for &'a PodSequenceRef {
    type RawValue = spa_sys::spa_pod_sequence_body;

    fn raw_value_ptr(&self) -> *const Self::RawValue {
        &self.raw.body
    }

    fn parse_raw_value(ptr: *const Self::RawValue, size: usize) -> PodResult<Self::Value> {
        let first_element_ptr = unsafe { ptr.offset(1) as *const PodControlRef };
        Ok(PodIterator::new(first_element_ptr, size))
    }
}

impl<'a> PodValue for &'a PodSequenceRef {
    type Value = PodIterator<'a, PodControlRef>;
    fn value(&self) -> PodResult<Self::Value> {
        Self::parse_raw_value(self.raw_value_ptr(), self.pod_header().size as usize)
    }
}

impl<'a> WritePod for &'a PodSequenceRef {
    fn write_pod<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<()>
    where
        W: Write + Seek,
    {
        let iterator_content = unsafe { value.as_bytes() };
        write_header(
            buffer,
            iterator_content.len() as u32,
            PodSequenceRef::static_type(),
        )?;
        buffer.write_all(iterator_content)?;
        write_align_padding(buffer)
    }
}
