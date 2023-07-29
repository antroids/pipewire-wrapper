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
pub struct PodStepValue<T> {
    default: T,
    min: T,
    max: T,
    step: T,
}

impl<T> PodStepValue<T> {
    pub fn default(&self) -> &T {
        &self.default
    }
    pub fn min(&self) -> &T {
        &self.min
    }
    pub fn max(&self) -> &T {
        &self.max
    }
    pub fn step(&self) -> &T {
        &self.step
    }

    pub fn new(default: T, min: T, max: T, step: T) -> Self {
        Self {
            default,
            min,
            max,
            step,
        }
    }

    pub fn to_alloc_pod<P>(&self) -> PodResult<AllocPod<PodStepRef<P>>>
    where
        PodStepRef<P>: WritePod,
        PodStepRef<P>: PodRawValue<Value = Self>,
        PodStepRef<P>: PrimitiveValue,
    {
        AllocPod::from_value(self)
    }
}

impl<T: Clone> PodStepValue<T> {
    pub fn from_default(default: T) -> Self {
        Self {
            default: default.clone(),
            min: default.clone(),
            max: default.clone(),
            step: default,
        }
    }
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodStepRef<T> {
    #[raw]
    raw: spa_sys::spa_pod_choice,
    phantom: PhantomData<T>,
}

impl<T: PodRawValue> PodStepRef<T> {
    pub fn choice(&self) -> &PodChoiceRef<T> {
        unsafe { PodChoiceRef::from_raw_ptr(addr_of!(self.raw)) }
    }
}

impl<'a, T: PodRawValue> From<&'a PodStepRef<T>> for &'a PodChoiceRef<T> {
    fn from(value: &'a PodStepRef<T>) -> Self {
        value.choice()
    }
}

impl<T> PodHeader for PodStepRef<T>
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

impl<T> PodRawValue for PodStepRef<T>
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
        if body.type_() == ChoiceType::STEP {
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
                let step = iter.next().unwrap();
                if iter.next().is_some() {
                    Err(PodError::UnexpectedChoiceElement)
                } else {
                    Ok(PodStepValue {
                        default,
                        min,
                        max,
                        step,
                    })
                }
            } else {
                Err(PodError::WrongPodTypeToCast(
                    T::static_type(),
                    body.child().type_(),
                ))
            }
        } else {
            Err(PodError::UnexpectedChoiceType(
                ChoiceType::STEP,
                body.type_(),
            ))
        }
    }
}

impl<T> PodValue for PodStepRef<T>
where
    T: PodRawValue,
    T: PodHeader,
{
    type Value = PodStepValue<T::Value>;
    fn value(&self) -> PodResult<Self::Value> {
        Self::parse_raw_value(self.raw_value_ptr(), self.pod_header().size as usize)
    }
}

impl<T> WritePod for PodStepRef<T>
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
        let child_size = value_size / 4;
        PodChoiceRef::<T>::write_raw_body(
            buffer,
            ChoiceType::STEP,
            0,
            child_size as u32,
            T::static_type(),
        )?;
        buffer.seek(SeekFrom::Start(end_pos))?;
        Ok(())
    }
}

impl<T> WriteValue for PodStepRef<T>
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
        for v in [&value.min, &value.max, &value.step] {
            let size = write_count_size(buffer, |buffer| T::write_raw_value(buffer, v))?;
            if element_size != size {
                return Err(PodError::UnexpectedChoiceElementSize(element_size, size));
            }
        }
        Ok(())
    }
}

impl<T> PrimitiveValue for PodStepRef<T> {}

impl<T> Debug for PodStepRef<T>
where
    T: PodRawValue,
    T: PodHeader,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodStepRef")
            .field("pod.type", &self.pod_type())
            .field("pod.size", &self.pod_size())
            .field("value", &self.value())
            .finish()
    }
}
