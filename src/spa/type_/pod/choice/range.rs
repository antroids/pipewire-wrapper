use std::fmt::{Debug, Formatter};
use std::io::{Seek, Write};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::addr_of;

use spa_sys::spa_pod;

use crate::spa::type_::pod::choice::{ChoiceType, PodChoiceBodyRef, PodChoiceRef};
use crate::spa::type_::pod::iterator::PodValueIterator;
use crate::spa::type_::pod::restricted::{PodHeader, StaticTypePod};
use crate::spa::type_::pod::{
    BasicTypePod, PodError, PodRef, PodResult, PodValueParser, ReadablePod, SizedPod, WritablePod,
    WritableValue,
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
}

#[repr(transparent)]
pub struct PodRangeRef<T> {
    raw: spa_sys::spa_pod_choice,
    phantom: PhantomData<T>,
}

impl<T> PodRangeRef<T> {
    pub fn choice(&self) -> &PodChoiceRef {
        unsafe { PodChoiceRef::from_raw_ptr(addr_of!(self.raw)) }
    }
}

impl<T> crate::wrapper::RawWrapper for PodRangeRef<T>
where
    T: PodValueParser<*const u8>,
{
    type CType = spa_sys::spa_pod_choice;

    fn as_raw_ptr(&self) -> *mut Self::CType {
        &self.raw as *const _ as *mut _
    }

    fn as_raw(&self) -> &Self::CType {
        &self.raw
    }

    fn from_raw(raw: Self::CType) -> Self {
        Self {
            raw,
            phantom: PhantomData::default(),
        }
    }

    unsafe fn mut_from_raw_ptr<'a>(raw: *mut Self::CType) -> &'a mut Self {
        &mut *(raw as *mut PodRangeRef<T>)
    }
}

impl<T> StaticTypePod for PodRangeRef<T>
where
    T: PodValueParser<*const u8>,
    T: StaticTypePod,
{
    fn static_type() -> Type {
        PodChoiceRef::static_type()
    }
}

impl<T> PodHeader for PodRangeRef<T>
where
    T: PodValueParser<*const u8>,
    T: StaticTypePod,
{
    fn pod_header(&self) -> &spa_pod {
        &self.raw.pod
    }
}

impl<T> ReadablePod for PodRangeRef<T>
where
    T: PodValueParser<*const u8>,
    T: StaticTypePod,
{
    type Value = PodRangeValue<T::Value>;

    fn value(&self) -> PodResult<Self::Value> {
        let body = self.choice().body();
        if body.type_() == ChoiceType::RANGE {
            if T::static_type() == body.child().type_() {
                let content_size = self.pod_size() - size_of::<PodRangeRef<T>>();
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

impl<T> WritablePod for PodRangeRef<T>
where
    T: PodValueParser<*const u8>,
    T: StaticTypePod,
    T: WritableValue,
{
    fn write_pod<W>(buffer: &mut W, value: &<Self as ReadablePod>::Value) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        Ok(Self::write_end_than_start(
            buffer,
            size_of::<spa_sys::spa_pod_choice>(),
            |buffer, value_size| {
                let child_size = value_size / 3;
                Ok(Self::write_header(
                    buffer,
                    (value_size + size_of::<spa_sys::spa_pod_choice_body>()) as u32,
                    Type::CHOICE,
                )? + PodChoiceRef::write_raw_body(
                    buffer,
                    ChoiceType::RANGE,
                    0,
                    child_size as u32,
                    T::static_type(),
                )?)
            },
            |buffer| Self::write_raw_value(buffer, value),
        )? + Self::write_align_padding(buffer)?)
    }
}

impl<T> WritableValue for PodRangeRef<T>
where
    T: PodValueParser<*const u8>,
    T: StaticTypePod,
    T: WritableValue,
{
    fn write_raw_value<W>(buffer: &mut W, value: &<Self as ReadablePod>::Value) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        let element_size = T::write_raw_value(buffer, &value.default)?;
        for v in [&value.min, &value.max] {
            let size = T::write_raw_value(buffer, v)?;
            if element_size != size {
                return Err(PodError::UnexpectedChoiceElementSize(element_size, size));
            }
        }
        Ok(element_size * 3)
    }
}

impl<T> Debug for PodRangeRef<T>
where
    T: PodValueParser<*const u8>,
    T: StaticTypePod,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodRangeRef")
            .field("pod.type", &self.upcast().type_())
            .field("pod.size", &self.upcast().size())
            .field("value", &self.value())
            .finish()
    }
}
