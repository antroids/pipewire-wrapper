/*
 * SPDX-License-Identifier: MIT
 */
use std::fmt::{Debug, Formatter};
use std::io::{Seek, Write};
use std::marker::PhantomData;
use std::mem::size_of;

use spa_sys::spa_pod;

use pipewire_proc_macro::RawWrapper;

use crate::spa::pod::object::prop::Prop;
use crate::spa::pod::pod_buf::{AllocatedData, PodBuf};
use crate::spa::pod::restricted::{PodHeader, PodRawValue, PrimitiveValue, StaticTypePod};
use crate::spa::pod::{
    BasicTypePod, FromPrimitiveValue, FromValue, PodError, PodResult, PodValue, SizedPod, WritePod,
    WriteValue, POD_ALIGN,
};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

use super::restricted::{write_align_padding, write_header, write_value};

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodIdRef<T: PodIdType = u32> {
    #[raw]
    raw: spa_sys::spa_pod_id,
    phantom: PhantomData<T>,
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

impl<T: PodIdType> PodRawValue for PodIdRef<T>
where
    T: PodIdType,
{
    type RawValue = u32;

    fn raw_value_ptr(&self) -> *const Self::RawValue {
        &self.raw.value
    }

    fn parse_raw_value(ptr: *const Self::RawValue, size: usize) -> PodResult<Self::Value> {
        unsafe { Ok((*ptr).into()) }
    }
}

impl<T: PodIdType> PodValue for PodIdRef<T>
where
    T: PodIdType,
{
    type Value = T;
    fn value(&self) -> PodResult<Self::Value> {
        Self::parse_raw_value(self.raw_value_ptr(), self.pod_header().size as usize)
    }
}

impl<T: PodIdType> PrimitiveValue for PodIdRef<T> {}

impl<T: PodIdType> WritePod for PodIdRef<T>
where
    T: PodIdType,
{
    fn write_pod<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<()>
    where
        W: Write + Seek,
    {
        write_header(
            buffer,
            size_of::<u32>() as u32,
            <PodIdRef<T>>::static_type(),
        )?;
        Self::write_raw_value(buffer, value)?;
        write_align_padding(buffer)?;
        Ok(())
    }
}

impl<T: PodIdType> WriteValue for PodIdRef<T>
where
    T: PodIdType,
{
    fn write_raw_value<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<()>
    where
        W: Write + Seek,
    {
        let raw_value: u32 = value.clone().into();
        write_value(buffer, &raw_value)
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
