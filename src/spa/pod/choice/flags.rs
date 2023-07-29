/*
 * SPDX-License-Identifier: MIT
 */
use std::fmt::{Debug, Formatter};
use std::io::{Seek, SeekFrom, Write};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::addr_of;

use spa_sys::spa_pod;

use pipewire_wrapper_proc_macro::RawWrapper;

use crate::spa::pod::choice::{ChoiceType, PodChoiceBodyRef, PodChoiceRef};
use crate::spa::pod::iterator::PodValueIterator;
use crate::spa::pod::pod_buf::AllocPod;
use crate::spa::pod::restricted::{
    write_count_size, write_header, PodHeader, PodRawValue, PrimitiveValue,
};
use crate::spa::pod::{
    BasicTypePod, PodError, PodResult, PodValue, SizedPod, WritePod, WriteValue,
};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PodFlagsValue<T> {
    default: T,
    alternatives: Vec<T>,
}

impl<T> PodFlagsValue<T> {
    pub fn default(&self) -> &T {
        &self.default
    }
    pub fn alternatives(&self) -> &Vec<T> {
        &self.alternatives
    }

    pub fn new(default: T, alternatives: Vec<T>) -> Self {
        Self {
            default,
            alternatives,
        }
    }

    pub fn to_alloc_pod<P>(&self) -> PodResult<AllocPod<PodFlagsRef<P>>>
    where
        PodFlagsRef<P>: WritePod,
        PodFlagsRef<P>: PodRawValue<Value = Self>,
        PodFlagsRef<P>: PrimitiveValue,
    {
        AllocPod::from_value(self)
    }
}

impl<T: Clone> PodFlagsValue<T> {
    pub fn from_default(default: T) -> Self {
        Self {
            default: default.clone(),
            alternatives: vec![default],
        }
    }
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodFlagsRef<T> {
    #[raw]
    raw: spa_sys::spa_pod_choice,
    phantom: PhantomData<T>,
}

impl<T: PodRawValue> PodFlagsRef<T> {
    pub fn choice(&self) -> &PodChoiceRef<T> {
        unsafe { PodChoiceRef::from_raw_ptr(addr_of!(self.raw)) }
    }
}

impl<'a, T: PodRawValue> From<&'a PodFlagsRef<T>> for &'a PodChoiceRef<T> {
    fn from(value: &'a PodFlagsRef<T>) -> Self {
        value.choice()
    }
}

impl<T> PodHeader for PodFlagsRef<T>
where
    T: PodRawValue,
{
    fn pod_header(&self) -> &spa_pod {
        &self.raw.pod
    }

    fn static_type() -> Type {
        PodChoiceRef::<T>::static_type()
    }
}

impl<T> PodRawValue for PodFlagsRef<T>
where
    T: PodRawValue,
    T: PodHeader,
{
    type RawValue = spa_sys::spa_pod_choice_body;

    fn raw_value_ptr(&self) -> *const Self::RawValue {
        &self.raw.body
    }

    fn parse_raw_value(ptr: *const Self::RawValue, size: usize) -> PodResult<Self::Value> {
        let body = unsafe { PodChoiceBodyRef::from_raw_ptr(ptr) };
        if body.type_() == ChoiceType::FLAGS {
            if T::static_type() == body.child().type_() || T::static_type() == Type::POD {
                let content_size = size - size_of::<Self::RawValue>();
                let element_size = body.child().size() as usize;
                let mut iter: PodValueIterator<T> = PodValueIterator::new(
                    unsafe { body.content_ptr().cast() },
                    content_size,
                    element_size,
                );
                let default = iter
                    .next()
                    .ok_or(PodError::DataIsTooShort(element_size, content_size))?;
                let mut alternatives = Vec::new();
                iter.for_each(|a| alternatives.push(a));
                Ok(PodFlagsValue {
                    default,
                    alternatives,
                })
            } else {
                Err(PodError::WrongPodTypeToCast(
                    T::static_type(),
                    body.child().type_(),
                ))
            }
        } else {
            Err(PodError::UnexpectedChoiceType(
                ChoiceType::FLAGS,
                body.type_(),
            ))
        }
    }
}

impl<T> PodValue for PodFlagsRef<T>
where
    T: PodRawValue,
    T: PodHeader,
{
    type Value = PodFlagsValue<T::Value>;
    fn value(&self) -> PodResult<Self::Value> {
        Self::parse_raw_value(self.raw_value_ptr(), self.pod_header().size as usize)
    }
}

impl<T> WritePod for PodFlagsRef<T>
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
        let elements_count = value.alternatives.len() + 1;
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
        let child_size = value_size / elements_count;
        PodChoiceRef::<T>::write_raw_body(
            buffer,
            ChoiceType::FLAGS,
            0,
            child_size as u32,
            T::static_type(),
        )?;
        buffer.seek(SeekFrom::Start(end_pos))?;
        Ok(())
    }
}

impl<T> WriteValue for PodFlagsRef<T>
where
    T: PodRawValue,
    T: WriteValue,
    T: PodHeader,
{
    fn write_raw_value<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<()>
    where
        W: Write + Seek,
    {
        let element_size =
            write_count_size(buffer, |buffer| T::write_raw_value(buffer, &value.default))?;
        for v in &value.alternatives {
            let size = write_count_size(buffer, |buffer| T::write_raw_value(buffer, v))?;
            if element_size != size {
                return Err(PodError::UnexpectedChoiceElementSize(element_size, size));
            }
        }
        Ok(())
    }
}

impl<T> PrimitiveValue for PodFlagsRef<T> {}

impl<T> Debug for PodFlagsRef<T>
where
    T: PodRawValue,
    T: PodHeader,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodFlagsRef")
            .field("pod.type", &self.pod_type())
            .field("pod.size", &self.pod_size())
            .field("value", &self.value())
            .finish()
    }
}
