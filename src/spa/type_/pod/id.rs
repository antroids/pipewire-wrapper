use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::mem::size_of;

use crate::spa::type_::pod::object::prop::Prop;
use crate::spa::type_::pod::{Pod, PodError, PodResult, PodSubtype, PodValueParser, ReadablePod};
use crate::spa::type_::Type;

#[repr(transparent)]
pub struct PodIdRef<T: PodIdType = u32> {
    raw: spa_sys::spa_pod_id,
    phantom: PhantomData<T>,
}

impl<T: PodIdType> crate::wrapper::RawWrapper for PodIdRef<T> {
    type CType = spa_sys::spa_pod_id;

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
        &mut *(raw as *mut PodIdRef<T>)
    }
}

pub trait PodIdType
where
    Self: From<u32>,
    Self: Debug,
{
}

impl PodIdType for u32 {}

impl PodIdType for Type {}

impl<T: PodIdType> PodValueParser<*const u8> for PodIdRef<T> {
    fn parse(
        content_size: usize,
        header_or_value: *const u8,
    ) -> PodResult<<Self as ReadablePod>::Value> {
        unsafe { Self::parse(content_size, *(header_or_value as *const u32)) }
    }
}

impl<T: PodIdType> PodValueParser<u32> for PodIdRef<T> {
    fn parse(content_size: usize, header_or_value: u32) -> PodResult<<Self as ReadablePod>::Value> {
        if content_size < size_of::<u32>() {
            Err(PodError::DataIsTooShort(size_of::<u32>(), content_size))
        } else {
            Ok(header_or_value.into())
        }
    }
}

impl<T: PodIdType> ReadablePod for PodIdRef<T> {
    type Value = T;

    fn value(&self) -> PodResult<Self::Value> {
        Ok(self.raw.value.into())
    }
}

impl<T: PodIdType> Pod for PodIdRef<T> {
    fn pod_size(&self) -> usize {
        self.upcast().pod_size()
    }
}

impl<T: PodIdType> PodSubtype for PodIdRef<T> {
    fn static_type() -> Type {
        Type::ID
    }
}

impl<T: PodIdType> Debug for PodIdRef<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("PodIdType")
                .field("pod", &self.upcast())
                .field("value", &self.value())
                .finish()
        }
    }
}
