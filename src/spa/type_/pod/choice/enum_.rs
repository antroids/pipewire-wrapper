use std::fmt::{Debug, Formatter};
use std::io::{Seek, SeekFrom, Write};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::addr_of;

use spa_sys::spa_pod;

use crate::spa::type_::pod::choice::{ChoiceType, PodChoiceBodyRef, PodChoiceRef};
use crate::spa::type_::pod::iterator::PodValueIterator;
use crate::spa::type_::pod::pod_buf::PodBuf;
use crate::spa::type_::pod::restricted::{PodHeader, StaticTypePod};
use crate::spa::type_::pod::{
    BasicTypePod, PodError, PodIntRef, PodRef, PodResult, PodValueParser, ReadablePod, SizedPod,
    WritablePod, WritableValue, POD_ALIGN,
};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PodEnumValue<T> {
    default: T,
    alternatives: Vec<T>,
}

impl<T> PodEnumValue<T> {
    pub fn default(&self) -> &T {
        &self.default
    }
    pub fn alternatives(&self) -> &Vec<T> {
        &self.alternatives
    }
}

#[repr(transparent)]
pub struct PodEnumRef<T> {
    raw: spa_sys::spa_pod_choice,
    phantom: PhantomData<T>,
}

impl<T> PodEnumRef<T> {
    pub fn choice(&self) -> &PodChoiceRef {
        unsafe { PodChoiceRef::from_raw_ptr(addr_of!(self.raw)) }
    }
}

impl<T> crate::wrapper::RawWrapper for PodEnumRef<T>
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
        &mut *(raw as *mut PodEnumRef<T>)
    }
}

impl<T> StaticTypePod for PodEnumRef<T>
where
    T: PodValueParser<*const u8>,
    T: StaticTypePod,
{
    fn static_type() -> Type {
        PodChoiceRef::static_type()
    }
}

impl<T> PodHeader for PodEnumRef<T>
where
    T: PodValueParser<*const u8>,
    T: StaticTypePod,
{
    fn pod_header(&self) -> &spa_pod {
        &self.raw.pod
    }
}

impl<T> ReadablePod for PodEnumRef<T>
where
    T: PodValueParser<*const u8>,
    T: StaticTypePod,
{
    type Value = PodEnumValue<T::Value>;

    fn value(&self) -> PodResult<Self::Value> {
        let body = self.choice().body();
        if body.type_() == ChoiceType::ENUM {
            if T::static_type() == body.child().type_() {
                let content_size = self.pod_size() - size_of::<PodEnumRef<T>>();
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
                Ok(PodEnumValue {
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
                ChoiceType::ENUM,
                body.type_(),
            ))
        }
    }
}

impl<T> WritablePod for PodEnumRef<T>
where
    T: PodValueParser<*const u8>,
    T: StaticTypePod,
    T: WritableValue,
{
    fn write_pod<W>(buffer: &mut W, value: &<Self as ReadablePod>::Value) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        let elements_count = value.alternatives.len() + 1;
        Ok(Self::write_end_than_start(
            buffer,
            size_of::<spa_sys::spa_pod_choice>(),
            |buffer, value_size| {
                let child_size = value_size / elements_count;
                Ok(Self::write_header(
                    buffer,
                    (value_size + size_of::<spa_sys::spa_pod_choice_body>()) as u32,
                    Type::CHOICE,
                )? + PodChoiceRef::write_raw_body(
                    buffer,
                    ChoiceType::ENUM,
                    0,
                    child_size as u32,
                    T::static_type(),
                )?)
            },
            |buffer| Self::write_raw_value(buffer, value),
        )? + Self::write_align_padding(buffer)?)
    }
}

impl<T> WritableValue for PodEnumRef<T>
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
        for v in &value.alternatives {
            let size = T::write_raw_value(buffer, v)?;
            if element_size != size {
                return Err(PodError::UnexpectedChoiceElementSize(element_size, size));
            }
        }
        Ok(element_size + element_size * value.alternatives.len())
    }
}

impl<T> Debug for PodEnumRef<T>
where
    T: PodValueParser<*const u8>,
    T: StaticTypePod,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodEnumRef")
            .field("pod.type", &self.upcast().type_())
            .field("pod.size", &self.upcast().size())
            .field("value", &self.value())
            .finish()
    }
}

#[test]
fn test_from_value() {
    let v1 = PodEnumValue {
        default: 123,
        alternatives: vec![123, 234, 345, 456],
    };
    let v2 = PodEnumValue {
        default: 123,
        alternatives: vec![],
    };
    let allocated_pod = PodBuf::<PodEnumRef<PodIntRef>>::from_value(&v1)
        .unwrap()
        .into_pod();
    assert_eq!(allocated_pod.as_pod().as_ptr().align_offset(POD_ALIGN), 0);
    assert_eq!(allocated_pod.as_pod().pod_size(), 44);
    assert_eq!(allocated_pod.as_pod().pod_header().size, 36);
    assert_eq!(allocated_pod.as_pod().pod_header().type_, Type::CHOICE.raw);
    assert_eq!(
        allocated_pod.as_pod().choice().body().type_(),
        ChoiceType::ENUM
    );
    assert_eq!(allocated_pod.as_pod().choice().body().flags(), 0);
    assert_eq!(
        allocated_pod.as_pod().choice().body().child().type_(),
        Type::INT
    );
    assert_eq!(allocated_pod.as_pod().choice().body().child().size(), 4);
    assert_eq!(allocated_pod.as_pod().value().unwrap(), v1);
    assert_ne!(allocated_pod.as_pod().value().unwrap(), v2);

    let allocated_pod = PodBuf::<PodEnumRef<PodIntRef>>::from_value(&v2)
        .unwrap()
        .into_pod();
    assert_eq!(allocated_pod.as_pod().as_ptr().align_offset(POD_ALIGN), 0);
    assert_eq!(allocated_pod.as_pod().pod_size(), 28);
    assert_eq!(allocated_pod.as_pod().pod_header().size, 20);
    assert_eq!(allocated_pod.as_pod().pod_header().type_, Type::CHOICE.raw);
    assert_eq!(
        allocated_pod.as_pod().choice().body().type_(),
        ChoiceType::ENUM
    );
    assert_eq!(allocated_pod.as_pod().choice().body().flags(), 0);
    assert_eq!(
        allocated_pod.as_pod().choice().body().child().type_(),
        Type::INT
    );
    assert_eq!(allocated_pod.as_pod().choice().body().child().size(), 4);
    assert_eq!(allocated_pod.as_pod().value().unwrap(), v2);
    assert_ne!(allocated_pod.as_pod().value().unwrap(), v1);
}
