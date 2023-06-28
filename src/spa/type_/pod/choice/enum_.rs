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
pub struct PodEnumValue<T> {
    default: T,
    alternatives: Vec<T>,
}

#[repr(transparent)]
pub struct PodEnumRef<T> {
    raw: spa_sys::spa_pod,
    phantom: PhantomData<T>,
}

impl<T> PodEnumRef<T> {
    fn content_size(&self) -> usize {
        self.raw.size as usize
    }
}

impl<T> crate::wrapper::RawWrapper for PodEnumRef<T>
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
        &mut *(raw as *mut PodEnumRef<T>)
    }
}

impl<T> PodSubtype for PodEnumRef<T>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
{
    fn static_type() -> Type {
        T::static_type()
    }
}

impl<T> Pod for PodEnumRef<T>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
{
    fn pod_size(&self) -> usize {
        self.upcast().pod_size()
    }
}

impl<'a, T> PodValueParser<&'a PodEnumRef<T>> for PodEnumRef<T>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
{
    fn parse(
        content_size: usize,
        header_or_value: &'a PodEnumRef<T>,
    ) -> PodResult<<Self as ReadablePod>::Value> {
        if T::static_type() == header_or_value.upcast().type_() {
            let element_size = header_or_value.raw.size as usize;
            let mut iter: PodValueIterator<T> = PodValueIterator::new(
                unsafe { header_or_value.content_ptr() },
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
            Err(PodError::WrongPodTypeToCast)
        }
    }
}

impl<'a, T> PodValueParser<&'a PodChoiceBodyRef> for PodEnumRef<T>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
{
    fn parse(
        content_size: usize,
        header_or_value: &'a PodChoiceBodyRef,
    ) -> PodResult<<Self as ReadablePod>::Value> {
        Self::parse(content_size, addr_of!(header_or_value.raw.child).cast())
    }
}

impl<T> PodValueParser<*const u8> for PodEnumRef<T>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
{
    fn parse(
        content_size: usize,
        header_or_value: *const u8,
    ) -> PodResult<<Self as ReadablePod>::Value> {
        unsafe {
            Self::parse(
                content_size,
                PodEnumRef::from_raw_ptr(header_or_value.cast()),
            )
        }
    }
}

impl<T> ReadablePod for PodEnumRef<T>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
{
    type Value = PodEnumValue<T::Value>;

    fn value(&self) -> PodResult<Self::Value> {
        Self::parse(self.content_size(), self)
    }
}

impl<T> Debug for PodEnumRef<T>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodEnumRef")
            .field("pod", &self.upcast())
            .field("value", &self.value())
            .finish()
    }
}
