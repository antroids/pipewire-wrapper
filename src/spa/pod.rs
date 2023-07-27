/*
 * SPDX-License-Identifier: MIT
 */
use std::ffi::{c_char, CStr};
use std::fmt::{Debug, Formatter};
use std::io::{Cursor, Seek, SeekFrom, Write};
use std::marker::PhantomData;
use std::mem::{size_of, size_of_val};
use std::ops::{Deref, Rem};
use std::os::fd::RawFd;
use std::ptr::addr_of;
use std::{mem, slice};

use spa_sys::{spa_pod, spa_pod_bool};

use array::{PodArrayBodyRef, PodArrayRef};
use bitmap::PodBitmapRef;
use bytes::PodBytesRef;
use choice::PodChoiceRef;
use id::PodIdRef;
use pipewire_wrapper_proc_macro::RawWrapper;
use restricted::PodRawValue;
use sequence::PodSequenceRef;
use string::PodStringRef;
use struct_::PodStructRef;

use crate::spa::pod::choice::{ChoiceType, ChoiceValueType};
use crate::spa::pod::object::prop::Prop;
use crate::spa::pod::object::PodObjectRef;
use crate::spa::pod::pod_buf::{AllocatedData, PodBuf};
use crate::spa::pod::pointer::PodPointerRef;
use crate::spa::pod::restricted::{
    BasicTypePod, CloneTo, PodHeader, PrimitiveValue, SizedPod, WritePod, WriteValue,
};
use crate::spa::type_::{FractionRef, RectangleRef, Type};
use crate::wrapper::RawWrapper;

use self::restricted::write_header;

pub mod array;
pub mod bitmap;
pub mod bytes;
pub mod choice;
pub mod control;
pub mod id;
pub mod iterator;
pub mod object;
pub mod pointer;
pub mod sequence;
pub mod string;
pub mod struct_;

pub mod pod_buf;
mod restricted;

macro_rules! primitive_type_pod_impl {
    ($pod_ref_type:ty, $pod_type:expr, $value_raw_type:ty) => {
        primitive_type_pod_impl!(
            $pod_ref_type,
            $pod_type,
            $value_raw_type,
            $value_raw_type,
            v,
            v
        );
    };

    ($pod_ref_type:ty, $pod_type:expr, $value_raw_type:ty, $value_type:ty) => {
        primitive_type_pod_impl!($pod_ref_type, $pod_type, $value_raw_type, $value_type, v, v);
    };

    ($pod_ref_type:ty, $pod_type:expr, $value_raw_type:ty, $value_type:ty, $value_ident:ident, $convert_value_expr:expr) => {
        impl restricted::PrimitiveValue for $pod_ref_type {}

        impl PodRawValue for $pod_ref_type {
            type RawValue = $value_raw_type;

            fn parse_raw_value(ptr: *const Self::RawValue, size: usize) -> PodResult<Self::Value> {
                if size < size_of::<$value_raw_type>() {
                    Err(PodError::DataIsTooShort(size_of::<$value_raw_type>(), size))
                } else {
                    let $value_ident = unsafe { *ptr };
                    Ok($convert_value_expr)
                }
            }

            fn raw_value_ptr(&self) -> *const Self::RawValue {
                &self.raw.value
            }
        }

        impl PodValue for $pod_ref_type {
            type Value = $value_type;

            fn value(&self) -> PodResult<Self::Value> {
                Self::parse_raw_value(&self.raw_value(), self.raw.pod.size as usize)
            }
        }

        impl WritePod for $pod_ref_type {
            fn write_pod<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<()>
            where
                W: Write + Seek,
            {
                restricted::write_header(
                    buffer,
                    size_of::<$value_raw_type>() as u32,
                    <$pod_ref_type>::static_type(),
                )?;
                Self::write_raw_value(buffer, value)?;
                restricted::write_align_padding(buffer)?;
                Ok(())
            }
        }

        impl WriteValue for $pod_ref_type {
            fn write_raw_value<W>(
                buffer: &mut W,
                value: &<Self as PodValue>::Value,
            ) -> PodResult<()>
            where
                W: Write + Seek,
            {
                let value: $value_raw_type = (*value).into();
                restricted::write_value(buffer, &value)?;
                Ok(())
            }
        }

        impl PodHeader for $pod_ref_type {
            fn pod_header(&self) -> &spa_sys::spa_pod {
                &self.raw.pod
            }

            fn static_type() -> Type {
                $pod_type
            }
        }

        impl $pod_ref_type {
            pub fn raw_value(&self) -> $value_raw_type {
                self.raw.value
            }
        }

        impl Debug for $pod_ref_type {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                unsafe {
                    f.debug_struct(stringify!($pod_ref_type))
                        .field("pod.type", &self.pod_type())
                        .field("pod.size", &self.pod_size())
                        .field("value", &self.value())
                        .finish()
                }
            }
        }
    };
}

