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
    BasicType, BasicTypeValue, Pod, PodBoolRef, PodDoubleRef, PodError, PodFdRef, PodFloatRef,
    PodFractionRef, PodIntRef, PodLongRef, PodPointerRef, PodRectangleRef, PodRef, PodResult,
    PodSubtype, PodValueParser, ReadablePod,
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
    T: PodSubtype,
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

impl<T> Pod for PodArrayRef<T>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
{
    fn pod_size(&self) -> usize {
        self.upcast().pod_size()
    }
}

impl<T> PodSubtype for PodArrayRef<T>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
{
    fn static_type() -> Type {
        Type::ARRAY
    }
}

impl<'a, T> ReadablePod for &'a PodArrayRef<T>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
{
    type Value = PodValueIterator<'a, T>;

    fn value(&self) -> PodResult<Self::Value> {
        Self::parse(self.body_size(), self.body())
    }
}

impl<T> Debug for PodArrayRef<T>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
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
    T: PodSubtype,
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

    pub fn element(&self, index: u32) -> PodResult<T::Value> {
        if T::static_type() != self.body().child().type_() {
            Err(PodError::WrongPodTypeToCast)
        } else if self.elements() >= index {
            Err(PodError::IndexIsOutOfRange)
        } else {
            let first_element_ptr: *const u8 = unsafe { self.body().content_ptr() };
            let ptr = unsafe {
                first_element_ptr.offset(index as isize * self.body().child().pod_size() as isize)
            };
            T::parse(self.body_size(), ptr)
        }
    }
}

impl<'a, T> PodValueParser<&'a PodArrayBodyRef> for &'a PodArrayRef<T>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
{
    fn parse(
        content_size: usize,
        header_or_value: &'a PodArrayBodyRef,
    ) -> PodResult<<Self as ReadablePod>::Value> {
        unsafe {
            Ok(PodValueIterator::new(
                header_or_value.content_ptr().cast(),
                content_size - size_of::<PodArrayBodyRef>(),
                header_or_value.child().size() as usize,
            ))
        }
    }
}

impl<'a, T> PodValueParser<*const u8> for &'a PodArrayRef<T>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
{
    fn parse(
        content_size: usize,
        header_or_value: *const u8,
    ) -> PodResult<<Self as ReadablePod>::Value> {
        unsafe { Self::parse(content_size, &*(header_or_value as *const PodArrayBodyRef)) }
    }
}
