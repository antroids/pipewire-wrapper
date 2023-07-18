use std::fmt::{Debug, Formatter};
use std::io::{Seek, Write};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::addr_of;

use spa_sys::spa_pod;

use pipewire_proc_macro::RawWrapper;

use crate::spa::pod::bitmap::PodBitmapRef;
use crate::spa::pod::bytes::PodBytesRef;
use crate::spa::pod::choice::PodChoiceRef;
use crate::spa::pod::id::PodIdRef;
use crate::spa::pod::iterator::PodValueIterator;
use crate::spa::pod::object::PodObjectRef;
use crate::spa::pod::restricted::{PodHeader, StaticTypePod};
use crate::spa::pod::sequence::PodSequenceRef;
use crate::spa::pod::string::PodStringRef;
use crate::spa::pod::struct_::PodStructRef;
use crate::spa::pod::{
    BasicType, BasicTypePod, BasicTypeValue, PodBoolRef, PodDoubleRef, PodError, PodFdRef,
    PodFloatRef, PodFractionRef, PodIntRef, PodLongRef, PodPointerRef, PodRectangleRef, PodRef,
    PodResult, PodValue, SizedPod, WritePod, WriteValue,
};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct PodArrayBodyRef {
    #[raw]
    raw: spa_sys::spa_pod_array_body,
}

impl PodArrayBodyRef {
    unsafe fn content_ptr(&self) -> *const u8 {
        (self.as_raw_ptr() as *const u8).offset(size_of::<PodArrayBodyRef>() as isize)
    }

    fn child(&self) -> &PodRef {
        unsafe { PodRef::from_raw_ptr(addr_of!(self.raw.child)) }
    }
}

#[repr(transparent)]
pub struct PodArrayRef<T: PodValue = PodRef> {
    raw: spa_sys::spa_pod_array,
    phantom: PhantomData<T>,
}

impl<T> crate::wrapper::RawWrapper for PodArrayRef<T>
where
    T: PodValue,
    T: BasicTypePod,
{
    type CType = spa_sys::spa_pod_array;

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
        &mut *(raw as *mut PodArrayRef<T>)
    }
}

impl<T> PodHeader for PodArrayRef<T>
where
    T: PodValue,
    T: BasicTypePod,
{
    fn pod_header(&self) -> &spa_pod {
        &self.raw.pod
    }
}

impl<T> StaticTypePod for PodArrayRef<T>
where
    T: PodValue,
    T: BasicTypePod,
{
    fn static_type() -> Type {
        Type::ARRAY
    }
}

impl<'a, T> PodValue for &'a PodArrayRef<T>
where
    T: PodValue,
    T: BasicTypePod,
{
    type Value = PodValueIterator<'a, T>;
    type RawValue = spa_sys::spa_pod_array_body;

    fn raw_value_ptr(&self) -> *const Self::RawValue {
        &self.raw.body
    }

    fn parse_raw_value(ptr: *const Self::RawValue, size: usize) -> PodResult<Self::Value> {
        let body = unsafe { PodArrayBodyRef::from_raw_ptr(ptr) };
        let size = size - size_of::<Self::RawValue>();
        let first_element_ptr = unsafe { body.content_ptr() };
        Ok(PodValueIterator::new(
            first_element_ptr.cast(),
            size,
            body.child().size() as usize,
        ))
    }

    fn value(&self) -> PodResult<Self::Value> {
        Self::parse_raw_value(self.raw_value_ptr(), self.pod_header().size as usize)
    }
}

impl<'a, T> WritePod for &'a PodArrayRef<T>
where
    T: PodValue,
    T: BasicTypePod,
{
    fn write_pod<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        todo!()
    }
}

impl<T> Debug for PodArrayRef<T>
where
    T: PodValue,
    T: BasicTypePod,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodArrayRef")
            .field("pod.type", &self.upcast().type_())
            .field("pod.size", &self.upcast().size())
            .field("body", &self.body())
            .field("value", &self.value().map(|i| i.collect::<Vec<_>>()))
            .finish()
    }
}

impl<T> PodArrayRef<T>
where
    T: PodValue,
    T: BasicTypePod,
{
    fn body(&self) -> &PodArrayBodyRef {
        unsafe { PodArrayBodyRef::from_raw_ptr(addr_of!(self.raw.body)) }
    }

    fn body_size(&self) -> usize {
        self.raw.pod.size as usize
    }

    fn elements(&self) -> u32 {
        ((self.body_size() - size_of::<PodArrayBodyRef>()) / self.raw.body.child.size as usize)
            as u32
    }
}
