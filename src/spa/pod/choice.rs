use std::any::TypeId;
use std::fmt::{Debug, Formatter};
use std::io::{Seek, Write};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::addr_of;

use spa_sys::spa_pod;

use pipewire_macro_impl::enum_wrapper;
use pipewire_proc_macro::RawWrapper;

use crate::spa::pod::choice::enum_::{PodEnumRef, PodEnumValue};
use crate::spa::pod::choice::flags::{PodFlagsRef, PodFlagsValue};
use crate::spa::pod::choice::none::PodNoneRef;
use crate::spa::pod::choice::range::{PodRangeRef, PodRangeValue};
use crate::spa::pod::choice::step::{PodStepRef, PodStepValue};
use crate::spa::pod::id::PodIdRef;
use crate::spa::pod::iterator::PodValueIterator;
use crate::spa::pod::restricted::{PodHeader, StaticTypePod};
use crate::spa::pod::string::PodStringRef;
use crate::spa::pod::{
    BasicTypePod, BasicTypeValue, PodBoolRef, PodDoubleRef, PodError, PodFloatRef, PodFractionRef,
    PodIntRef, PodLongRef, PodRectangleRef, PodRef, PodResult, PodValue, SizedPod, WritePod,
    WriteValue,
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
    T: PodValue,
{
    NONE(T::Value) = ChoiceType::NONE.raw,
    RANGE(PodRangeValue<T::Value>) = ChoiceType::RANGE.raw,
    STEP(PodStepValue<T::Value>) = ChoiceType::STEP.raw,
    ENUM(PodEnumValue<T::Value>) = ChoiceType::ENUM.raw,
    FLAGS(PodFlagsValue<T::Value>) = ChoiceType::FLAGS.raw,
    VALUE(T::Value),
}

impl<T> ChoiceStructType<T>
where
    T: PodValue,
{
    pub fn default(&self) -> &T::Value {
        match self {
            ChoiceStructType::NONE(v) => v,
            ChoiceStructType::RANGE(v) => v.default(),
            ChoiceStructType::STEP(v) => v.default(),
            ChoiceStructType::ENUM(v) => v.default(),
            ChoiceStructType::FLAGS(v) => v.default(),
            ChoiceStructType::VALUE(v) => v,
        }
    }
}

#[repr(u32)]
#[derive(Debug)]
// Used by PodRef as value for choice, because generic type is unknown
pub enum ChoiceValueType {
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

    pub(crate) fn child(&self) -> &PodRef {
        unsafe { PodRef::from_raw_ptr(addr_of!(self.raw.child)) }
    }

    pub(crate) unsafe fn content_ptr(&self) -> *const u8 {
        (self.as_raw_ptr() as *const u8).offset(size_of::<PodChoiceBodyRef>() as isize)
    }
}

#[repr(transparent)]
pub struct PodChoiceRef<T: PodValue = PodRef> {
    raw: spa_sys::spa_pod_choice,
    phantom: PhantomData<T>,
}

impl<T: PodValue> RawWrapper for PodChoiceRef<T> {
    type CType = spa_sys::spa_pod_choice;

    fn as_raw_ptr(&self) -> *mut Self::CType {
        &self.raw as *const _ as *mut _
    }

    fn as_raw(&self) -> &Self::CType {
        &self.raw
    }

    fn from_raw(raw: Self::CType) -> Self {
        Self {
            raw,
            phantom: Default::default(),
        }
    }

    unsafe fn mut_from_raw_ptr<'a>(raw: *mut Self::CType) -> &'a mut Self {
        &mut *(raw as *mut PodChoiceRef<T>)
    }
}

impl<T> PodChoiceRef<T>
where
    T: PodValue,
{
    pub(crate) fn body(&self) -> &PodChoiceBodyRef {
        unsafe { PodChoiceBodyRef::from_raw_ptr(addr_of!(self.raw.body)) }
    }

    pub fn value_choice(&self) -> bool {
        self.pod_type() != Type::CHOICE
    }

    pub fn fixated(&self) -> bool {
        self.body().type_() == ChoiceType::NONE
    }

    fn content_size(&self) -> usize {
        self.raw.pod.size as usize - size_of::<PodChoiceRef<T>>()
    }

    fn parse_choice(
        ptr: *const spa_sys::spa_pod_choice_body,
        size: usize,
    ) -> PodResult<ChoiceStructType<T>>
    where
        T: PodValue,
        T: StaticTypePod,
    {
        match unsafe { PodChoiceBodyRef::from_raw_ptr(ptr).type_() } {
            ChoiceType::NONE => <PodNoneRef<T> as PodValue>::parse_raw_value(ptr, size)
                .map(|r| ChoiceStructType::NONE(r)),
            ChoiceType::RANGE => <PodRangeRef<T> as PodValue>::parse_raw_value(ptr, size)
                .map(|r| ChoiceStructType::RANGE(r)),
            ChoiceType::STEP => <PodStepRef<T> as PodValue>::parse_raw_value(ptr, size)
                .map(|r| ChoiceStructType::STEP(r)),
            ChoiceType::ENUM => <PodEnumRef<T> as PodValue>::parse_raw_value(ptr, size)
                .map(|r| ChoiceStructType::ENUM(r)),
            ChoiceType::FLAGS => <PodFlagsRef<T> as PodValue>::parse_raw_value(ptr, size)
                .map(|r| ChoiceStructType::FLAGS(r)),
            _ => Err(PodError::UnknownPodTypeToDowncast),
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
        T: BasicTypePod,
        T: WriteValue,
        T: WritePod,
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

    fn write_pod<W>(buffer: &mut W, value: &ChoiceStructType<T>) -> PodResult<usize>
    where
        W: Write + Seek,
        T: WriteValue,
        T: WritePod,
        T: BasicTypePod,
        T: StaticTypePod,
    {
        match value {
            ChoiceStructType::NONE(val) => PodNoneRef::<T>::write_pod(buffer, val),
            ChoiceStructType::RANGE(val) => PodRangeRef::<T>::write_pod(buffer, val),
            ChoiceStructType::STEP(val) => PodStepRef::<T>::write_pod(buffer, val),
            ChoiceStructType::ENUM(val) => PodEnumRef::<T>::write_pod(buffer, val),
            ChoiceStructType::FLAGS(val) => PodFlagsRef::<T>::write_pod(buffer, val),
            ChoiceStructType::VALUE(val) => T::write_pod(buffer, val),
        }
    }
}

impl PodChoiceRef<PodRef> {
    pub fn choice_value(&self) -> PodResult<ChoiceValueType> {
        let size = self.pod_header().size as usize;
        let body_ptr = self.body().as_raw_ptr();
        Ok(match self.body().child().type_() {
            Type::BOOL => {
                ChoiceValueType::BOOL(PodChoiceRef::<PodBoolRef>::parse_choice(body_ptr, size)?)
            }
            Type::ID => {
                ChoiceValueType::ID(PodChoiceRef::<PodIdRef>::parse_choice(body_ptr, size)?)
            }
            Type::INT => {
                ChoiceValueType::INT(PodChoiceRef::<PodIntRef>::parse_choice(body_ptr, size)?)
            }
            Type::LONG => {
                ChoiceValueType::LONG(PodChoiceRef::<PodLongRef>::parse_choice(body_ptr, size)?)
            }
            Type::FLOAT => {
                ChoiceValueType::FLOAT(PodChoiceRef::<PodFloatRef>::parse_choice(body_ptr, size)?)
            }
            Type::DOUBLE => {
                ChoiceValueType::DOUBLE(PodChoiceRef::<PodDoubleRef>::parse_choice(body_ptr, size)?)
            }
            Type::RECTANGLE => ChoiceValueType::RECTANGLE(
                PodChoiceRef::<PodRectangleRef>::parse_choice(body_ptr, size)?,
            ),
            Type::FRACTION => ChoiceValueType::FRACTION(
                PodChoiceRef::<PodFractionRef>::parse_choice(body_ptr, size)?,
            ),
            type_ => return Err(PodError::UnsupportedChoiceElementType),
        })
    }

    pub fn write_choice_value<W>(buffer: &mut W, value: &ChoiceValueType) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        match value {
            ChoiceValueType::BOOL(v) => PodChoiceRef::<PodBoolRef>::write_pod(buffer, v),
            ChoiceValueType::ID(v) => PodChoiceRef::<PodIdRef>::write_pod(buffer, v),
            ChoiceValueType::INT(v) => PodChoiceRef::<PodIntRef>::write_pod(buffer, v),
            ChoiceValueType::LONG(v) => PodChoiceRef::<PodLongRef>::write_pod(buffer, v),
            ChoiceValueType::FLOAT(v) => PodChoiceRef::<PodFloatRef>::write_pod(buffer, v),
            ChoiceValueType::DOUBLE(v) => PodChoiceRef::<PodDoubleRef>::write_pod(buffer, v),
            ChoiceValueType::RECTANGLE(v) => PodChoiceRef::<PodRectangleRef>::write_pod(buffer, v),
            ChoiceValueType::FRACTION(v) => PodChoiceRef::<PodFractionRef>::write_pod(buffer, v),
        }
    }
}

impl<T> Debug for PodChoiceRef<T>
where
    T: PodValue,
    T: BasicTypePod,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodChoiceRef")
            .field("pod.type", &self.pod_type())
            .field("pod.size", &self.pod_size())
            .field("value", &self.value())
            .finish()
    }
}

