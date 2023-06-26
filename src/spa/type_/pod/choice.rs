use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::addr_of;

use pipewire_macro_impl::enum_wrapper;
use pipewire_proc_macro::RawWrapper;

use crate::spa::type_::pod::id::PodIdRef;
use crate::spa::type_::pod::string::PodStringRef;
use crate::spa::type_::pod::{
    BasicTypePod, BasicTypeValue, PodBoolRef, PodDoubleRef, PodError, PodFloatRef, PodFractionRef,
    PodIntRef, PodLongRef, PodRectangleRef, PodRef, PodResult, PodValueParser, ReadablePod,
    SizedPod,
};
use crate::spa::type_::{PointRef, Type};
use crate::wrapper::RawWrapper;

enum_wrapper!(
    ChoiceType,
    spa_sys::spa_choice_type,
    NONE: spa_sys::SPA_CHOICE_None,
    RANGE: spa_sys::SPA_CHOICE_Range,
    STEP: spa_sys::SPA_CHOICE_Step,
    ENUM: spa_sys::SPA_CHOICE_Enum,
    FLAGS: spa_sys::SPA_CHOICE_Flags,
);

#[repr(u32)]
#[derive(Debug)]
pub enum ChoiceStructType<T>
where
    T: PodValueParser<*const u8>,
{
    NONE(T::To) = ChoiceType::NONE.raw,                      // value
    RANGE(T::To, T::To, T::To) = ChoiceType::RANGE.raw,      // (default, min, max)
    STEP(T::To, T::To, T::To, T::To) = ChoiceType::STEP.raw, // (default, min, max, step)
    ENUM(T::To, Vec<T::To>) = ChoiceType::ENUM.raw,          // (default, alternatives)
    FLAGS(T::To, Vec<T::To>) = ChoiceType::FLAGS.raw,        // (default, possible flags)
}

#[repr(u32)]
#[derive(Debug)]
pub enum ChoiceValueType<'a> {
    NONE() = Type::NONE.raw,
    BOOL(ChoiceStructType<PodBoolRef>) = Type::BOOL.raw,
    ID(ChoiceStructType<PodIdRef>) = Type::ID.raw,
    INT(ChoiceStructType<PodIntRef>) = Type::INT.raw,
    LONG(ChoiceStructType<PodLongRef>) = Type::LONG.raw,
    FLOAT(ChoiceStructType<PodFloatRef>) = Type::FLOAT.raw,
    DOUBLE(ChoiceStructType<PodDoubleRef>) = Type::DOUBLE.raw,
    STRING(ChoiceStructType<&'a PodStringRef>) = Type::STRING.raw,
    RECTANGLE(ChoiceStructType<PodRectangleRef>) = Type::RECTANGLE.raw,
    FRACTION(ChoiceStructType<PodFractionRef>) = Type::FRACTION.raw,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodChoiceBodyRef {
    #[raw]
    raw: spa_sys::spa_pod_choice_body,
}

impl PodChoiceBodyRef {
    fn type_(&self) -> ChoiceType {
        ChoiceType::from_raw(self.raw.type_)
    }

    fn flags(&self) -> u32 {
        self.raw.flags
    }

    fn child(&self) -> &PodRef {
        unsafe { PodRef::from_raw_ptr(addr_of!(self.raw.child)) }
    }

    unsafe fn content_ptr(&self) -> *const u8 {
        (self.as_raw_ptr() as *const u8).offset(size_of::<PodChoiceBodyRef>() as isize)
    }
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodChoiceRef {
    #[raw]
    raw: spa_sys::spa_pod_choice,
}

impl PodChoiceRef {
    fn body(&self) -> &PodChoiceBodyRef {
        unsafe { PodChoiceBodyRef::from_raw_ptr(addr_of!(self.raw.body)) }
    }
}

impl Debug for PodChoiceRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodChoiceRef")
            .field("pod", &self.upcast())
            .field("value", &self.value())
            .finish()
    }
}

impl<'a> ReadablePod for &'a PodChoiceRef {
    type Value = ChoiceValueType<'a>;

    fn value(&self) -> PodResult<Self::Value> {
        let content_size = self.size_bytes() - size_of::<PodChoiceRef>();
        Self::parse(content_size as u32, self.body())
    }
}

impl SizedPod for PodChoiceRef {
    fn size_bytes(&self) -> usize {
        self.upcast().size_bytes()
    }
}

impl BasicTypePod for PodChoiceRef {}

