use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::addr_of;

use crate::spa::type_::pod::choice::{ChoiceType, PodChoiceBodyRef, PodChoiceRef};
use crate::spa::type_::pod::iterator::PodValueIterator;
use crate::spa::type_::pod::{
    Pod, PodError, PodRef, PodResult, PodSubtype, PodValueParser, ReadablePod,
};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

#[derive(Debug)]
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
}

#[repr(transparent)]
pub struct PodFlagsRef<T> {
    raw: spa_sys::spa_pod_choice,
    phantom: PhantomData<T>,
}

impl<T> PodFlagsRef<T> {
    pub fn choice(&self) -> &PodChoiceRef {
        unsafe { PodChoiceRef::from_raw_ptr(addr_of!(self.raw)) }
    }
}

impl<T> crate::wrapper::RawWrapper for PodFlagsRef<T>
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
        &mut *(raw as *mut PodFlagsRef<T>)
    }
}

impl<T> PodSubtype for PodFlagsRef<T>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
{
    fn static_type() -> Type {
        PodChoiceRef::static_type()
    }
}

impl<T> Pod for PodFlagsRef<T>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
{
    fn pod_size(&self) -> usize {
        self.upcast().pod_size()
    }
}

impl<T> ReadablePod for PodFlagsRef<T>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
{
    type Value = PodFlagsValue<T::Value>;

    fn value(&self) -> PodResult<Self::Value> {
        let body = self.choice().body();
        if body.type_() == ChoiceType::FLAGS {
            if T::static_type() == body.child().type_() {
                let content_size = self.pod_size() - size_of::<PodFlagsRef<T>>();
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

impl<T> Debug for PodFlagsRef<T>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodFlagsRef")
            .field("pod.type", &self.upcast().type_())
            .field("pod.size", &self.upcast().size())
            .field("value", &self.value())
            .finish()
    }
}
