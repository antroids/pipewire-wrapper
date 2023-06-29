use std::any::TypeId;
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
    NONE(Option<T::Value>) = ChoiceType::NONE.raw,
    RANGE(PodRangeValue<T::Value>) = ChoiceType::RANGE.raw,
    STEP(PodStepValue<T::Value>) = ChoiceType::STEP.raw,
    ENUM(PodEnumValue<T::Value>) = ChoiceType::ENUM.raw,
    FLAGS(PodFlagsValue<T::Value>) = ChoiceType::FLAGS.raw,
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
    pub fn type_(&self) -> ChoiceType {
        ChoiceType::from_raw(self.raw.type_)
    }

    pub fn flags(&self) -> u32 {
        self.raw.flags
    }

    pub fn child(&self) -> &PodRef {
        unsafe { PodRef::from_raw_ptr(addr_of!(self.raw.child)) }
    }

    pub(crate) unsafe fn content_ptr(&self) -> *const u8 {
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
    pub fn body(&self) -> &PodChoiceBodyRef {
        unsafe { PodChoiceBodyRef::from_raw_ptr(addr_of!(self.raw.body)) }
    }

    pub fn fixated(&self) -> bool {
        self.body().type_() == ChoiceType::NONE
    }

    fn content_size(&self) -> usize {
        self.raw.pod.size as usize - size_of::<PodChoiceRef>()
    }

    fn parse_choice<T>(&self) -> PodResult<ChoiceStructType<T>>
    where
        T: PodValueParser<*const u8>,
        T: PodSubtype,
    {
        let body = self.body();
        unsafe {
            match body.type_() {
                ChoiceType::NONE => {
                    let mut iter: PodValueIterator<T> = PodValueIterator::new(
                        unsafe { body.content_ptr().cast() },
                        self.content_size(),
                        body.child().size() as usize,
                    );
                    Ok(ChoiceStructType::NONE(iter.next()))
                }
                ChoiceType::RANGE => <PodRangeRef<T> as ReadablePod>::value(
                    PodRangeRef::from_raw_ptr(self.as_raw_ptr()),
                )
                .map(|r| ChoiceStructType::RANGE(r)),
                ChoiceType::STEP => <PodStepRef<T> as ReadablePod>::value(
                    PodStepRef::from_raw_ptr(self.as_raw_ptr()),
                )
                .map(|r| ChoiceStructType::STEP(r)),
                ChoiceType::ENUM => <PodEnumRef<T> as ReadablePod>::value(
                    PodEnumRef::from_raw_ptr(self.as_raw_ptr()),
                )
                .map(|r| ChoiceStructType::ENUM(r)),
                ChoiceType::FLAGS => <PodFlagsRef<T> as ReadablePod>::value(
                    PodFlagsRef::from_raw_ptr(self.as_raw_ptr()),
                )
                .map(|r| ChoiceStructType::FLAGS(r)),
                _ => Err(PodError::UnknownPodTypeToDowncast),
            }
        }
    }
}

impl Debug for PodChoiceRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodChoiceRef")
            .field("pod.type", &self.upcast().type_())
            .field("pod.size", &self.upcast().size())
            .field("value", &self.value())
            .finish()
    }
}

impl<'a> ReadablePod for &'a PodChoiceRef {
    type Value = ChoiceValueType<'a>;

    fn value(&self) -> PodResult<Self::Value> {
        match self.body().child().type_() {
            Type::NONE => Ok(ChoiceValueType::NONE()),
            Type::BOOL => Ok(ChoiceValueType::BOOL(self.parse_choice()?)),
            Type::ID => Ok(ChoiceValueType::ID(self.parse_choice()?)),
            Type::INT => Ok(ChoiceValueType::INT(self.parse_choice()?)),
            Type::LONG => Ok(ChoiceValueType::LONG(self.parse_choice()?)),
            Type::FLOAT => Ok(ChoiceValueType::FLOAT(self.parse_choice()?)),
            Type::DOUBLE => Ok(ChoiceValueType::DOUBLE(self.parse_choice()?)),
            //Type::STRING => Ok(ChoiceValueType::STRING(self.parse_choice(size, value)?)),
            Type::RECTANGLE => Ok(ChoiceValueType::RECTANGLE(self.parse_choice()?)),
            Type::FRACTION => Ok(ChoiceValueType::FRACTION(self.parse_choice()?)),
            _ => Err(PodError::UnsupportedChoiceElementType),
        }
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
