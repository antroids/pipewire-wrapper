use std::ffi::{c_char, CStr};
use std::fmt::{Debug, Formatter};
use std::io::{Cursor, Seek, Write};
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
use crate::spa::type_::pod::pointer::PodPointerRef;
use crate::spa::type_::pod::restricted::{PodHeader, PodValueParser, StaticTypePod};
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
            fn parse(size: usize, value: *const u8) -> PodResult<Self::Value> {
                unsafe { Self::parse(size, *(value as *const $value_raw_type)) }
            }
        }

        impl restricted::PodValueParser<$value_raw_type> for $pod_ref_type {
            fn parse(size: usize, value: $value_raw_type) -> PodResult<Self::Value> {
                if size < size_of::<$value_raw_type>() {
                    Err(PodError::DataIsTooShort(size_of::<$value_raw_type>(), size))
                } else {
                    let $value_ident = value;
                    Ok($convert_value_expr)
                }
            }
        }

        impl ReadablePod for $pod_ref_type {
            type Value = $value_type;

            fn value(&self) -> PodResult<Self::Value> {
                Self::parse(self.upcast().size() as usize, self.raw_value())
            }
        }

        impl WritablePod for $pod_ref_type {
            fn write<W>(buffer: &mut W, value: <Self as ReadablePod>::Value) -> PodResult<usize>
            where
                W: Write + Seek,
            {
                let value: $value_raw_type = value.into();
                Ok(Self::write_header(
                    buffer,
                    size_of::<$value_raw_type>() as u32,
                    <$pod_ref_type>::static_type(),
                )? + Self::write_value(buffer, &value)?
                    + Self::write_align_padding(buffer)?)
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
const ZEROED_POD: u64 = 0;

pub trait SizedPod {
    fn pod_size(&self) -> usize;
}

pub trait ReadablePod {
    type Value: Debug;

    fn value(&self) -> PodResult<Self::Value>;
}

impl<'a, T> ReadablePod for &'a T
where
    T: ReadablePod,
{
    type Value = T::Value;

    fn value(&self) -> PodResult<Self::Value> {
        (*self).value()
    }
}

pub trait WritablePod: ReadablePod {
    fn write<W>(buffer: &mut W, value: <Self as ReadablePod>::Value) -> PodResult<usize>
    where
        W: std::io::Write + std::io::Seek;

    fn check_align<W>(buffer: &mut W) -> PodResult<()>
    where
        W: std::io::Write + std::io::Seek,
    {
        if buffer.stream_position()?.rem(POD_ALIGN as u64) == 0 {
            Ok(())
        } else {
            Err(PodError::PodIsNotAligned)
        }
    }

    fn write_header<W>(buffer: &mut W, size: u32, type_: Type) -> PodResult<usize>
    where
        W: std::io::Write + std::io::Seek,
    {
        Self::check_align(buffer)?;
        Self::write_value(
            buffer,
            &spa_sys::spa_pod {
                size,
                type_: type_.raw,
            },
        )
    }

    fn write_value<W, V>(buffer: &mut W, value: &V) -> PodResult<usize>
    where
        W: std::io::Write + std::io::Seek,
        V: Sized,
    {
        let size = size_of::<V>();
        let ptr = value as *const V as *const u8;
        let slice = unsafe { slice::from_raw_parts(ptr, size) };
        buffer.write_all(slice)?;
        Ok(size)
    }

    fn write_align_padding<W>(buffer: &mut W) -> PodResult<usize>
    where
        W: std::io::Write + std::io::Seek,
    {
        let position = buffer.stream_position()? as usize;
        let padding_len = position.rem(POD_ALIGN);
        let padding = vec![0u8; padding_len];
        buffer.write_all(padding.as_slice())?;
        Ok(padding_len)
    }
}

impl<'a, T> WritablePod for &'a T
where
    T: WritablePod,
    <T as ReadablePod>::Value: Clone,
{
    fn write<W>(buffer: &mut W, value: <Self as ReadablePod>::Value) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        T::write(buffer, value.clone())
    }
}

impl<T: PodHeader> SizedPod for T {
    fn pod_size(&self) -> usize {
        self.pod_header().size as usize + size_of::<spa_sys::spa_pod>()
    }
}

pub(crate) mod restricted {
    use std::any::{Any, TypeId};
    use std::fmt::Debug;
    use std::mem::size_of;
    use std::ptr::addr_of;

    use spa_sys::spa_pod;

    use crate::spa::type_::pod::choice::enum_::PodEnumRef;
    use crate::spa::type_::pod::choice::{ChoiceType, PodChoiceRef};
    use crate::spa::type_::pod::{PodError, PodRef, PodResult, ReadablePod, SizedPod};
    use crate::spa::type_::Type;
    use crate::wrapper::RawWrapper;

    pub trait PodHeader {
        fn pod_header(&self) -> &spa_sys::spa_pod;

        fn pod_type(&self) -> Type {
            Type::from_raw(self.pod_header().type_)
        }
    }

    pub trait StaticTypePod {
        fn static_type() -> Type;
    }

    pub trait PodValueParser<F: Copy>: ReadablePod {
        /// Parse Pod value from F.
        /// * `content_size` size in bytes of the provided data with body structure.
        /// Body structure size should be subtracted to get elements size.
        /// * `header_or_value` header or value, used to parse value.
        /// Can be first value, body structure or pointer to the value itself.
        fn parse(
            content_size: usize,
            header_or_value: F,
        ) -> PodResult<<Self as ReadablePod>::Value>;
    }
}

pub trait BasicTypePod
where
    Self: StaticTypePod,
    Self: PodHeader,
    Self: RawWrapper,
    Self: Debug,
{
    fn upcast(&self) -> &PodRef {
        unsafe { PodRef::from_raw_ptr(self.pod_header()) }
    }

    fn cast<T>(&self) -> PodResult<&T>
    where
        T: BasicTypePod,
    {
        let target_type = T::static_type();
        let pod_type = self.upcast().type_();
        if target_type == PodRef::static_type() || target_type == pod_type {
            unsafe { Ok(self.cast_unchecked()) }
        } else if pod_type == PodChoiceRef::static_type() {
            let choice: &PodChoiceRef = self.cast()?;
            // Handle fixated choices and wrong values
            // Due to ugly not restricted format, there can be any kind of data
            // So, we doing our best to parse at least default value
            choice.body().child().cast()
        } else {
            Err(PodError::WrongPodTypeToCast(target_type, pod_type))
        }
    }

    unsafe fn cast_unchecked<T>(&self) -> &T
    where
        T: BasicTypePod,
    {
        T::from_raw_ptr(addr_of!(*self) as *const _)
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

impl<'a> restricted::PodValueParser<*const u8> for &'a PodRef {
    fn parse(
        content_size: usize,
        header_or_value: *const u8,
    ) -> PodResult<<Self as ReadablePod>::Value> {
        unsafe {
            Self::parse(
                content_size,
                PodRef::from_raw_ptr(header_or_value as *const spa_sys::spa_pod),
            )
        }
    }
}

impl<'a> restricted::PodValueParser<&'a PodRef> for &'a PodRef {
    fn parse(
        content_size: usize,
        header_or_value: &'a PodRef,
    ) -> PodResult<<Self as ReadablePod>::Value> {
        header_or_value.downcast()
    }
}

impl<'a> ReadablePod for &'a PodRef {
    type Value = BasicType<'a>;

    fn value(&self) -> PodResult<Self::Value> {
        Self::parse(self.size() as usize, *self)
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
    STRUCT(<&'a PodStructRef as ReadablePod>::Value) = Type::STRUCT.raw,
    OBJECT(<&'a PodObjectRef as ReadablePod>::Value) = Type::OBJECT.raw,
    SEQUENCE(<&'a PodSequenceRef as ReadablePod>::Value) = Type::SEQUENCE.raw,
    POINTER(<&'a PodPointerRef as ReadablePod>::Value) = Type::POINTER.raw,
    FD(<PodFdRef as ReadablePod>::Value) = Type::FD.raw,
    CHOICE(<&'a PodChoiceRef as ReadablePod>::Value) = Type::CHOICE.raw,
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

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodFdRef {
    #[raw]
    raw: spa_sys::spa_pod_fd,
}

primitive_type_pod_impl!(PodFdRef, Type::FD, i64, RawFd, v, v as RawFd);
