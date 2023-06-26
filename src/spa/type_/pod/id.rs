use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::mem::size_of;

use crate::spa::type_::pod::object::Prop;
use crate::spa::type_::pod::{
    BasicTypePod, PodError, PodResult, PodValueParser, ReadablePod, SizedPod,
};
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

impl PodIdType for Prop {}

impl<T: PodIdType> PodValueParser<*const u8> for PodIdRef<T> {
    type To = T;

    fn parse(size: u32, value: *const u8) -> PodResult<Self::To> {
        unsafe { Self::parse(size, *(value as *const u32)) }
    }
}

impl<T: PodIdType> PodValueParser<u32> for PodIdRef<T> {
    type To = T;

    fn parse(size: u32, value: u32) -> PodResult<Self::To> {
        if (size as usize) < size_of::<u32>() {
            Err(PodError::DataIsTooShort)
        } else {
            Ok(value.into())
        }
    }
}

impl<T: PodIdType> ReadablePod for PodIdRef<T> {
    type Value = T;

    fn value(&self) -> PodResult<Self::Value> {
        Ok(self.raw.value.into())
    }
}

impl<T: PodIdType> SizedPod for PodIdRef<T> {
    fn size_bytes(&self) -> usize {
        self.upcast().size_bytes()
    }
}

impl<T: PodIdType> BasicTypePod for PodIdRef<T> {}

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
