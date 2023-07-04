use std::any::TypeId;
use std::fmt::{Debug, Formatter};
use std::io::{Seek, Write};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::addr_of;

use spa_sys::spa_pod;

use pipewire_macro_impl::enum_wrapper;
use pipewire_proc_macro::RawWrapper;

use crate::spa::type_::pod::choice::enum_::{PodEnumRef, PodEnumValue};
use crate::spa::type_::pod::choice::flags::{PodFlagsRef, PodFlagsValue};
use crate::spa::type_::pod::choice::none::PodNoneRef;
use crate::spa::type_::pod::choice::range::{PodRangeRef, PodRangeValue};
use crate::spa::type_::pod::choice::step::{PodStepRef, PodStepValue};
use crate::spa::type_::pod::id::PodIdRef;
use crate::spa::type_::pod::iterator::PodValueIterator;
use crate::spa::type_::pod::restricted::{PodHeader, StaticTypePod};
use crate::spa::type_::pod::string::PodStringRef;
use crate::spa::type_::pod::{
    BasicTypePod, BasicTypeValue, PodBoolRef, PodDoubleRef, PodError, PodFloatRef, PodFractionRef,
    PodIntRef, PodLongRef, PodRectangleRef, PodRef, PodResult, PodValueParser, ReadablePod,
    SizedPod, WritablePod, WritableValue,
};
use crate::spa::type_::{PointRef, Type};
use crate::wrapper::RawWrapper;

pub mod enum_;
pub mod flags;
pub mod none;
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
pub enum ChoiceValueType {
    NONE() = Type::NONE.raw,
    BOOL(ChoiceStructType<PodBoolRef>) = Type::BOOL.raw,
    ID(ChoiceStructType<PodIdRef>) = Type::ID.raw,
    INT(ChoiceStructType<PodIntRef>) = Type::INT.raw,
    LONG(ChoiceStructType<PodLongRef>) = Type::LONG.raw,
    FLOAT(ChoiceStructType<PodFloatRef>) = Type::FLOAT.raw,
    DOUBLE(ChoiceStructType<PodDoubleRef>) = Type::DOUBLE.raw,
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
        T: BasicTypePod,
    {
        let body = self.body();
        unsafe {
            match body.type_() {
                ChoiceType::NONE => <PodNoneRef<T> as ReadablePod>::value(
                    PodNoneRef::from_raw_ptr(self.as_raw_ptr()),
                )
                .map(|r| ChoiceStructType::NONE(r)),
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

    fn write_raw_body<W>(
        buffer: &mut W,
        choice_type: ChoiceType,
        flags: u32,
        child_size: u32,
        child_type: Type,
    ) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        <&Self>::write_value(
            buffer,
            &spa_sys::spa_pod_choice_body {
                type_: choice_type.raw,
                flags,
                child: spa_pod {
                    size: child_size as u32,
                    type_: child_type.raw,
                },
            },
        )
    }

    fn write_pod<W, T>(buffer: &mut W, value: &ChoiceStructType<T>) -> PodResult<usize>
    where
        W: Write + Seek,
        T: WritableValue,
        T: PodValueParser<*const u8>,
        T: StaticTypePod,
    {
        match value {
            ChoiceStructType::NONE(val) => PodNoneRef::<T>::write_pod(buffer, val),
            ChoiceStructType::RANGE(val) => PodRangeRef::<T>::write_pod(buffer, val),
            ChoiceStructType::STEP(val) => PodStepRef::<T>::write_pod(buffer, val),
            ChoiceStructType::ENUM(val) => PodEnumRef::<T>::write_pod(buffer, val),
            ChoiceStructType::FLAGS(val) => PodFlagsRef::<T>::write_pod(buffer, val),
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
    type Value = ChoiceValueType;

    fn value(&self) -> PodResult<Self::Value> {
        match self.body().child().type_() {
            Type::NONE => Ok(ChoiceValueType::NONE()),
            Type::BOOL => Ok(ChoiceValueType::BOOL(self.parse_choice()?)),
            Type::ID => Ok(ChoiceValueType::ID(self.parse_choice()?)),
            Type::INT => Ok(ChoiceValueType::INT(self.parse_choice()?)),
            Type::LONG => Ok(ChoiceValueType::LONG(self.parse_choice()?)),
            Type::FLOAT => Ok(ChoiceValueType::FLOAT(self.parse_choice()?)),
            Type::DOUBLE => Ok(ChoiceValueType::DOUBLE(self.parse_choice()?)),
            Type::RECTANGLE => Ok(ChoiceValueType::RECTANGLE(self.parse_choice()?)),
            Type::FRACTION => Ok(ChoiceValueType::FRACTION(self.parse_choice()?)),
            _ => Err(PodError::UnsupportedChoiceElementType),
        }
    }
}

impl<'a> WritablePod for &'a PodChoiceRef {
    fn write_pod<W>(buffer: &mut W, value: &<Self as ReadablePod>::Value) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        match value {
            ChoiceValueType::NONE() => {
                Ok(Self::write_header(
                    buffer,
                    size_of::<spa_sys::spa_pod_choice_body>() as u32,
                    PodChoiceRef::static_type(),
                )? + PodChoiceRef::write_raw_body(buffer, ChoiceType::NONE, 0, 0, Type::NONE)?)
            }
            ChoiceValueType::BOOL(v) => PodChoiceRef::write_pod(buffer, v),
            ChoiceValueType::ID(v) => PodChoiceRef::write_pod(buffer, v),
            ChoiceValueType::INT(v) => PodChoiceRef::write_pod(buffer, v),
            ChoiceValueType::LONG(v) => PodChoiceRef::write_pod(buffer, v),
            ChoiceValueType::FLOAT(v) => PodChoiceRef::write_pod(buffer, v),
            ChoiceValueType::DOUBLE(v) => PodChoiceRef::write_pod(buffer, v),
            ChoiceValueType::RECTANGLE(v) => PodChoiceRef::write_pod(buffer, v),
            ChoiceValueType::FRACTION(v) => PodChoiceRef::write_pod(buffer, v),
        }
    }
}

impl PodHeader for PodChoiceRef {
    fn pod_header(&self) -> &spa_pod {
        &self.raw.pod
    }
}

impl StaticTypePod for PodChoiceRef {
    fn static_type() -> Type {
        Type::CHOICE
    }
}
