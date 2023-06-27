use std::ffi::{c_char, CStr};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::mem::size_of;
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
use crate::spa::type_::pod::object::Prop;
use crate::spa::type_::{FractionRef, RectangleRef, Type};
use crate::wrapper::RawWrapper;

pub mod array;
pub mod bitmap;
pub mod bytes;
pub mod choice;
pub mod control;
pub mod format;
pub mod id;
pub mod iterator;
pub mod object;
pub mod pointer;
pub mod sequence;
pub mod string;
pub mod struct_;

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
        impl PodValueParser<*const u8> for $pod_ref_type {
            type To = $value_type;

            fn parse(size: u32, value: *const u8) -> PodResult<Self::To> {
                unsafe { Self::parse(size, *(value as *const $value_raw_type)) }
            }
        }

        impl PodValueParser<$value_raw_type> for $pod_ref_type {
            type To = $value_type;

            fn parse(size: u32, value: $value_raw_type) -> PodResult<Self::To> {
                if (size as usize) < size_of::<$value_raw_type>() {
                    Err(PodError::DataIsTooShort)
                } else {
                    let $value_ident = value;
                    Ok($convert_value_expr)
                }
            }
        }

        impl ReadablePod for $pod_ref_type {
            type Value = $value_type;

            fn value(&self) -> PodResult<Self::Value> {
                Self::parse(self.upcast().size(), self.value())
            }
        }

        impl SizedPod for $pod_ref_type {
            fn pod_size(&self) -> usize {
                self.upcast().pod_size()
            }
        }

        impl BasicTypePod for $pod_ref_type {
            fn static_type() -> Type {
                $pod_type
            }
        }

        impl $pod_ref_type {
            pub fn value(&self) -> $value_raw_type {
                self.raw.value
            }
        }

        impl Debug for $pod_ref_type {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                unsafe {
                    f.debug_struct(stringify!($pod_ref_type))
                        .field("pod", &self.upcast())
                        .field("value", &self.value())
                        .finish()
                }
            }
        }
    };
}

#[derive(Debug)]
pub enum PodError {
    DataIsTooShort,
    UnknownPodTypeToDowncast,
    WrongPodTypeToCast,
    StringIsNotNullTerminated,
    IndexIsOutOfRange,
    ChoiceElementMissing,
    UnexpectedChoiceElement,
    UnsupportedChoiceElementType,
}

impl From<PodError> for crate::Error {
    fn from(value: PodError) -> Self {
        Self::PodParseError(match value {
            PodError::DataIsTooShort => "POD data size is too small for that type",
            PodError::UnknownPodTypeToDowncast => "Cannot downcast Pod, child type is unknown",
            PodError::WrongPodTypeToCast => "Cannot cast Pod to this type",
            PodError::StringIsNotNullTerminated => "String is not null terminated",
            PodError::IndexIsOutOfRange => "Index is out of range",
            PodError::ChoiceElementMissing => "Cannot find required element for choice type",
            PodError::UnexpectedChoiceElement => "Unexpected element in the choice type",
            PodError::UnsupportedChoiceElementType => "Unsupported choice element type",
        })
    }
}

type PodResult<T> = Result<T, PodError>;

pub trait SizedPod {
    fn pod_size(&self) -> usize;
}

pub trait PodValueParser<F: Copy> {
    type To: Debug;

    fn parse(size: u32, value: F) -> PodResult<Self::To>;
}

pub trait ReadablePod {
    type Value;

    fn value(&self) -> PodResult<Self::Value>;
}

pub trait BasicTypePod: RawWrapper + SizedPod + Debug {
    fn static_type() -> Type;

    fn upcast(&self) -> &PodRef {
        unsafe { PodRef::from_raw_ptr(self as *const _ as *const spa_pod) }
    }

    unsafe fn cast<T>(&self) -> PodResult<&T>
    where
        T: BasicTypePod,
    {
        if T::static_type() == self.upcast().type_() {
            Ok(T::from_raw_ptr(addr_of!(*self) as *const _))
        } else {
            Err(PodError::WrongPodTypeToCast)
        }
    }

    unsafe fn content_slice(&self) -> &[u8] {
        let content_ptr: *const u8 = self.content_ptr();
        let content_size = self.upcast().size() as usize;
        std::slice::from_raw_parts(content_ptr, content_size)
    }