impl<'a, T> PodValue for &'a PodChoiceRef<T>
where
    T: PodValue,
    T: BasicTypePod,
{
    type Value = ChoiceStructType<T>;
    type RawValue = spa_sys::spa_pod_choice_body;

    fn raw_value_ptr(&self) -> *const Self::RawValue {
        &self.raw.body
    }

    fn parse_raw_value(ptr: *const Self::RawValue, size: usize) -> PodResult<Self::Value> {
        PodChoiceRef::<T>::parse_choice(ptr, size)
    }

    fn value(&self) -> PodResult<Self::Value> {
        let value_ptr = self.raw_value_ptr();
        let value_size = self.pod_header().size as usize;
        let pod_type = self.pod_type();
        // Sometimes we can use Choice in place of primitives to specify possible values for given format
        // VALUE choice type is added to handle this.
        // There is also NONE choice type, but it's not used as single value for some reason
        if pod_type == PodChoiceRef::<T>::static_type() {
            Self::parse_raw_value(value_ptr, value_size)
        } else if pod_type == T::static_type() {
            Ok(ChoiceStructType::VALUE(T::parse_raw_value(
                value_ptr.cast(),
                value_size,
            )?))
        } else {
            Err(PodError::UnsupportedChoiceElementType)
        }
    }
}

impl<'a, T> WritePod for &'a PodChoiceRef<T>
where
    T: PodValue,
    T: BasicTypePod,
    T: WriteValue,
    T: WritePod,
{
    fn write_pod<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        PodChoiceRef::write_pod(buffer, value)
    }
}

impl<T: PodValue> PodHeader for PodChoiceRef<T> {
    fn pod_header(&self) -> &spa_pod {
        &self.raw.pod
    }
}

impl<T: PodValue> StaticTypePod for PodChoiceRef<T> {
    fn static_type() -> Type {
        Type::CHOICE
    }
}
