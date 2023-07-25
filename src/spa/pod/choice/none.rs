use std::fmt::{Debug, Formatter};
use std::io::{Seek, Write};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::addr_of;

use spa_sys::spa_pod;

use pipewire_proc_macro::RawWrapper;

use crate::spa::pod::choice::{ChoiceType, PodChoiceBodyRef, PodChoiceRef};
use crate::spa::pod::iterator::PodValueIterator;
use crate::spa::pod::pod_buf::PodBuf;
use crate::spa::pod::restricted::{PodHeader, PodRawValue, PrimitiveValue, StaticTypePod};
use crate::spa::pod::{
    BasicTypePod, PodError, PodIntRef, PodLongRef, PodRef, PodResult, PodValue, SizedPod, Upcast,
    WritePod, WriteValue, POD_ALIGN,
};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodNoneRef<T> {
    #[raw]
    raw: spa_sys::spa_pod_choice,
    phantom: PhantomData<T>,
}

impl<T: PodRawValue> PodNoneRef<T> {
    pub fn choice(&self) -> &PodChoiceRef<T> {
        unsafe { PodChoiceRef::from_raw_ptr(addr_of!(self.raw)) }
    }
}

impl<'a, T: PodRawValue> From<&'a PodNoneRef<T>> for &'a PodChoiceRef<T> {
    fn from(value: &'a PodNoneRef<T>) -> Self {
        value.choice()
    }
}

impl<T> StaticTypePod for PodNoneRef<T>
where
    T: PodRawValue,
    T: StaticTypePod,
{
    fn static_type() -> Type {
        PodChoiceRef::<T>::static_type()
    }
}

impl<T> PodHeader for PodNoneRef<T>
where
    T: PodRawValue,
    T: StaticTypePod,
{
    fn pod_header(&self) -> &spa_pod {
        &self.raw.pod
    }
}

impl<T> PodRawValue for PodNoneRef<T>
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
        if body.type_() == ChoiceType::NONE {
            if T::static_type() == body.child().type_() || T::static_type() == Type::POD {
                let content_size = size - size_of::<Self::RawValue>();
                let element_size = body.child().size() as usize;
                let mut iter: PodValueIterator<T> = PodValueIterator::new(
                    unsafe { body.content_ptr().cast() },
                    content_size,
                    element_size,
                );
                let value = iter
                    .next()
                    .ok_or(PodError::DataIsTooShort(element_size, content_size))?;
                Ok(value)
            } else {
                Err(PodError::WrongPodTypeToCast(
                    T::static_type(),
                    body.child().type_(),
                ))
            }
        } else {
            Err(PodError::UnexpectedChoiceType(
                ChoiceType::NONE,
                body.type_(),
            ))
        }
    }
}

impl<T> PodValue for PodNoneRef<T>
where
    T: PodRawValue,
    T: StaticTypePod,
{
    type Value = T::Value;
    fn value(&self) -> PodResult<Self::Value> {
        Self::parse_raw_value(self.raw_value_ptr(), self.pod_header().size as usize)
    }
}

impl<T> WritePod for PodNoneRef<T>
where
    T: PodRawValue,
    T: BasicTypePod,
    T: WriteValue,
    T: WritePod,
{
    fn write_pod<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        Ok(Self::write_end_than_start(
            buffer,
            size_of::<spa_sys::spa_pod_choice>(),
            |buffer, value_size| {
                Ok(Self::write_header(
                    buffer,
                    (value_size + size_of::<spa_sys::spa_pod_choice_body>()) as u32,
                    Type::CHOICE,
                )? + PodChoiceRef::<T>::write_raw_body(
                    buffer,
                    ChoiceType::NONE,
                    0,
                    value_size as u32,
                    T::static_type(),
                )?)
            },
            |buffer| Self::write_raw_value(buffer, value),
        )? + Self::write_align_padding(buffer)?)
    }
}

impl<T> WriteValue for PodNoneRef<T>
where
    T: PodRawValue,
    T: StaticTypePod,
    T: WriteValue,
{
    fn write_raw_value<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        T::write_raw_value(buffer, value)
    }
}

impl<T> PrimitiveValue for PodNoneRef<T> {}

impl<T> Debug for PodNoneRef<T>
where
    T: PodRawValue,
    T: StaticTypePod,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodNoneRef")
            .field("pod.type", &self.pod_type())
            .field("pod.size", &self.pod_size())
            .field("value", &self.value())
            .finish()
    }
}

#[test]
fn test_from_value() {
    let allocated_pod = PodBuf::<PodNoneRef<PodLongRef>>::from_primitive_value(1234567i64)
        .unwrap()
        .into_pod();
    assert_eq!(allocated_pod.as_pod().as_ptr().align_offset(POD_ALIGN), 0);
    assert_eq!(allocated_pod.as_pod().pod_size(), 32);
    assert_eq!(allocated_pod.as_pod().pod_header().size, 24);
    assert_eq!(allocated_pod.as_pod().pod_header().type_, Type::CHOICE.raw);
    assert_eq!(
        allocated_pod.as_pod().choice().body().type_(),
        ChoiceType::NONE
    );
    assert_eq!(allocated_pod.as_pod().choice().body().flags(), 0);
    assert_eq!(
        allocated_pod.as_pod().choice().body().child().type_(),
        Type::LONG
    );
    assert_eq!(allocated_pod.as_pod().choice().body().child().size(), 8);
    assert_eq!(allocated_pod.as_pod().value().unwrap(), 1234567i64);
}