pub enum PodError {
    DataIsTooShort(usize, usize),
    UnknownPodTypeToDowncast,
    WrongPodTypeToCast(Type, Type),
    StringIsNotNullTerminated,
    IndexIsOutOfRange,
    ChoiceElementMissing,
    UnexpectedChoiceElement,
    UnexpectedChoiceElementSize(usize, usize),
    UnsupportedChoiceElementType,
    UnexpectedControlType(u32),
    UnexpectedObjectType(u32),
    UnexpectedChoiceType(ChoiceType, ChoiceType),
    IOError(std::io::Error),
    PodIsNotAligned,
}

impl From<PodError> for crate::Error {
    fn from(value: PodError) -> Self {
        Self::PodParseError(value)
    }
}

impl From<std::io::Error> for PodError {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}

impl Debug for PodError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PodError::DataIsTooShort(expected, actual) => write!(
                f,
                "POD data size is too small for that type, expected {} > actual {}",
                expected, actual
            ),
            PodError::UnknownPodTypeToDowncast => {
                write!(f, "Cannot downcast Pod, child type is unknown")
            }
            PodError::WrongPodTypeToCast(cast_type, actual_type) => write!(
                f,
                "Cannot cast Pod with type {:?} to type {:?}",
                actual_type, cast_type
            ),
            PodError::StringIsNotNullTerminated => write!(f, "String is not null terminated"),
            PodError::IndexIsOutOfRange => write!(f, "Index is out of range"),
            PodError::ChoiceElementMissing => {
                write!(f, "Cannot find required element for choice type")
            }
            PodError::UnexpectedChoiceElement => write!(f, "Unexpected element in the choice type"),
            PodError::UnexpectedChoiceElementSize(expected, actual) => write!(
                f,
                "Unexpected choice element size, expected {} != actual {}",
                expected, actual
            ),
            PodError::UnsupportedChoiceElementType => write!(f, "Unsupported choice element type"),
            PodError::UnexpectedControlType(type_) => {
                write!(f, "Unexpected control type {}", type_)
            }
            PodError::UnexpectedObjectType(type_) => {
                write!(f, "Unexpected object type {}", type_)
            }
            PodError::UnexpectedChoiceType(expected, actual) => {
                write!(
                    f,
                    "Unexpected choice type, expected {:?} actual {:?}",
                    expected, actual
                )
            }
            PodError::IOError(err) => {
                write!(f, "IOError {:?}", err)
            }
            PodError::PodIsNotAligned => {
                write!(f, "Pod is not aligned!")
            }
        }
    }
}

type PodResult<T> = Result<T, PodError>;

const POD_ALIGN: usize = 8;

pub trait PodValue {
    type Value: Debug;
    fn value(&self) -> PodResult<Self::Value>;
}

