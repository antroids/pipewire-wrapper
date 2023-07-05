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
use pipewire_proc_macro::RawWrapper;
use sequence::PodSequenceRef;
use string::PodStringRef;
use struct_::PodStructRef;

use crate::spa::type_::object::PodObjectRef;
use crate::spa::type_::pod::choice::ChoiceType;
use crate::spa::type_::pod::object::prop::Prop;
use crate::spa::type_::pod::pod_buf::{AllocatedData, PodBuf};
use crate::spa::type_::pod::pointer::PodPointerRef;
use crate::spa::type_::pod::restricted::{
    BasicTypePod, CloneTo, PodHeader, PrimitiveValue, SizedPod, StaticTypePod, WritePod, WriteValue,
};
use crate::spa::type_::{FractionRef, RectangleRef, Type};
use crate::wrapper::RawWrapper;

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

        impl PodValue for $pod_ref_type {
            type Value = $value_type;
            type RawValue = $value_raw_type;

            fn raw_value_ptr(&self) -> *const Self::RawValue {
                &self.raw.value
            }

            fn parse_raw_value(ptr: *const Self::RawValue, size: usize) -> PodResult<Self::Value> {
                if size < size_of::<$value_raw_type>() {
                    Err(PodError::DataIsTooShort(size_of::<$value_raw_type>(), size))
                } else {
                    let $value_ident = unsafe { *ptr };
                    Ok($convert_value_expr)
                }
            }

            fn value(&self) -> PodResult<Self::Value> {
                Self::parse_raw_value(&self.raw_value(), self.raw.pod.size as usize)
            }
        }

        impl WritePod for $pod_ref_type {
            fn write_pod<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<usize>
            where
                W: Write + Seek,
            {
                Ok(Self::write_header(
                    buffer,
                    size_of::<$value_raw_type>() as u32,
                    <$pod_ref_type>::static_type(),
                )? + Self::write_raw_value(buffer, value)?
                    + Self::write_align_padding(buffer)?)
            }
        }

        impl WriteValue for $pod_ref_type {
            fn write_raw_value<W>(
                buffer: &mut W,
                value: &<Self as PodValue>::Value,
            ) -> PodResult<usize>
            where
                W: Write + Seek,
            {
                let value: $value_raw_type = (*value).into();
                Ok(Self::write_value(buffer, &value)?)
            }
        }

        impl PodHeader for $pod_ref_type {
            fn pod_header(&self) -> &spa_sys::spa_pod {
                &self.raw.pod
            }
        }

        impl StaticTypePod for $pod_ref_type {
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
                        .field("pod.type", &self.upcast().type_())
                        .field("pod.size", &self.upcast().size())
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
    type RawValue;

    fn raw_value_ptr(&self) -> *const Self::RawValue;

    fn parse_raw_value(ptr: *const Self::RawValue, size: usize) -> PodResult<Self::Value>;

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

impl<'a, T> PodValue for &'a T
where
    T: PodValue,
    T: PrimitiveValue,
{
    type Value = T::Value;
    type RawValue = T::RawValue;

    fn raw_value_ptr(&self) -> *const Self::RawValue {
        (*self).raw_value_ptr()
    }

    fn parse_raw_value(ptr: *const Self::RawValue, size: usize) -> PodResult<Self::Value> {
        T::parse_raw_value(ptr, size)
    }

    fn value(&self) -> PodResult<Self::Value> {
        (*self).value()
    }
}

impl<'a, T> WritePod for &'a T
where
    T: WritePod,
    T: PrimitiveValue,
{
    fn write_pod<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<usize>
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
    fn write_raw_value<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<usize>
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
    fn clone_to(&self, buffer: &mut impl Write) -> PodResult<usize> {
        let size = self.pod_size();
        let slice = unsafe { slice::from_raw_parts(*self as *const T as *const u8, size) };
        buffer.write_all(slice)?;
        let rem = size.rem(POD_ALIGN);
        if rem > 0 {
            let padding_len = POD_ALIGN - rem;
            let padding = vec![0u8; padding_len];
            buffer.write_all(padding.as_slice())?;
            Ok(padding_len)
        } else {
            Ok(0)
        }
    }
}

impl<T> BasicTypePod for T
where
    T: StaticTypePod,
    T: PodHeader,
    T: RawWrapper,
    T: Debug,
{
}

#[derive(RawWrapper)]
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
            match self.upcast().type_() {
                Type::NONE => Ok(BasicType::NONE(self.upcast())),
                Type::BOOL => self.cast().map(|r| BasicType::BOOL(r)),
                Type::ID => self.cast().map(|r| BasicType::ID(r)),
                Type::INT => self.cast().map(|r| BasicType::INT(r)),
                Type::LONG => self.cast().map(|r| BasicType::LONG(r)),
                Type::FLOAT => self.cast().map(|r| BasicType::FLOAT(r)),
                Type::DOUBLE => self.cast().map(|r| BasicType::DOUBLE(r)),
                Type::STRING => self.cast().map(|r| BasicType::STRING(r)),
                Type::BYTES => self.cast().map(|r| BasicType::BYTES(r)),
                Type::RECTANGLE => self.cast().map(|r| BasicType::RECTANGLE(r)),
                Type::FRACTION => self.cast().map(|r| BasicType::FRACTION(r)),
                Type::BITMAP => self.cast().map(|r| BasicType::BITMAP(r)),
                Type::ARRAY => self.cast().map(|r| BasicType::ARRAY(r)),
                Type::STRUCT => self.cast().map(|r| BasicType::STRUCT(r)),
                Type::OBJECT => self.cast().map(|r| BasicType::OBJECT(r)),
                Type::SEQUENCE => self.cast().map(|r| BasicType::SEQUENCE(r)),
                Type::POINTER => self.cast().map(|r| BasicType::POINTER(r)),
                Type::FD => self.cast().map(|r| BasicType::FD(r)),
                Type::CHOICE => self.cast().map(|r| BasicType::CHOICE(r)),
                Type::POD => self.cast().map(|r| BasicType::POD(r)),
                _ => Err(PodError::UnknownPodTypeToDowncast),
            }
        }
    }
}

