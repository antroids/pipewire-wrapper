use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::addr_of;

use pipewire_macro_impl::enum_wrapper;
use pipewire_proc_macro::RawWrapper;

use crate::spa::type_::pod::choice::enum_::{PodEnumRef, PodEnumValue};
use crate::spa::type_::pod::choice::flags::{PodFlagsRef, PodFlagsValue};
use crate::spa::type_::pod::choice::range::{PodRangeRef, PodRangeValue};
use crate::spa::type_::pod::choice::step::{PodStepRef, PodStepValue};
use crate::spa::type_::pod::id::PodIdRef;
use crate::spa::type_::pod::iterator::PodValueIterator;
use crate::spa::type_::pod::string::PodStringRef;
use crate::spa::type_::pod::{
    BasicTypeValue, Pod, PodBoolRef, PodDoubleRef, PodError, PodFloatRef, PodFractionRef,
    PodIntRef, PodLongRef, PodRectangleRef, PodRef, PodResult, PodSubtype, PodValueParser,
    ReadablePod,
};
use crate::spa::type_::{PointRef, Type};
use crate::wrapper::RawWrapper;

pub mod enum_;
pub mod flags;
pub mod range;
pub mod step;

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
    NONE(Option<T::Value>) = ChoiceType::NONE.raw, // value
    RANGE(PodRangeValue<T::Value>) = ChoiceType::RANGE.raw, // (default, min, max)
    STEP(PodStepValue<T::Value>) = ChoiceType::STEP.raw, // (default, min, max, step)
    ENUM(PodEnumValue<T::Value>) = ChoiceType::ENUM.raw, // (default, alternatives)
    FLAGS(PodFlagsValue<T::Value>) = ChoiceType::FLAGS.raw, // (default, possible flags)
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

    fn content_size(&self) -> usize {
        self.raw.pod.size as usize
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
        Self::parse(self.content_size(), self.body())
    }
}

impl Pod for PodChoiceRef {
    fn pod_size(&self) -> usize {
        self.upcast().pod_size()
    }
}

impl PodSubtype for PodChoiceRef {
    fn static_type() -> Type {
        Type::CHOICE
    }
}

fn parse_choice<T>(s: usize, value: &PodChoiceBodyRef) -> PodResult<ChoiceStructType<T>>
where
    T: PodValueParser<*const u8>,
    T: PodSubtype,
{
    let s = s - size_of::<PodChoiceBodyRef>();
    match value.type_() {
        ChoiceType::NONE => {
            let mut iter: PodValueIterator<T> = PodValueIterator::new(
                unsafe { value.content_ptr().cast() },
                s,
                value.child().size() as usize,
            );
            Ok(ChoiceStructType::NONE(iter.next()))
        }
        ChoiceType::RANGE => PodRangeRef::<T>::parse(s, value).map(|r| ChoiceStructType::RANGE(r)),
        ChoiceType::STEP => PodStepRef::<T>::parse(s, value).map(|r| ChoiceStructType::STEP(r)),
        ChoiceType::ENUM => PodEnumRef::<T>::parse(s, value).map(|r| ChoiceStructType::ENUM(r)),
        ChoiceType::FLAGS => PodFlagsRef::<T>::parse(s, value).map(|r| ChoiceStructType::FLAGS(r)),
        _ => Err(PodError::UnknownPodTypeToDowncast),
    }
}

impl<'a> PodValueParser<&'a PodChoiceBodyRef> for &'a PodChoiceRef {
    fn parse(s: usize, v: &'a PodChoiceBodyRef) -> PodResult<<Self as ReadablePod>::Value> {
        match v.child().type_() {
            Type::NONE => Ok(ChoiceValueType::NONE()),
            Type::BOOL => Ok(ChoiceValueType::BOOL(parse_choice(s, v)?)),
            Type::ID => Ok(ChoiceValueType::ID(parse_choice(s, v)?)),
            Type::INT => Ok(ChoiceValueType::INT(parse_choice(s, v)?)),
            Type::LONG => Ok(ChoiceValueType::LONG(parse_choice(s, v)?)),
            Type::FLOAT => Ok(ChoiceValueType::FLOAT(parse_choice(s, v)?)),
            Type::DOUBLE => Ok(ChoiceValueType::DOUBLE(parse_choice(s, v)?)),
            //Type::STRING => Ok(ChoiceValueType::STRING(parse_choice(size, value)?)),
            Type::RECTANGLE => Ok(ChoiceValueType::RECTANGLE(parse_choice(s, v)?)),
            Type::FRACTION => Ok(ChoiceValueType::FRACTION(parse_choice(s, v)?)),
            _ => Err(PodError::UnsupportedChoiceElementType),
        }
    }
}

impl<'a> PodValueParser<*const u8> for &'a PodChoiceRef {
    fn parse(
        content_size: usize,
        header_or_value: *const u8,
    ) -> PodResult<<Self as ReadablePod>::Value> {
        unsafe { Self::parse(content_size, &*(header_or_value as *const PodChoiceBodyRef)) }
    }
}
