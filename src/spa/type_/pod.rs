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
use crate::spa::type_::pod::object::prop::Prop;
use crate::spa::type_::pod::restricted::{PodSubtype, PodValueParser};
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
        impl restricted::PodValueParser<*const u8> for $pod_ref_type {
            fn parse(size: u32, value: *const u8) -> PodResult<Self::Value> {
                unsafe { Self::parse(size, *(value as *const $value_raw_type)) }
            }
        }

        impl restricted::PodValueParser<$value_raw_type> for $pod_ref_type {
            fn parse(size: u32, value: $value_raw_type) -> PodResult<Self::Value> {
                if (size as usize) < size_of::<$value_raw_type>() {
                    Err(PodError::DataIsTooShort(
                        size_of::<$value_raw_type>(),
                        size as usize,
                    ))
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

        impl Pod for $pod_ref_type {
            fn pod_size(&self) -> usize {
                self.upcast().pod_size()
            }
        }

        impl ValuePod for $pod_ref_type {
            type Value = $value_type;
        }

        impl restricted::PodSubtype for $pod_ref_type {
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

pub enum PodError {
    DataIsTooShort(usize, usize),
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
        Self::PodParseError(value)
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
            PodError::WrongPodTypeToCast => write!(f, "Cannot cast Pod to this type"),
            PodError::StringIsNotNullTerminated => write!(f, "String is not null terminated"),
            PodError::IndexIsOutOfRange => write!(f, "Index is out of range"),
            PodError::ChoiceElementMissing => {
                write!(f, "Cannot find required element for choice type")
            }
            PodError::UnexpectedChoiceElement => write!(f, "Unexpected element in the choice type"),
            PodError::UnsupportedChoiceElementType => write!(f, "Unsupported choice element type"),
        }
    }
}

type PodResult<T> = Result<T, PodError>;

pub trait Pod {
    fn pod_size(&self) -> usize;
}

pub trait ValuePod {
    type Value;
}

pub trait ReadablePod {
    type Value: Debug;

    fn value(&self) -> PodResult<Self::Value>;
}

pub(crate) mod restricted {
    use std::fmt::Debug;
    use std::ptr::addr_of;

    use spa_sys::spa_pod;

    use crate::spa::type_::pod::{Pod, PodError, PodRef, PodResult, ReadablePod};
    use crate::spa::type_::Type;
    use crate::wrapper::RawWrapper;

    pub trait PodValueParser<F: Copy>: ReadablePod {
        fn parse(size: u32, value: F) -> PodResult<<Self as ReadablePod>::Value>;
    }

    pub trait PodSubtype: RawWrapper + Pod + Debug {
        fn static_type() -> Type;

        fn upcast(&self) -> &PodRef {
            unsafe { PodRef::from_raw_ptr(self as *const _ as *const spa_pod) }
        }

        fn cast<T>(&self) -> PodResult<&T>
        where
            T: PodSubtype,
        {
            if T::static_type() == self.upcast().type_() {
                unsafe { Ok(self.cast_unchecked()) }
            } else {
                Err(PodError::WrongPodTypeToCast)
            }
        }

        unsafe fn cast_unchecked<T>(&self) -> &T
        where
            T: PodSubtype,
        {
            T::from_raw_ptr(addr_of!(*self) as *const _)
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

impl Pod for PodRef {
    fn pod_size(&self) -> usize {
        size_of::<PodRef>() + self.as_raw().size as usize
    }
}

impl<'a> ValuePod for &'a PodRef {
    type Value = BasicType<'a>;
}

impl<'a> restricted::PodValueParser<*const u8> for &'a PodRef {
    fn parse(size: u32, value: *const u8) -> PodResult<Self::Value> {
        unsafe { Self::parse(size, PodRef::from_raw_ptr(value as *const spa_sys::spa_pod)) }
    }
}

impl<'a> restricted::PodValueParser<&'a PodRef> for &'a PodRef {
    fn parse(size: u32, value: &'a PodRef) -> PodResult<Self::Value> {
        value.downcast()
    }
}

impl<'a> ReadablePod for &'a PodRef {
    type Value = BasicType<'a>;

    fn value(&self) -> PodResult<Self::Value> {
        Self::parse(self.size(), *self)
    }
}

impl restricted::PodSubtype for PodRef {
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
    NONE(<&'a PodRef as ReadablePod>::Value) = Type::NONE.raw,
    BOOL(<PodBoolRef as ReadablePod>::Value) = Type::BOOL.raw,
    ID(<PodIdRef as ReadablePod>::Value) = Type::ID.raw,
    INT(<PodIntRef as ReadablePod>::Value) = Type::INT.raw,
    LONG(<PodLongRef as ReadablePod>::Value) = Type::LONG.raw,
    FLOAT(<PodFloatRef as ReadablePod>::Value) = Type::FLOAT.raw,
    DOUBLE(<PodDoubleRef as ReadablePod>::Value) = Type::DOUBLE.raw,
    STRING(<&'a PodStringRef as ReadablePod>::Value) = Type::STRING.raw,
    BYTES(<&'a PodBytesRef as ReadablePod>::Value) = Type::BYTES.raw,
    RECTANGLE(<PodRectangleRef as ReadablePod>::Value) = Type::RECTANGLE.raw,
    FRACTION(<PodFractionRef as ReadablePod>::Value) = Type::FRACTION.raw,
    BITMAP(<&'a PodBitmapRef as ReadablePod>::Value) = Type::BITMAP.raw,
    ARRAY(<&'a PodArrayRef as ReadablePod>::Value) = Type::ARRAY.raw,
    // STRUCT(<&'a PodStructRef as ReadablePod>::Value)=Type::STRUCT.raw,
    // OBJECT(<&'a PodObjectRef  as ReadablePod>::Value)=Type::OBJECT.raw,
    // SEQUENCE(<&'a PodSequenceRef as ReadablePod>::Value)=Type::SEQUENCE.raw,
    // POINTER(<&'a PodPointerRef as ReadablePod>::Value)=Type::POINTER.raw,
    // FD(<PodFdRef as ReadablePod>::Value)=Type::FD.raw,
    // CHOICE(<&'a PodChoiceRef as ReadablePod>::Value) = Type::CHOICE.raw,
    POD(<&'a PodRef as ReadablePod>::Value) = Type::POD.raw,
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
