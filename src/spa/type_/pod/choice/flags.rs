use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::addr_of;

use crate::spa::type_::pod::choice::PodChoiceBodyRef;
use crate::spa::type_::pod::iterator::PodValueIterator;
use crate::spa::type_::pod::{Pod, PodError, PodResult, PodSubtype, PodValueParser, ReadablePod};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

#[derive(Debug)]
pub struct PodFlagsValue<T> {
    default: T,
    alternatives: Vec<T>,
}

#[repr(transparent)]
pub struct PodFlagsRef<T> {
    raw: spa_sys::spa_pod,
    phantom: PhantomData<T>,
}

impl<T> crate::wrapper::RawWrapper for PodFlagsRef<T>
where
    T: PodValueParser<*const u8>,
{
    type CType = spa_sys::spa_pod;

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
        T::static_type()
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

impl<'a, T> PodValueParser<&'a PodChoiceBodyRef> for PodFlagsRef<T>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
{
    type To = PodFlagsValue<T::To>;

    fn parse(size: u32, value: &'a PodChoiceBodyRef) -> PodResult<Self::To> {
        Self::parse(size, addr_of!(value.raw.child).cast())
    }
}

impl<'a, T> PodValueParser<&'a PodFlagsRef<T>> for PodFlagsRef<T>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
{
    type To = PodFlagsValue<T::To>;

    fn parse(size: u32, value: &'a PodFlagsRef<T>) -> PodResult<Self::To> {
        if T::static_type() == value.upcast().type_() {
            let size = size as usize;
            let element_size = value.raw.size as usize;
            let mut iter: PodValueIterator<T> =
                PodValueIterator::new(unsafe { value.content_ptr() }, size, element_size);
            let default = iter
                .next()
                .ok_or(PodError::DataIsTooShort(element_size, size))?;
            let mut alternatives = Vec::new();
            iter.for_each(|a| alternatives.push(a));
            Ok(PodFlagsValue {
                default,
                alternatives,
            })
        } else {
            Err(PodError::WrongPodTypeToCast)
        }
    }
}

impl<T> PodValueParser<*const u8> for PodFlagsRef<T>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
{
    type To = PodFlagsValue<T::To>;

    fn parse(size: u32, value: *const u8) -> PodResult<Self::To> {
        unsafe { Self::parse(size, PodFlagsRef::from_raw_ptr(value.cast())) }
    }
}

impl<T> ReadablePod for PodFlagsRef<T>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
{
    type Value = PodFlagsValue<T::To>;

    fn value(&self) -> PodResult<Self::Value> {
        let content_size = self.pod_size() - size_of::<PodFlagsRef<T>>();
        Self::parse(content_size as u32, self)
    }
}

impl<T> Debug for PodFlagsRef<T>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodFlagsRef")
            .field("pod", &self.upcast())
            .field("value", &self.value())
            .finish()
    }
}
