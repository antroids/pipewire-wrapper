/*
 * SPDX-License-Identifier: MIT
 */
use std::fmt::{Debug, Formatter};
use std::io::{Seek, Write};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::addr_of;

use spa_sys::spa_pod;

use pipewire_wrapper_proc_macro::RawWrapper;

use crate::spa::pod::iterator::PodValueIterator;
use crate::spa::pod::restricted::{PodHeader, PodRawValue};
use crate::spa::pod::{
    BasicTypePod, PodBoolRef, PodIntRef, PodRef, PodResult, PodValue, SizedPod, WritePod,
    WriteValue,
};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

use super::iterator::AllocatedPodValueIterator;
use super::restricted::{write_align_padding, write_header};
use super::FromValue;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct PodArrayBodyRef {
    #[raw]
    raw: spa_sys::spa_pod_array_body,
}

impl PodArrayBodyRef {
    unsafe fn content_ptr(&self) -> *const u8 {
        (self.as_raw_ptr() as *const u8).add(size_of::<PodArrayBodyRef>())
    }

    fn child(&self) -> &PodRef {
        unsafe { PodRef::from_raw_ptr(addr_of!(self.raw.child)) }
    }
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodArrayRef<T: PodRawValue = PodRef> {
    #[raw]
    raw: spa_sys::spa_pod_array,
    phantom: PhantomData<T>,
}

impl<T> PodHeader for PodArrayRef<T>
where
    T: PodRawValue,
    T: BasicTypePod,
{
    fn pod_header(&self) -> &spa_pod {
        &self.raw.pod
    }

    fn static_type() -> Type {
        Type::ARRAY
    }
}

impl<'a, T> PodRawValue for &'a PodArrayRef<T>
where
    T: PodRawValue,
    T: BasicTypePod,
{
    type RawValue = spa_sys::spa_pod_array_body;

    fn raw_value_ptr(&self) -> *const Self::RawValue {
        &self.raw.body
    }

    fn parse_raw_value(ptr: *const Self::RawValue, size: usize) -> PodResult<Self::Value> {
        let body = unsafe { PodArrayBodyRef::from_raw_ptr(ptr) };
        let size = size - size_of::<Self::RawValue>();
        let first_element_ptr = unsafe { body.content_ptr() };
        Ok(PodValueIterator::new(
            first_element_ptr.cast(),
            size,
            body.child().size() as usize,
        ))
    }
}

impl<'a, T> PodValue for &'a PodArrayRef<T>
where
    T: PodRawValue,
    T: BasicTypePod,
{
    type Value = PodValueIterator<'a, T>;
    fn value(&self) -> PodResult<Self::Value> {
        Self::parse_raw_value(self.raw_value_ptr(), self.pod_header().size as usize)
    }
}

impl<'a, T> WritePod for &'a PodArrayRef<T>
where
    T: PodRawValue,
    T: BasicTypePod,
{
    fn write_pod<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<()>
    where
        W: Write + Seek,
    {
        let iterator_content = unsafe { value.as_bytes() };
        write_header(
            buffer,
            (iterator_content.len() + size_of::<spa_sys::spa_pod_array_body>()) as u32,
            PodArrayRef::<T>::static_type(),
        )?;
        write_header(buffer, value.element_size() as u32, T::static_type())?;
        buffer.write_all(iterator_content)?;
        write_align_padding(buffer)
    }
}

impl<'a, T> WriteValue for &'a PodArrayRef<T>
where
    T: PodRawValue,
    T: BasicTypePod,
{
    fn write_raw_value<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<()>
    where
        W: std::io::Write + std::io::Seek,
    {
        let iterator_content = unsafe { value.as_bytes() };
        buffer.write_all(iterator_content)?;
        Ok(())
    }
}

impl<T> Debug for PodArrayRef<T>
where
    T: PodRawValue,
    T: BasicTypePod,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodArrayRef")
            .field("pod.type", &self.pod_type())
            .field("pod.size", &self.pod_size())
            .field("body", &self.body())
            .field("value", &self.value().map(|i| i.collect::<Vec<_>>()))
            .finish()
    }
}

impl<T> PodArrayRef<T>
where
    T: PodRawValue,
    T: BasicTypePod,
{
    fn body(&self) -> &PodArrayBodyRef {
        unsafe { PodArrayBodyRef::from_raw_ptr(addr_of!(self.raw.body)) }
    }

    fn body_size(&self) -> usize {
        self.raw.pod.size as usize
    }

    fn elements(&self) -> u32 {
        ((self.body_size() - size_of::<PodArrayBodyRef>()) / self.raw.body.child.size as usize)
            as u32
    }
}

#[test]
fn test_array() {
    let allocated_iter = AllocatedPodValueIterator::<PodIntRef>::new(vec![1, 2, 3]);
    let allocated_array = PodArrayRef::from_value(&allocated_iter.iter()).unwrap();
    let array = allocated_array.as_pod();

    let mut array_value = array.value().unwrap();
    assert_eq!(array.raw.pod.type_, Type::ARRAY.raw);
    assert_eq!(array.raw.pod.size, 20);
    assert_eq!(array_value.next(), Some(1i32));
    assert_eq!(array_value.next(), Some(2i32));
    assert_eq!(array_value.next(), Some(3i32));
    assert_eq!(array_value.next(), None);

    assert_eq!(array.elements(), 3);

    let allocated_iter = AllocatedPodValueIterator::<PodBoolRef>::new(vec![]);
    let allocated_array = PodArrayRef::from_value(&allocated_iter.iter()).unwrap();
    let array = allocated_array.as_pod();

    let mut array_value = array.value().unwrap();
    assert_eq!(array.raw.pod.type_, Type::ARRAY.raw);
    assert_eq!(array.raw.pod.size, 8);
    assert_eq!(array_value.next(), None);

    assert_eq!(array.elements(), 0);
}