impl PodHeader for PodRef {
    fn pod_header(&self) -> &spa_pod {
        &self.raw
    }
}

impl<'a> PodValue for &'a PodRef {
    type Value = BasicType<'a>;
    type RawValue = spa_sys::spa_pod;

    fn raw_value_ptr(&self) -> *const Self::RawValue {
        &self.raw
    }

    fn parse_raw_value(ptr: *const Self::RawValue, _size: usize) -> PodResult<Self::Value> {
        unsafe { PodRef::from_raw_ptr(ptr).downcast() }
    }

    fn value(&self) -> PodResult<Self::Value> {
        Self::parse_raw_value(self.as_raw_ptr(), self.size() as usize)
    }
}

impl<'a> WritePod for &'a PodRef {
    fn write_pod<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        match value {
            BasicType::NONE(pod) => pod.clone_to(buffer),
            BasicType::BOOL(pod) => pod.clone_to(buffer),
            BasicType::ID(pod) => pod.clone_to(buffer),
            BasicType::INT(pod) => pod.clone_to(buffer),
            BasicType::LONG(pod) => pod.clone_to(buffer),
            BasicType::FLOAT(pod) => pod.clone_to(buffer),
            BasicType::DOUBLE(pod) => pod.clone_to(buffer),
            BasicType::STRING(pod) => pod.clone_to(buffer),
            BasicType::BYTES(pod) => pod.clone_to(buffer),
            BasicType::RECTANGLE(pod) => pod.clone_to(buffer),
            BasicType::FRACTION(pod) => pod.clone_to(buffer),
            BasicType::BITMAP(pod) => pod.clone_to(buffer),
            BasicType::ARRAY(pod) => pod.clone_to(buffer),
            BasicType::STRUCT(pod) => pod.clone_to(buffer),
            BasicType::OBJECT(pod) => pod.clone_to(buffer),
            BasicType::SEQUENCE(pod) => pod.clone_to(buffer),
            BasicType::POINTER(pod) => pod.clone_to(buffer),
            BasicType::FD(pod) => pod.clone_to(buffer),
            BasicType::CHOICE(pod) => pod.clone_to(buffer),
            BasicType::POD(pod) => Err(PodError::UnknownPodTypeToDowncast),
        }
    }
}

impl StaticTypePod for PodRef {
    fn static_type() -> Type {
        Type::POD
    }
}

impl Debug for PodRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodRef")
            .field("size", &self.size())
            .field("type", &self.type_())
            .field("value", &self.downcast())
            .finish()
    }
}

#[repr(u32)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum BasicType<'a> {
    NONE(&'a PodRef) = Type::NONE.raw,
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
    NONE(<&'a PodRef as PodValue>::Value) = Type::NONE.raw,
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
    CHOICE(<&'a PodChoiceRef as PodValue>::Value) = Type::CHOICE.raw,
    POD(<&'a PodRef as PodValue>::Value) = Type::POD.raw,
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