    unsafe fn content_ptr<T>(&self) -> *const T {
        let self_ptr = self as *const Self;
        self_ptr.offset(1).cast()
    }
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
                //Type::SEQUENCE => self.cast().map(|r| BasicType::SEQUENCE(r)),
                //Type::POINTER => self.cast().map(|r| BasicType::POINTER(r)),
                Type::FD => self.cast().map(|r| BasicType::FD(r)),
                Type::CHOICE => self.cast().map(|r| BasicType::CHOICE(r)),
                Type::POD => self.cast().map(|r| BasicType::POD(r)),
                _ => Err(PodError::UnknownPodTypeToDowncast),
            }
        }
    }
}

impl SizedPod for PodRef {
    fn pod_size(&self) -> usize {
        size_of::<PodRef>() + self.as_raw().size as usize
    }
}

impl<'a> PodValueParser<*const u8> for &'a PodRef {
    type To = BasicType<'a>;

    fn parse(size: u32, value: *const u8) -> PodResult<Self::To> {
        unsafe { Self::parse(size, PodRef::from_raw_ptr(value as *const spa_sys::spa_pod)) }
    }
}

impl<'a> PodValueParser<&'a PodRef> for &'a PodRef {
    type To = BasicType<'a>;

    fn parse(size: u32, value: &'a PodRef) -> PodResult<Self::To> {
        value.downcast()
    }
}

impl<'a> ReadablePod for &'a PodRef {
    type Value = BasicType<'a>;

    fn value(&self) -> PodResult<Self::Value> {
        Self::parse(self.size(), *self)
    }
}

impl BasicTypePod for PodRef {
    fn static_type() -> Type {
        Type::POD
    }
}

impl Debug for PodRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodRef")
            .field("size", &self.size())
            .field("type", &self.type_())
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
    NONE(<&'a PodRef as PodValueParser<*const u8>>::To) = Type::NONE.raw,
    BOOL(<PodBoolRef as PodValueParser<*const u8>>::To) = Type::BOOL.raw,
    ID(<PodIdRef as PodValueParser<*const u8>>::To) = Type::ID.raw,
    INT(<PodIntRef as PodValueParser<*const u8>>::To) = Type::INT.raw,
    LONG(<PodLongRef as PodValueParser<*const u8>>::To) = Type::LONG.raw,
    FLOAT(<PodFloatRef as PodValueParser<*const u8>>::To) = Type::FLOAT.raw,
    DOUBLE(<PodDoubleRef as PodValueParser<*const u8>>::To) = Type::DOUBLE.raw,
    STRING(<&'a PodStringRef as PodValueParser<*const u8>>::To) = Type::STRING.raw,
    BYTES(<&'a PodBytesRef as PodValueParser<*const u8>>::To) = Type::BYTES.raw,
    RECTANGLE(<PodRectangleRef as PodValueParser<*const u8>>::To) = Type::RECTANGLE.raw,
    FRACTION(<PodFractionRef as PodValueParser<*const u8>>::To) = Type::FRACTION.raw,
    BITMAP(<&'a PodBitmapRef as PodValueParser<*const u8>>::To) = Type::BITMAP.raw,
    ARRAY(<&'a PodArrayRef as PodValueParser<*const u8>>::To) = Type::ARRAY.raw,
    // STRUCT(<&'a PodStructRef as  PodValueParser<*const u8>>::To)=Type::STRUCT.raw,
    // OBJECT(<&'a PodObjectRef  as PodValueParser<*const u8>>::To)=Type::OBJECT.raw,
    // SEQUENCE(<&'a PodSequenceRef as  PodValueParser<*const u8>>::To)=Type::SEQUENCE.raw,
    // POINTER(<&'a PodPointerRef as  PodValueParser<*const u8>>::To)=Type::POINTER.raw,
    // FD(<PodFdRef as  PodValueParser<*const u8>>::To)=Type::FD.raw,
    // CHOICE(<&'a PodChoiceRef as PodValueParser<*const u8>>::To) = Type::CHOICE.raw,
    POD(<&'a PodRef as PodValueParser<*const u8>>::To) = Type::POD.raw,
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

#[derive(RawWrapper)]
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

#[derive(RawWrapper)]
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

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct PodPointerRef {
    #[raw]
    raw: spa_sys::spa_pod_pointer,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodFdRef {
    #[raw]
    raw: spa_sys::spa_pod_fd,
}

primitive_type_pod_impl!(PodFdRef, Type::FD, i64, RawFd, v, v as RawFd);
