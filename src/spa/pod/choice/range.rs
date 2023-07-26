/*
 * SPDX-License-Identifier: MIT
 */
use std::fmt::{Debug, Formatter};
use std::io::{Seek, SeekFrom, Write};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::addr_of;

use spa_sys::spa_pod;

use pipewire_proc_macro::RawWrapper;

use crate::spa::pod::choice::{ChoiceType, PodChoiceBodyRef, PodChoiceRef};
use crate::spa::pod::iterator::PodValueIterator;
use crate::spa::pod::pod_buf::AllocatedData;
use crate::spa::pod::restricted::{
    write_count_size, write_header, PodHeader, PodRawValue, PrimitiveValue, StaticTypePod,
};
use crate::spa::pod::{
    BasicTypePod, PodError, PodRef, PodResult, PodValue, SizedPod, Upcast, WritePod, WriteValue,
};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PodRangeValue<T> {
    default: T,
    min: T,
    max: T,
}

impl<T> PodRangeValue<T> {
    pub fn default(&self) -> &T {
        &self.default
    }
    pub fn min(&self) -> &T {
        &self.min
    }
    pub fn max(&self) -> &T {
        &self.max
    }
    pub fn new(default: T, min: T, max: T) -> Self {
        Self { default, min, max }
    }

    pub fn to_alloc_pod<P>(&self) -> PodResult<AllocatedData<PodRangeRef<P>>>
    where
        PodRangeRef<P>: WritePod,
        PodRangeRef<P>: PodRawValue<Value = Self>,
        PodRangeRef<P>: PrimitiveValue,
    {
        AllocatedData::from_value(self)
    }
}

impl<T> PodRangeValue<T>
where
    T: PodRawValue,
    T: BasicTypePod,
    T: WriteValue,
    T: WritePod,
{
}

impl<T: Clone> PodRangeValue<T> {
    pub fn from_default(default: T) -> Self {
        Self {
            default: default.clone(),
            min: default.clone(),
            max: default,
        }
    }
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodRangeRef<T> {
    #[raw]
    raw: spa_sys::spa_pod_choice,
    phantom: PhantomData<T>,
}

impl<T: PodRawValue> PodRangeRef<T> {
    pub fn choice(&self) -> &PodChoiceRef<T> {
        unsafe { PodChoiceRef::from_raw_ptr(addr_of!(self.raw)) }
    }
}

impl<'a, T: PodRawValue> From<&'a PodRangeRef<T>> for &'a PodChoiceRef<T> {
    fn from(value: &'a PodRangeRef<T>) -> Self {
        value.choice()
    }
}

impl<T> StaticTypePod for PodRangeRef<T>
where
    T: PodRawValue,
    T: StaticTypePod,
{
    fn static_type() -> Type {
        PodChoiceRef::<T>::static_type()
    }
}

impl<T> PodHeader for PodRangeRef<T>
where
    T: PodRawValue,
    T: StaticTypePod,
{
    fn pod_header(&self) -> &spa_pod {
        &self.raw.pod
    }
}

impl<T> PodRawValue for PodRangeRef<T>
where
    T: PodRawValue,
    T: StaticTypePod,
{
    type RawValue = spa_sys::spa_pod_choice_body;

    fn raw_value_ptr(&self) -> *const Self::RawValue {
        &self.raw.body
    }

    fn parse_raw_value(ptr: *const Self::RawValue, size: usize) -> PodResult<Self::Value> {
        let body = unsafe { PodChoiceBodyRef::from_raw_ptr(ptr) };
        if body.type_() == ChoiceType::RANGE {
            if T::static_type() == body.child().type_() || T::static_type() == Type::POD {
                let content_size = size - size_of::<Self::RawValue>();
                let element_size = body.child().size() as usize;
                let mut iter: PodValueIterator<T> = PodValueIterator::new(
                    unsafe { body.content_ptr().cast() },
                    content_size,
                    element_size,
                );
                let default = iter.next().unwrap();
                let min = iter.next().unwrap();
                let max = iter.next().unwrap();
                if iter.next().is_some() {
                    Err(PodError::UnexpectedChoiceElement)
                } else {
                    Ok(PodRangeValue { default, min, max })
                }
            } else {
                Err(PodError::WrongPodTypeToCast(
                    T::static_type(),
                    body.child().type_(),
                ))
            }
        } else {
            Err(PodError::UnexpectedChoiceType(
                ChoiceType::RANGE,
                body.type_(),
            ))
        }
    }
}

impl<T> PodValue for PodRangeRef<T>
where
    T: PodRawValue,
    T: StaticTypePod,
{
    type Value = PodRangeValue<T::Value>;
    fn value(&self) -> PodResult<Self::Value> {
        Self::parse_raw_value(self.raw_value_ptr(), self.pod_header().size as usize)
    }
}

impl<T> WritePod for PodRangeRef<T>
where
    T: PodRawValue,
    T: BasicTypePod,
    T: WriteValue,
    T: WritePod,
{
    fn write_pod<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<()>
    where
        W: Write + Seek,
    {
        let value_offset = size_of::<spa_sys::spa_pod_choice>() as i64;
        let start_pos = buffer.stream_position()?;
        buffer.seek(SeekFrom::Current(value_offset))?;
        let value_size = write_count_size(buffer, |buffer| Self::write_raw_value(buffer, value))?;
        let end_pos = buffer.stream_position()?;
        buffer.seek(SeekFrom::Start(start_pos))?;
        write_header(
            buffer,
            (value_size + size_of::<spa_sys::spa_pod_choice_body>()) as u32,
            Type::CHOICE,
        )?;
        let child_size = value_size / 3;
        PodChoiceRef::<T>::write_raw_body(
            buffer,
            ChoiceType::RANGE,
            0,
            child_size as u32,
            T::static_type(),
        )?;
        buffer.seek(SeekFrom::Start(end_pos))?;
        Ok(())
    }
}

impl<T> WriteValue for PodRangeRef<T>
where
    T: PodRawValue,
    T: StaticTypePod,
    T: WriteValue,
{
    fn write_raw_value<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<()>
    where
        W: Write + Seek,
    {
        let element_size =
            write_count_size(buffer, |buffer| T::write_raw_value(buffer, &value.default))?;
        for v in [&value.min, &value.max] {
            let size = write_count_size(buffer, |buffer| T::write_raw_value(buffer, v))?;
            if element_size != size {
                return Err(PodError::UnexpectedChoiceElementSize(element_size, size));
            }
        }
        Ok(())
    }
}

impl<T> PrimitiveValue for PodRangeRef<T> {}

impl<T> Debug for PodRangeRef<T>
where
    T: PodRawValue,
    T: StaticTypePod,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodRangeRef")
            .field("pod.type", &self.pod_type())
            .field("pod.size", &self.pod_size())
            .field("value", &self.value())
            .finish()
    }
}
