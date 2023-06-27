use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::addr_of;

use pipewire_proc_macro::RawWrapper;

use crate::spa::type_::pod::bitmap::PodBitmapRef;
use crate::spa::type_::pod::bytes::PodBytesRef;
use crate::spa::type_::pod::choice::PodChoiceRef;
use crate::spa::type_::pod::id::PodIdRef;
use crate::spa::type_::pod::iterator::PodValueIterator;
use crate::spa::type_::pod::object::PodObjectRef;
use crate::spa::type_::pod::sequence::PodSequenceRef;
use crate::spa::type_::pod::string::PodStringRef;
use crate::spa::type_::pod::struct_::PodStructRef;
use crate::spa::type_::pod::{
    BasicType, BasicTypePod, BasicTypeValue, PodBoolRef, PodDoubleRef, PodError, PodFdRef,
    PodFloatRef, PodFractionRef, PodIntRef, PodLongRef, PodPointerRef, PodRectangleRef, PodRef,
    PodResult, PodValueParser, ReadablePod, SizedPod,
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
pub struct PodArrayRef<T: PodValueParser<*const u8> = PodIdRef> {
    raw: spa_sys::spa_pod_array,
    phantom: PhantomData<T>,
}

impl<T> crate::wrapper::RawWrapper for PodArrayRef<T>
where
    T: PodValueParser<*const u8>,
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

impl<T> SizedPod for PodArrayRef<T>
where
    T: PodValueParser<*const u8>,
    T: BasicTypePod,
{
    fn pod_size(&self) -> usize {
        self.upcast().pod_size()
    }
}

impl<T> BasicTypePod for PodArrayRef<T>
where
    T: PodValueParser<*const u8>,
    T: BasicTypePod,
{
    fn static_type() -> Type {
        Type::ARRAY
    }
}

impl<'a, T> ReadablePod for &'a PodArrayRef<T>
where
    T: PodValueParser<*const u8>,
    T: BasicTypePod,
{
    type Value = PodValueIterator<'a, T>;

    fn value(&self) -> PodResult<Self::Value> {
        let content_size = self.pod_size() - size_of::<PodArrayRef>();
        Self::parse(content_size as u32, self.body())
    }
}

impl<T> Debug for PodArrayRef<T>
where
    T: PodValueParser<*const u8>,
    T: BasicTypePod,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodArrayRef")
            .field("pod", &self.upcast())
            .field("body", &self.body())
            .field("value", &self.value().map(|v| v.collect::<Vec<_>>()))
            .finish()
    }
}

impl<T> PodArrayRef<T>
where
    T: PodValueParser<*const u8>,
    T: BasicTypePod,
{
    fn body(&self) -> &PodArrayBodyRef {
        unsafe { PodArrayBodyRef::from_raw_ptr(addr_of!(self.raw.body)) }
    }

    fn elements(&self) -> u32 {
        self.raw.pod.size / self.raw.body.child.size
    }

    pub fn element(&self, index: u32) -> PodResult<T::To> {
        if T::static_type() != self.body().child().type_() {
            Err(PodError::WrongPodTypeToCast)
        } else if self.elements() >= index {
            Err(PodError::IndexIsOutOfRange)
        } else {
            let first_element_ptr: *const u8 = unsafe { self.body().content_ptr() };
            let ptr = unsafe {
                first_element_ptr.offset(index as isize * self.body().child().pod_size() as isize)
            };
            let size = self.raw.pod.size;
            T::parse(size, ptr)
        }
    }
}

impl<'a, T> PodValueParser<&'a PodArrayBodyRef> for &'a PodArrayRef<T>
where
    T: PodValueParser<*const u8>,
    T: BasicTypePod,
{
    type To = PodValueIterator<'a, T>;

    fn parse(s: u32, b: &'a PodArrayBodyRef) -> PodResult<Self::To> {
        unsafe {
            Ok(PodValueIterator::new(
                b.content_ptr().cast(),
                s as usize,
                b.child().size() as usize,
            ))
        }
    }
}

impl<'a, T> PodValueParser<*const u8> for &'a PodArrayRef<T>
where
    T: PodValueParser<*const u8>,
    T: BasicTypePod,
{
    type To = PodValueIterator<'a, T>;

    fn parse(size: u32, value: *const u8) -> PodResult<Self::To> {
        unsafe { Self::parse(size, &*(value as *const PodArrayBodyRef)) }
    }
}