pub trait FromValue<'a>
where
    Self: 'a,
    Self: Sized,
    &'a Self: WritePod,
{
    fn from_value(value: &<&'a Self as PodValue>::Value) -> PodResult<AllocatedData<Self>>;
}

pub trait FromPrimitiveValue
where
    Self: Sized,
    Self: WritePod,
{
    fn from_primitive(value: <Self as PodValue>::Value) -> PodResult<AllocatedData<Self>>;
}

impl<'a, T> FromValue<'a> for T
where
    T: Sized,
    T: 'a,
    &'a T: WritePod,
{
    fn from_value(value: &<&'a Self as PodValue>::Value) -> PodResult<AllocatedData<Self>> {
        Ok(PodBuf::<Self>::from_value(value)?.into_pod())
    }
}

impl<T> FromPrimitiveValue for T
where
    T: Sized,
    T: WritePod,
{
    fn from_primitive(value: <Self as PodValue>::Value) -> PodResult<AllocatedData<Self>> {
        Ok(PodBuf::<Self>::from_primitive_value(value)?.into_pod())
    }
}

impl<'a, T> PodValue for &'a T
where
    T: PodRawValue,
    T: PrimitiveValue,
{
    type Value = T::Value;
    fn value(&self) -> PodResult<Self::Value> {
        (*self).value()
    }
}

impl<'a, T> WritePod for &'a T
where
    T: WritePod,
    T: PrimitiveValue,
{
    fn write_pod<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<()>
    where
        W: Write + Seek,
    {
        T::write_pod(buffer, value)
    }
}

impl<'a, T> WriteValue for &'a T
where
    T: WriteValue,
    T: PrimitiveValue,
{
    fn write_raw_value<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<()>
    where
        W: Write + Seek,
    {
        T::write_raw_value(buffer, value)
    }
}

impl<T: PodHeader> SizedPod for T {
    fn pod_size(&self) -> usize {
        self.pod_header().size as usize + size_of::<spa_sys::spa_pod>()
    }
}

impl<T: PodHeader> CloneTo for &T {
    fn clone_to(&self, buffer: &mut impl Write) -> PodResult<()> {
        let size = self.pod_size();
        let slice = unsafe { slice::from_raw_parts(*self as *const T as *const u8, size) };
        buffer.write_all(slice)?;
        let rem = size.rem(POD_ALIGN);
        if rem > 0 {
            let padding_len = POD_ALIGN - rem;
            let padding = vec![0u8; padding_len];
            buffer.write_all(padding.as_slice())?;
        }
        Ok(())
    }
}

impl<T> BasicTypePod for T
where
    T: PodHeader,
    T: RawWrapper,
    T: Debug,
{
}

pub trait Upcast {
    fn upcast(&self) -> &PodRef;
}

impl<'a, T: BasicTypePod> Upcast for &'a T {
    fn upcast(&self) -> &PodRef {
        unsafe { PodRef::from_raw_ptr(self.pod_header()) }
    }
}

#[derive(RawWrapper, Clone)]
#[repr(transparent)]
pub struct PodRef {
    #[raw]
    raw: spa_sys::spa_pod,
}

impl PodRef {
    pub fn size(&self) -> u32 {
        self.raw.size
    }

    pub fn type_(&self) -> Type {
        Type::from_raw(self.raw.type_)
    }

    pub fn downcast(&self) -> PodResult<BasicType> {
        unsafe {
            match self.pod_type() {
                Type::NONE => Ok(BasicType::NONE),
                Type::BOOL => self.cast().map(BasicType::BOOL),
                Type::ID => self.cast().map(BasicType::ID),
                Type::INT => self.cast().map(BasicType::INT),
                Type::LONG => self.cast().map(BasicType::LONG),
                Type::FLOAT => self.cast().map(BasicType::FLOAT),
                Type::DOUBLE => self.cast().map(BasicType::DOUBLE),
                Type::STRING => self.cast().map(BasicType::STRING),
                Type::BYTES => self.cast().map(BasicType::BYTES),
                Type::RECTANGLE => self.cast().map(BasicType::RECTANGLE),
                Type::FRACTION => self.cast().map(BasicType::FRACTION),
                Type::BITMAP => self.cast().map(BasicType::BITMAP),
                Type::ARRAY => self.cast().map(BasicType::ARRAY),
                Type::STRUCT => self.cast().map(BasicType::STRUCT),
                Type::OBJECT => self.cast().map(BasicType::OBJECT),
                Type::SEQUENCE => self.cast().map(BasicType::SEQUENCE),
                Type::POINTER => self.cast().map(BasicType::POINTER),
                Type::FD => self.cast().map(BasicType::FD),
                Type::CHOICE => self.cast().map(BasicType::CHOICE),
                Type::POD => self.cast().map(BasicType::POD),
                _ => Err(PodError::UnknownPodTypeToDowncast),
            }
        }
    }
}

impl PodHeader for PodRef {
    fn pod_header(&self) -> &spa_pod {
        &self.raw
    }

    fn static_type() -> Type {
        Type::POD
    }
}

impl PodValue for PodRef {
    type Value = ();
    fn value(&self) -> PodResult<Self::Value> {
        Ok(())
    }
}

impl<'a> PodValue for &'a PodRef {
    type Value = BasicTypeValue<'a>;
    fn value(&self) -> PodResult<Self::Value> {
        Self::parse_raw_value(self.as_raw_ptr(), self.size() as usize)
    }
}

impl<'a> WritePod for &'a PodRef {
    fn write_pod<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<()>
    where
        W: Write + Seek,
    {
        match value {
            BasicTypeValue::NONE => write_header(buffer, 0, Type::NONE),
            BasicTypeValue::BOOL(v) => PodBoolRef::write_pod(buffer, v),
            BasicTypeValue::ID(v) => PodIdRef::write_pod(buffer, v),
            BasicTypeValue::INT(v) => PodIntRef::write_pod(buffer, v),
            BasicTypeValue::LONG(v) => PodLongRef::write_pod(buffer, v),
            BasicTypeValue::FLOAT(v) => PodFloatRef::write_pod(buffer, v),
            BasicTypeValue::DOUBLE(v) => PodDoubleRef::write_pod(buffer, v),
            BasicTypeValue::STRING(v) => <&PodStringRef>::write_pod(buffer, v),
            BasicTypeValue::BYTES(v) => <&PodBytesRef>::write_pod(buffer, v),
            BasicTypeValue::RECTANGLE(v) => PodRectangleRef::write_pod(buffer, v),
            BasicTypeValue::FRACTION(v) => PodFractionRef::write_pod(buffer, v),
            BasicTypeValue::BITMAP(v) => <&PodBitmapRef>::write_pod(buffer, v),
            BasicTypeValue::ARRAY(v) => <&PodArrayRef>::write_pod(buffer, v),
            BasicTypeValue::STRUCT(v) => <&PodStructRef>::write_pod(buffer, v),
            BasicTypeValue::OBJECT(v) => <&PodObjectRef>::write_pod(buffer, v),
            BasicTypeValue::SEQUENCE(v) => <&PodSequenceRef>::write_pod(buffer, v),
            BasicTypeValue::POINTER(v) => <&PodPointerRef>::write_pod(buffer, v),
            BasicTypeValue::FD(v) => PodFdRef::write_pod(buffer, v),
            BasicTypeValue::CHOICE(v) => PodChoiceRef::<PodRef>::write_choice_value(buffer, v),
            BasicTypeValue::POD(v) => Err(PodError::UnknownPodTypeToDowncast),
        }
    }
}

impl<'a, T> PodRawValue for &'a T
where
    T: PodRawValue,
    T: PrimitiveValue,
{
    type RawValue = T::RawValue;

    fn raw_value_ptr(&self) -> *const Self::RawValue {
        (*self).raw_value_ptr()
    }

    fn parse_raw_value(ptr: *const Self::RawValue, size: usize) -> PodResult<Self::Value> {
        T::parse_raw_value(ptr, size)
    }
}

impl PodRawValue for PodRef {
    type RawValue = spa_sys::spa_pod;

    fn raw_value_ptr(&self) -> *const Self::RawValue {
        &self.raw
    }

    fn parse_raw_value(ptr: *const Self::RawValue, size: usize) -> PodResult<Self::Value> {
        Ok(())
    }
}

impl<'a> PodRawValue for &'a PodRef {
    type RawValue = spa_sys::spa_pod;

    fn raw_value_ptr(&self) -> *const Self::RawValue {
        &self.raw
    }

    fn parse_raw_value(ptr: *const Self::RawValue, _size: usize) -> PodResult<Self::Value> {
        Ok(match unsafe { PodRef::from_raw_ptr(ptr).downcast()? } {
            BasicType::NONE => BasicTypeValue::NONE,
            BasicType::BOOL(pod) => BasicTypeValue::BOOL(pod.value()?),
            BasicType::ID(pod) => BasicTypeValue::ID(pod.value()?),
            BasicType::INT(pod) => BasicTypeValue::INT(pod.value()?),
            BasicType::LONG(pod) => BasicTypeValue::LONG(pod.value()?),
            BasicType::FLOAT(pod) => BasicTypeValue::FLOAT(pod.value()?),
            BasicType::DOUBLE(pod) => BasicTypeValue::DOUBLE(pod.value()?),
            BasicType::STRING(pod) => BasicTypeValue::STRING(pod.value()?),
            BasicType::BYTES(pod) => BasicTypeValue::BYTES(pod.value()?),
            BasicType::RECTANGLE(pod) => BasicTypeValue::RECTANGLE(pod.value()?),
            BasicType::FRACTION(pod) => BasicTypeValue::FRACTION(pod.value()?),
            BasicType::BITMAP(pod) => BasicTypeValue::BITMAP(pod.value()?),
            BasicType::ARRAY(pod) => BasicTypeValue::ARRAY(pod.value()?),
            BasicType::STRUCT(pod) => BasicTypeValue::STRUCT(pod.value()?),
            BasicType::OBJECT(pod) => BasicTypeValue::OBJECT(pod.value()?),
            BasicType::SEQUENCE(pod) => BasicTypeValue::SEQUENCE(pod.value()?),
            BasicType::POINTER(pod) => BasicTypeValue::POINTER(pod.value()?),
            BasicType::FD(pod) => BasicTypeValue::FD(pod.value()?),
            BasicType::CHOICE(pod) => BasicTypeValue::CHOICE(pod.choice_value()?),
            _ => return Err(PodError::UnknownPodTypeToDowncast),
        })
    }
}

impl Debug for PodRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodRef")
            .field("size", &self.size())
            .field("type", &self.type_())
            .field("value", &self.value())
            .finish()
    }
}