fn parse_choice<T>(size: u32, value: &PodChoiceBodyRef) -> PodResult<ChoiceStructType<T>>
where
    T: PodValueParser<*const u8>,
{
    let choice_type = value.type_();
    let mut iter: ChoiceIterator<T> = ChoiceIterator::new(value, size);
    match choice_type {
        ChoiceType::NONE => {
            let value = iter.next().ok_or(PodError::DataIsTooShort)?;
            if iter.next().is_some() {
                Err(PodError::UnexpectedChoiceElement)
            } else {
                Ok(ChoiceStructType::NONE(value))
            }
        }
        ChoiceType::RANGE => {
            let default = iter.next().ok_or(PodError::DataIsTooShort)?;
            let min = iter.next().ok_or(PodError::DataIsTooShort)?;
            let max = iter.next().ok_or(PodError::DataIsTooShort)?;
            if iter.next().is_some() {
                Err(PodError::UnexpectedChoiceElement)
            } else {
                Ok(ChoiceStructType::RANGE(default, min, max))
            }
        }
        ChoiceType::STEP => {
            let default = iter.next().ok_or(PodError::DataIsTooShort)?;
            let min = iter.next().ok_or(PodError::DataIsTooShort)?;
            let max = iter.next().ok_or(PodError::DataIsTooShort)?;
            let step = iter.next().ok_or(PodError::DataIsTooShort)?;
            if iter.next().is_some() {
                Err(PodError::UnexpectedChoiceElement)
            } else {
                Ok(ChoiceStructType::STEP(default, min, max, step))
            }
        }
        ChoiceType::ENUM => {
            let default = iter.next().ok_or(PodError::DataIsTooShort)?;
            let mut alternatives = Vec::new();
            iter.for_each(|a| alternatives.push(a));
            Ok(ChoiceStructType::ENUM(default, alternatives))
        }
        ChoiceType::FLAGS => {
            let default = iter.next().ok_or(PodError::DataIsTooShort)?;
            let mut alternatives = Vec::new();
            iter.for_each(|a| alternatives.push(a));
            Ok(ChoiceStructType::FLAGS(default, alternatives))
        }
        _ => Err(PodError::UnknownPodTypeToDowncast),
    }
}

impl<'a> PodValueParser<&'a PodChoiceBodyRef> for &'a PodChoiceRef {
    type To = ChoiceValueType<'a>;

    fn parse(size: u32, value: &'a PodChoiceBodyRef) -> PodResult<Self::To> {
        match value.child().type_() {
            Type::NONE => Ok(ChoiceValueType::NONE()),
            Type::BOOL => Ok(ChoiceValueType::BOOL(parse_choice(size, value)?)),
            Type::ID => Ok(ChoiceValueType::ID(parse_choice(size, value)?)),
            Type::INT => Ok(ChoiceValueType::INT(parse_choice(size, value)?)),
            Type::LONG => Ok(ChoiceValueType::LONG(parse_choice(size, value)?)),
            Type::FLOAT => Ok(ChoiceValueType::FLOAT(parse_choice(size, value)?)),
            Type::DOUBLE => Ok(ChoiceValueType::DOUBLE(parse_choice(size, value)?)),
            Type::STRING => Ok(ChoiceValueType::STRING(parse_choice(size, value)?)),
            Type::RECTANGLE => Ok(ChoiceValueType::RECTANGLE(parse_choice(size, value)?)),
            Type::FRACTION => Ok(ChoiceValueType::FRACTION(parse_choice(size, value)?)),
            _ => Err(PodError::UnsupportedChoiceElementType),
        }
    }
}

impl<'a> PodValueParser<*const u8> for &'a PodChoiceRef {
    type To = ChoiceValueType<'a>;

    fn parse(size: u32, value: *const u8) -> PodResult<Self::To> {
        unsafe { Self::parse(size, &*(value as *const PodChoiceBodyRef)) }
    }
}

pub struct ChoiceIterator<'a, T: PodValueParser<*const u8>> {
    body: &'a PodChoiceBodyRef,
    size: u32,
    first_element_ptr: *const u8,
    current_element_ptr: *const u8,
    phantom: PhantomData<T>,
}

impl<'a, T: PodValueParser<*const u8>> ChoiceIterator<'a, T> {
    pub fn new(body: &'a PodChoiceBodyRef, size: u32) -> Self {
        let first_element_ptr = unsafe { body.content_ptr() };
        Self {
            body,
            size,
            first_element_ptr,
            current_element_ptr: first_element_ptr,
            phantom: PhantomData::default(),
        }
    }

    unsafe fn inside(&self, ptr: *const u8) -> bool {
        let max_offset = self.size as isize;
        let offset = ptr.offset_from(self.first_element_ptr);
        offset < max_offset && (offset + self.body.raw.child.size as isize) <= max_offset
    }

    unsafe fn next_ptr(&self, ptr: *const u8) -> *const u8 {
        ptr.offset(self.body.raw.child.size as isize)
    }
}

impl<T: PodValueParser<*const u8>> Debug for ChoiceIterator<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChoiceIterator").finish()
    }
}

impl<'a, T: PodValueParser<*const u8>> Iterator for ChoiceIterator<'a, T> {
    type Item = T::To;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let current_element_ptr = self.current_element_ptr;
            if self.inside(current_element_ptr) {
                self.current_element_ptr = self.next_ptr(current_element_ptr);
                T::parse(self.body.raw.child.size, current_element_ptr).ok()
            } else {
                None
            }
        }
    }
}
