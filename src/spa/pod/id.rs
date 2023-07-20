use std::fmt::{Debug, Formatter};
use std::io::{Seek, Write};
use std::marker::PhantomData;
use std::mem::size_of;

use spa_sys::spa_pod;

use crate::spa::pod::object::prop::Prop;
use crate::spa::pod::pod_buf::{AllocatedData, PodBuf};
use crate::spa::pod::restricted::{PodHeader, PrimitiveValue, StaticTypePod};
use crate::spa::pod::{
    BasicTypePod, FromPrimitiveValue, FromValue, PodError, PodResult, PodValue, SizedPod, WritePod,
    WriteValue, POD_ALIGN,
};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

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
    Self: Into<u32>,
    Self: Debug,
    Self: Clone,
{
    fn as_alloc_pod(&self) -> AllocatedData<PodIdRef<Self>> {
        PodIdRef::from_value(self).unwrap()
    }
}

impl PodIdType for u32 {}

impl PodIdType for Type {}

impl<T: PodIdType> PodValue for PodIdRef<T>
where
    T: PodIdType,
{
    type Value = T;
    type RawValue = u32;

    fn raw_value_ptr(&self) -> *const Self::RawValue {
        &self.raw.value
    }

    fn parse_raw_value(ptr: *const Self::RawValue, size: usize) -> PodResult<Self::Value> {
        unsafe { Ok((*ptr).into()) }
    }

    fn value(&self) -> PodResult<Self::Value> {
        Self::parse_raw_value(self.raw_value_ptr(), self.pod_header().size as usize)
    }
}

impl<'a, T: PodIdType> PrimitiveValue for PodIdRef<T> {}

impl<T: PodIdType> WritePod for PodIdRef<T>
where
    T: PodIdType,
{
    fn write_pod<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        Ok(Self::write_header(
            buffer,
            size_of::<u32>() as u32,
            <PodIdRef<T>>::static_type(),
        )? + Self::write_raw_value(buffer, value)?
            + Self::write_align_padding(buffer)?)
    }
}

impl<T: PodIdType> WriteValue for PodIdRef<T>
where
    T: PodIdType,
{
    fn write_raw_value<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        let raw_value: u32 = value.clone().into();
        Ok(Self::write_value(buffer, &raw_value)?)
    }
}

impl<T: PodIdType> PodHeader for PodIdRef<T> {
    fn pod_header(&self) -> &spa_pod {
        &self.raw.pod
    }
}

impl<T: PodIdType> StaticTypePod for PodIdRef<T> {
    fn static_type() -> Type {
        Type::ID
    }
}

impl<T: PodIdType> Debug for PodIdRef<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("PodIdType")
                .field("pod.type", &self.pod_type())
                .field("pod.size", &self.pod_size())
                .field("value", &self.value())
                .finish()
        }
    }
}

#[test]
fn test_from_value() {
    let allocated_pod = PodBuf::<PodIdRef<Type>>::from_value(&Type::POINTER)
        .unwrap()
        .into_pod();
    assert_eq!(allocated_pod.as_pod().as_ptr().align_offset(POD_ALIGN), 0);
    assert_eq!(allocated_pod.as_pod().pod_size(), 12);
    assert_eq!(allocated_pod.as_pod().pod_header().size, 4);
    assert_eq!(allocated_pod.as_pod().pod_header().type_, Type::ID.raw);
    assert_eq!(allocated_pod.as_pod().value().unwrap(), Type::POINTER);
}