#[repr(u32)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum BasicType<'a> {
    NONE = Type::NONE.raw,
    BOOL(&'a PodBoolRef) = Type::BOOL.raw,
    ID(&'a PodIdRef) = Type::ID.raw,
    INT(&'a PodIntRef) = Type::INT.raw,
    LONG(&'a PodLongRef) = Type::LONG.raw,
    FLOAT(&'a PodFloatRef) = Type::FLOAT.raw,
    DOUBLE(&'a PodDoubleRef) = Type::DOUBLE.raw,
    STRING(&'a PodStringRef) = Type::STRING.raw,
    BYTES(&'a PodBytesRef) = Type::BYTES.raw,
    RECTANGLE(&'a PodRectangleRef) = Type::RECTANGLE.raw,
    FRACTION(&'a PodFractionRef) = Type::FRACTION.raw,
    BITMAP(&'a PodBitmapRef) = Type::BITMAP.raw,
    ARRAY(&'a PodArrayRef) = Type::ARRAY.raw,
    STRUCT(&'a PodStructRef) = Type::STRUCT.raw,
    OBJECT(&'a PodObjectRef) = Type::OBJECT.raw,
    SEQUENCE(&'a PodSequenceRef) = Type::SEQUENCE.raw,
    POINTER(&'a PodPointerRef) = Type::POINTER.raw,
    FD(&'a PodFdRef) = Type::FD.raw,
    CHOICE(&'a PodChoiceRef) = Type::CHOICE.raw,
    POD(&'a PodRef) = Type::POD.raw,
}

#[repr(u32)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum BasicTypeValue<'a> {
    NONE = Type::NONE.raw,
    BOOL(<PodBoolRef as PodValue>::Value) = Type::BOOL.raw,
    ID(<PodIdRef as PodValue>::Value) = Type::ID.raw,
    INT(<PodIntRef as PodValue>::Value) = Type::INT.raw,
    LONG(<PodLongRef as PodValue>::Value) = Type::LONG.raw,
    FLOAT(<PodFloatRef as PodValue>::Value) = Type::FLOAT.raw,
    DOUBLE(<PodDoubleRef as PodValue>::Value) = Type::DOUBLE.raw,
    STRING(<&'a PodStringRef as PodValue>::Value) = Type::STRING.raw,
    BYTES(<&'a PodBytesRef as PodValue>::Value) = Type::BYTES.raw,
    RECTANGLE(<PodRectangleRef as PodValue>::Value) = Type::RECTANGLE.raw,
    FRACTION(<PodFractionRef as PodValue>::Value) = Type::FRACTION.raw,
    BITMAP(<&'a PodBitmapRef as PodValue>::Value) = Type::BITMAP.raw,
    ARRAY(<&'a PodArrayRef as PodValue>::Value) = Type::ARRAY.raw,
    STRUCT(<&'a PodStructRef as PodValue>::Value) = Type::STRUCT.raw,
    OBJECT(<&'a PodObjectRef as PodValue>::Value) = Type::OBJECT.raw,
    SEQUENCE(<&'a PodSequenceRef as PodValue>::Value) = Type::SEQUENCE.raw,
    POINTER(<&'a PodPointerRef as PodValue>::Value) = Type::POINTER.raw,
    FD(<PodFdRef as PodValue>::Value) = Type::FD.raw,
    CHOICE(ChoiceValueType) = Type::CHOICE.raw,
    POD(&'a PodRef) = Type::POD.raw,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodBoolRef {
    #[raw]
    raw: spa_sys::spa_pod_bool,
}

primitive_type_pod_impl!(PodBoolRef, Type::BOOL, i32, bool, v, v != 0);

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodIntRef {
    #[raw]
    raw: spa_sys::spa_pod_int,
}

primitive_type_pod_impl!(PodIntRef, Type::INT, i32);

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodLongRef {
    #[raw]
    raw: spa_sys::spa_pod_long,
}

primitive_type_pod_impl!(PodLongRef, Type::LONG, i64);

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodFloatRef {
    #[raw]
    raw: spa_sys::spa_pod_float,
}

primitive_type_pod_impl!(PodFloatRef, Type::FLOAT, f32);

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodDoubleRef {
    #[raw]
    raw: spa_sys::spa_pod_double,
}

primitive_type_pod_impl!(PodDoubleRef, Type::DOUBLE, f64);

#[derive(RawWrapper, Copy, Clone)]
#[repr(transparent)]
pub struct PodRectangleRef {
    #[raw]
    raw: spa_sys::spa_pod_rectangle,
}

primitive_type_pod_impl!(
    PodRectangleRef,
    Type::RECTANGLE,
    spa_sys::spa_rectangle,
    RectangleRef,
    v,
    RectangleRef::from_raw(v)
);

#[derive(RawWrapper, Copy, Clone)]
#[repr(transparent)]
pub struct PodFractionRef {
    #[raw]
    raw: spa_sys::spa_pod_fraction,
}

primitive_type_pod_impl!(
    PodFractionRef,
    Type::FRACTION,
    spa_sys::spa_fraction,
    FractionRef,
    v,
    FractionRef::from_raw(v)
);

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodFdRef {
    #[raw]
    raw: spa_sys::spa_pod_fd,
}

primitive_type_pod_impl!(PodFdRef, Type::FD, i64, RawFd, v, v as RawFd);
