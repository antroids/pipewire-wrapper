use std::any::TypeId;
use std::fmt::{Debug, Formatter};
use std::io::{Seek, Write};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::addr_of;

use spa_sys::spa_pod;

use pipewire_proc_macro::RawWrapper;

use crate::enum_wrapper;
use crate::spa::pod::choice::enum_::{PodEnumRef, PodEnumValue};
use crate::spa::pod::choice::flags::{PodFlagsRef, PodFlagsValue};
use crate::spa::pod::choice::none::PodNoneRef;
use crate::spa::pod::choice::range::{PodRangeRef, PodRangeValue};
use crate::spa::pod::choice::step::{PodStepRef, PodStepValue};
use crate::spa::pod::id::{PodIdRef, PodIdType};
use crate::spa::pod::iterator::PodValueIterator;
use crate::spa::pod::object::format::AudioFormat;
use crate::spa::pod::object::PodObjectRef;
use crate::spa::pod::pod_buf::AllocatedData;
use crate::spa::pod::restricted::{PodHeader, StaticTypePod};
use crate::spa::pod::string::PodStringRef;
use crate::spa::pod::{
    BasicType, BasicTypePod, BasicTypeValue, FromPrimitiveValue, PodBoolRef, PodDoubleRef,
    PodError, PodFloatRef, PodFractionRef, PodIntRef, PodLongRef, PodRawValue, PodRectangleRef,
    PodRef, PodResult, PodValue, SizedPod, Upcast, WritePod, WriteValue,
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
    T: PodRawValue,
{
    NONE(T::Value) = ChoiceType::NONE.raw,
    RANGE(PodRangeValue<T::Value>) = ChoiceType::RANGE.raw,
    STEP(PodStepValue<T::Value>) = ChoiceType::STEP.raw,
    ENUM(PodEnumValue<T::Value>) = ChoiceType::ENUM.raw,
    FLAGS(PodFlagsValue<T::Value>) = ChoiceType::FLAGS.raw,
    /// Any Pod could be here, at least primitives.
    /// This value is used to be able to parse Format as EnumFormat without errors.
    /// It's possible to force object to parse value as format by call
    /// [param_value](PodObjectRef::param_value)
    VALUE(T::Value),
}

impl<T> ChoiceStructType<T>
where
    T: PodRawValue,
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

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodChoiceRef<T: PodRawValue = PodRef> {
    #[raw]
    raw: spa_sys::spa_pod_choice,
    phantom: PhantomData<T>,
}

impl<T> PodChoiceRef<T>
where
    T: PodRawValue,
{
    pub(crate) fn body(&self) -> &PodChoiceBodyRef {
        if !self.value_choice() {
            unsafe { PodChoiceBodyRef::from_raw_ptr(addr_of!(self.raw.body)) }
        } else {
            panic!("Choice with VALUE type cannot have body");
        }
    }

    pub fn value_choice(&self) -> bool {
        self.pod_type() != Type::CHOICE
    }

    pub fn fixated(&self) -> bool {
        !self.value_choice() && self.body().type_() == ChoiceType::NONE
    }

    fn content_size(&self) -> usize {
        self.raw.pod.size as usize - size_of::<PodChoiceRef<T>>()
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

fn parse_choice<P>(
    ptr: *const spa_sys::spa_pod_choice_body,
    size: usize,
) -> PodResult<ChoiceStructType<P>>
where
    P: PodRawValue,
    P: StaticTypePod,
{
    match unsafe { PodChoiceBodyRef::from_raw_ptr(ptr).type_() } {
        ChoiceType::NONE => <PodNoneRef<P> as PodRawValue>::parse_raw_value(ptr, size)
            .map(|r| ChoiceStructType::NONE(r)),
        ChoiceType::RANGE => <PodRangeRef<P> as PodRawValue>::parse_raw_value(ptr, size)
            .map(|r| ChoiceStructType::RANGE(r)),
        ChoiceType::STEP => <PodStepRef<P> as PodRawValue>::parse_raw_value(ptr, size)
            .map(|r| ChoiceStructType::STEP(r)),
        ChoiceType::ENUM => <PodEnumRef<P> as PodRawValue>::parse_raw_value(ptr, size)
            .map(|r| ChoiceStructType::ENUM(r)),
        ChoiceType::FLAGS => <PodFlagsRef<P> as PodRawValue>::parse_raw_value(ptr, size)
            .map(|r| ChoiceStructType::FLAGS(r)),
        _ => Err(PodError::UnknownPodTypeToDowncast),
    }
}

impl PodChoiceRef<PodRef> {
    pub fn choice_value(&self) -> PodResult<ChoiceValueType> {
        if !self.value_choice() {
            let size = self.pod_header().size as usize;
            let body_ptr = self.body().as_raw_ptr();
            Ok(match self.body().child().type_() {
                Type::BOOL => ChoiceValueType::BOOL(parse_choice(body_ptr, size)?),
                Type::ID => ChoiceValueType::ID(parse_choice(body_ptr, size)?),
                Type::INT => ChoiceValueType::INT(parse_choice(body_ptr, size)?),
                Type::LONG => ChoiceValueType::LONG(parse_choice(body_ptr, size)?),
                Type::FLOAT => ChoiceValueType::FLOAT(parse_choice(body_ptr, size)?),
                Type::DOUBLE => ChoiceValueType::DOUBLE(parse_choice(body_ptr, size)?),
                Type::RECTANGLE => ChoiceValueType::RECTANGLE(parse_choice(body_ptr, size)?),
                Type::FRACTION => ChoiceValueType::FRACTION(parse_choice(body_ptr, size)?),
                type_ => return Err(PodError::UnsupportedChoiceElementType),
            })
        } else {
            let size = self.pod_size();
            let body_ptr = self.raw_value_ptr();
            Ok(match self.pod_type() {
                Type::BOOL => ChoiceValueType::BOOL(ChoiceStructType::VALUE(
                    PodBoolRef::parse_raw_value(body_ptr.cast(), size)?,
                )),
                Type::ID => ChoiceValueType::ID(ChoiceStructType::VALUE(
                    PodIdRef::parse_raw_value(body_ptr.cast(), size)?,
                )),
                Type::INT => ChoiceValueType::INT(ChoiceStructType::VALUE(
                    PodIntRef::parse_raw_value(body_ptr.cast(), size)?,
                )),
                Type::LONG => ChoiceValueType::LONG(ChoiceStructType::VALUE(
                    PodLongRef::parse_raw_value(body_ptr.cast(), size)?,
                )),
                Type::FLOAT => ChoiceValueType::FLOAT(ChoiceStructType::VALUE(
                    PodFloatRef::parse_raw_value(body_ptr.cast(), size)?,
                )),
                Type::DOUBLE => ChoiceValueType::DOUBLE(ChoiceStructType::VALUE(
                    PodDoubleRef::parse_raw_value(body_ptr.cast(), size)?,
                )),
                Type::RECTANGLE => ChoiceValueType::RECTANGLE(ChoiceStructType::VALUE(
                    PodRectangleRef::parse_raw_value(body_ptr.cast(), size)?,
                )),
                Type::FRACTION => ChoiceValueType::FRACTION(ChoiceStructType::VALUE(
                    PodFractionRef::parse_raw_value(body_ptr.cast(), size)?,
                )),
                type_ => return Err(PodError::UnsupportedChoiceElementType),
            })
        }
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
    T: PodRawValue,
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

impl<'a, T> PodRawValue for &'a PodChoiceRef<T>
where
    T: PodRawValue,
    T: BasicTypePod,
{
    type RawValue = spa_sys::spa_pod_choice_body;

    fn raw_value_ptr(&self) -> *const Self::RawValue {
        &self.raw.body
    }

    fn parse_raw_value(ptr: *const Self::RawValue, size: usize) -> PodResult<Self::Value> {
        parse_choice(ptr, size)
    }
}

impl<'a, T> PodValue for &'a PodChoiceRef<T>
where
    T: PodRawValue,
    T: BasicTypePod,
{
    type Value = ChoiceStructType<T>;
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
    T: PodRawValue,
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

impl<T: PodRawValue> PodHeader for PodChoiceRef<T> {
    fn pod_header(&self) -> &spa_pod {
        &self.raw.pod
    }
}

impl<T: PodRawValue> StaticTypePod for PodChoiceRef<T> {
    fn static_type() -> Type {
        Type::CHOICE
    }
}

impl<'a, T: WriteValue> From<&'a T> for &'a PodChoiceRef<T> {
    fn from(value: &'a T) -> Self {
        // Extremely unsafe, choice should be able to parse any pod
        unsafe { PodChoiceRef::from_raw_ptr(value as *const T as *const _) }
    }
}

#[test]
fn test_enum_choice() {
    let enum_choice = PodEnumValue::new(5, vec![1, 2, 3])
        .to_alloc_pod::<PodIntRef>()
        .unwrap();
    let enum_choice_pod = enum_choice.as_pod();

    assert_eq!(enum_choice_pod.pod_type(), Type::CHOICE);
    assert!(!enum_choice_pod.choice().value_choice());
    assert!(!enum_choice_pod.choice().fixated());
    if let ChoiceStructType::ENUM(val) = enum_choice_pod.choice().value().unwrap() {
        assert_eq!(val.default(), &5);
        assert_eq!(val.alternatives(), &vec![1, 2, 3]);
    } else {
        assert!(false)
    }
    if let BasicType::CHOICE(choice) = enum_choice_pod.upcast().downcast().unwrap() {
        if let ChoiceValueType::INT(ChoiceStructType::ENUM(val)) = choice.choice_value().unwrap() {
            assert_eq!(val.default(), &5);
            assert_eq!(val.alternatives(), &vec![1, 2, 3]);
        } else {
            assert!(false)
        }
    } else {
        assert!(false)
    }
}

#[test]
fn test_flags_choice() {
    let flags_choice = PodFlagsValue::new(5, vec![1, 2, 3])
        .to_alloc_pod::<PodIntRef>()
        .unwrap();
    let flags_choice_pod = flags_choice.as_pod();

    assert_eq!(flags_choice_pod.pod_type(), Type::CHOICE);
    assert!(!flags_choice_pod.choice().value_choice());
    assert!(!flags_choice_pod.choice().fixated());
    if let ChoiceStructType::FLAGS(val) = flags_choice_pod.choice().value().unwrap() {
        assert_eq!(val.default(), &5);
        assert_eq!(val.alternatives(), &vec![1, 2, 3]);
    } else {
        assert!(false)
    }
    if let BasicType::CHOICE(choice) = flags_choice_pod.upcast().downcast().unwrap() {
        if let ChoiceValueType::INT(ChoiceStructType::FLAGS(val)) = choice.choice_value().unwrap() {
            assert_eq!(val.default(), &5);
            assert_eq!(val.alternatives(), &vec![1, 2, 3]);
        } else {
            assert!(false)
        }
    } else {
        assert!(false)
    }
}

#[test]
fn test_range_choice() {
    let range_choice = PodRangeValue::new(5, 1, 10)
        .to_alloc_pod::<PodIntRef>()
        .unwrap();
    let range_choice_pod = range_choice.as_pod();

    assert_eq!(range_choice_pod.pod_type(), Type::CHOICE);
    assert!(!range_choice_pod.choice().value_choice());
    assert!(!range_choice_pod.choice().fixated());
    if let ChoiceStructType::RANGE(val) = range_choice_pod.choice().value().unwrap() {
        assert_eq!(val.default(), &5);
        assert_eq!(val.min(), &1);
        assert_eq!(val.max(), &10);
    } else {
        assert!(false)
    }
    if let BasicType::CHOICE(choice) = range_choice_pod.upcast().downcast().unwrap() {
        if let ChoiceValueType::INT(ChoiceStructType::RANGE(val)) = choice.choice_value().unwrap() {
            assert_eq!(val.default(), &5);
            assert_eq!(val.min(), &1);
            assert_eq!(val.max(), &10);
        } else {
            assert!(false)
        }
    } else {
        assert!(false)
    }
}

#[test]
fn test_step_choice() {
    let step_choice = PodStepValue::new(5, 1, 10, 2)
        .to_alloc_pod::<PodIntRef>()
        .unwrap();
    let step_choice_pod = step_choice.as_pod();

    assert_eq!(step_choice_pod.pod_type(), Type::CHOICE);
    assert!(!step_choice_pod.choice().value_choice());
    assert!(!step_choice_pod.choice().fixated());
    if let ChoiceStructType::STEP(val) = step_choice_pod.choice().value().unwrap() {
        assert_eq!(val.default(), &5);
        assert_eq!(val.min(), &1);
        assert_eq!(val.max(), &10);
        assert_eq!(val.step(), &2);
    } else {
        assert!(false)
    }
    if let BasicType::CHOICE(choice) = step_choice_pod.upcast().downcast().unwrap() {
        if let ChoiceValueType::INT(ChoiceStructType::STEP(val)) = choice.choice_value().unwrap() {
            assert_eq!(val.default(), &5);
            assert_eq!(val.min(), &1);
            assert_eq!(val.max(), &10);
            assert_eq!(val.step(), &2);
        } else {
            assert!(false)
        }
    } else {
        assert!(false)
    }
}

#[test]
fn test_none_choice() {
    let none_choice = PodNoneRef::from_primitive(123).unwrap();
    let none_choice_pod: &PodNoneRef<PodIntRef> = none_choice.as_pod();

    assert_eq!(none_choice_pod.pod_type(), Type::CHOICE);
    assert!(!none_choice_pod.choice().value_choice());
    assert!(none_choice_pod.choice().fixated());
    if let ChoiceStructType::NONE(val) = none_choice_pod.choice().value().unwrap() {
        assert_eq!(val, 123);
    } else {
        assert!(false)
    }
    if let BasicType::CHOICE(choice) = none_choice_pod.upcast().downcast().unwrap() {
        if let ChoiceValueType::INT(ChoiceStructType::NONE(val)) = choice.choice_value().unwrap() {
            assert_eq!(val, 123);
        } else {
            assert!(false)
        }
    } else {
        assert!(false)
    }
}

#[test]
fn test_value_choice() {
    let pod = AudioFormat::F32.as_alloc_pod();
    let choice: &PodChoiceRef<PodIdRef<AudioFormat>> = pod.as_pod().into();

    assert_eq!(choice.pod_type(), Type::ID);
    assert!(choice.value_choice());
    assert!(!choice.fixated());
    if let ChoiceStructType::VALUE(val) = choice.value().unwrap() {
        assert_eq!(val, AudioFormat::F32);
    } else {
        assert!(false)
    }
    if let BasicType::ID(val) = choice.upcast().downcast().unwrap() {
        assert_eq!(val.value().unwrap(), AudioFormat::F32.raw);
    } else {
        assert!(false)
    }
}
