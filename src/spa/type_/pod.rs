use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;
use pipewire_proc_macro::RawWrapper;
use spa_sys::{spa_pod, spa_pod_bool};
use std::fmt::{Debug, Formatter};
use std::mem;
use std::mem::size_of;
use std::ptr::addr_of;

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

    fn downcast(&self) -> crate::Result<BasicType> {
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
            _ => Err(crate::Error::PodParseError("Unknown basic pod type")),
        }
    }
}

impl BasicTypePod for PodRef {
    type Value = ();

    fn parse_value(&self) -> crate::Result<Self::Value> {
        Ok(())
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
// todo #[derive(Debug)]
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

pub trait BasicTypePod: RawWrapper {
    type Value;

    fn parse_value(&self) -> crate::Result<Self::Value>;

    fn upcast(&self) -> &PodRef {
        unsafe { PodRef::from_raw_ptr(self as *const _ as *const spa_pod) }
    }

    fn cast<T>(&self) -> crate::Result<&T>
    where
        T: RawWrapper,
    {
        if self.upcast().size() as usize >= mem::size_of::<T>() {
            unsafe { Ok(T::from_raw_ptr(addr_of!(*self) as *const _)) }
        } else {
            Err(crate::Error::PodParseError(
                "POD data size is too small for that type",
            ))
        }
    }

    unsafe fn content(&self) -> &[u8] {
        let self_ptr = &self as *const _ as *const u8;
        let content_ptr = self_ptr.offset(size_of::<Self>() as isize);
        let content_size = self.upcast().size() as usize;
        std::slice::from_raw_parts(content_ptr, content_size)
    }
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodBoolRef {
    #[raw]
    raw: spa_sys::spa_pod_bool,
}

impl BasicTypePod for PodBoolRef {
    type Value = bool;

    fn parse_value(&self) -> crate::Result<Self::Value> {
        Ok(self.raw.value != 0)
    }
}

impl Debug for PodBoolRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("PodBoolRef")
                .field("pod", &self.upcast())
                .field("value", &self.parse_value())
                .finish()
        }
    }
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodIdRef {
    #[raw]
    raw: spa_sys::spa_pod_id,
}

impl BasicTypePod for PodIdRef {
    type Value = u32;

    fn parse_value(&self) -> crate::Result<Self::Value> {
        Ok(self.raw.value)
    }
}

impl Debug for PodIdRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("PodIdRef")
                .field("pod", &self.upcast())
                .field("value", &self.parse_value())
                .finish()
        }
    }
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodIntRef {
    #[raw]
    raw: spa_sys::spa_pod_int,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodLongRef {
    #[raw]
    raw: spa_sys::spa_pod_long,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodFloatRef {
    #[raw]
    raw: spa_sys::spa_pod_float,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodDoubleRef {
    #[raw]
    raw: spa_sys::spa_pod_double,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodStringRef {
    #[raw]
    raw: spa_sys::spa_pod_string,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodBytesRef {
    #[raw]
    raw: spa_sys::spa_pod_bytes,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodRectangleRef {
    #[raw]
    raw: spa_sys::spa_pod_rectangle,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodFractionRef {
    #[raw]
    raw: spa_sys::spa_pod_fraction,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodBitmapRef {
    #[raw]
    raw: spa_sys::spa_pod_bitmap,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodArrayBodyRef {
    #[raw]
    raw: spa_sys::spa_pod_array_body,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodArrayRef {
    #[raw]
    raw: spa_sys::spa_pod_array,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodChoiceBodyRef {
    #[raw]
    raw: spa_sys::spa_pod_choice_body,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodChoiceRef {
    #[raw]
    raw: spa_sys::spa_pod_choice,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodStructRef {
    #[raw]
    raw: spa_sys::spa_pod_struct,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodObjectBodyRef {
    #[raw]
    raw: spa_sys::spa_pod_object_body,
}

impl PodObjectBodyRef {
    pub fn type_(&self) -> Type {
        Type::from_raw(self.raw.type_)
    }

    pub fn id(&self) -> u32 {
        self.raw.id
    }
}

impl Debug for PodObjectBodyRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodObjectBodyRef")
            .field("type", &self.type_())
            .field("id", &self.id())
            .finish()
    }
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodObjectRef {
    #[raw]
    raw: spa_sys::spa_pod_object,
}

impl BasicTypePod for PodObjectRef {
    type Value = PodObjectBodyRef;

    fn parse_value(&self) -> crate::Result<Self::Value> {
        todo!()
    }
}

impl Debug for PodObjectRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("PodObjectRef")
                .field("pod", &self.upcast())
                .field("body", &self.body())
                .finish()
        }
    }
}

impl PodObjectRef {
    pub fn body(&self) -> &PodObjectBodyRef {
        unsafe { PodObjectBodyRef::from_raw_ptr(&self.raw.body) }
    }
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodPointerBodyRef {
    #[raw]
    raw: spa_sys::spa_pod_pointer_body,
}

#[derive(RawWrapper)]
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

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodPropRef {
    #[raw]
    raw: spa_sys::spa_pod_prop,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodControlRef {
    #[raw]
    raw: spa_sys::spa_pod_control,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodSequenceBodyRef {
    #[raw]
    raw: spa_sys::spa_pod_sequence_body,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodSequenceRef {
    #[raw]
    raw: spa_sys::spa_pod_sequence,
}
