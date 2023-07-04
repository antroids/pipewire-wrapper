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
};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

#[derive(Debug)]
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
{
    fn write<W>(buffer: &mut W, value: <Self as ReadablePod>::Value) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        todo!()
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
