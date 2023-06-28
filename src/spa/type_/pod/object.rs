use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::addr_of;

use bitflags::{bitflags, Flags};

use pipewire_macro_impl::enum_wrapper;
use pipewire_proc_macro::RawWrapper;
use prop::ObjectPropType;
use prop_info::ObjectPropInfoType;

use crate::spa::type_::pod::array::PodArrayRef;
use crate::spa::type_::pod::choice::PodChoiceRef;
use crate::spa::type_::pod::id::{PodIdRef, PodIdType};
use crate::spa::type_::pod::iterator::PodIterator;
use crate::spa::type_::pod::object::format::ObjectFormatType;
use crate::spa::type_::pod::object::prop::Prop;
use crate::spa::type_::pod::string::PodStringRef;
use crate::spa::type_::pod::struct_::PodStructRef;
use crate::spa::type_::pod::{
    BasicType, Pod, PodBoolRef, PodDoubleRef, PodError, PodFdRef, PodFloatRef, PodIntRef,
    PodLongRef, PodRef, PodResult, PodSubtype, ReadablePod,
};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

pub mod format;
pub mod prop;
pub mod prop_info;

#[derive(RawWrapper)]
#[repr(transparent)]
struct PodObjectBodyRef {
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

impl Pod for PodObjectRef {
    fn pod_size(&self) -> usize {
        self.upcast().pod_size()
    }
}

impl PodSubtype for PodObjectRef {
    fn static_type() -> Type {
        Type::OBJECT
    }
}

impl Debug for PodObjectRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("PodObjectRef")
                .field("pod", &self.upcast())
                .field("body_type", &self.body_type())
                .field("body_id", &self.body_id())
                .field(
                    "value",
                    &self.value().map(|v| match v {
                        ObjectType::OBJECT_PROP_INFO(iter) => {
                            iter.map(|p| format!("{:?}", p.value())).collect::<Vec<_>>()
                        }
                        ObjectType::OBJECT_PROPS(iter) => {
                            iter.map(|p| format!("{:?}", p.value())).collect::<Vec<_>>()
                        }
                        ObjectType::OBJECT_FORMAT(iter) => {
                            iter.map(|p| format!("{:?}", p.value())).collect::<Vec<_>>()
                        }
                    }),
                )
                .finish()
        }
    }
}

impl PodObjectRef {
    fn body(&self) -> &PodObjectBodyRef {
        unsafe { PodObjectBodyRef::from_raw_ptr(&self.raw.body) }
    }

    pub fn body_type(&self) -> Type {
        self.body().type_()
    }

    pub fn body_id(&self) -> u32 {
        self.body().id()
    }
}

impl<'a> ReadablePod for &'a PodObjectRef {
    type Value = ObjectType<'a>;

    fn value(&self) -> PodResult<Self::Value> {
        Ok(match self.body_type() {
            Type::OBJECT_PROP_INFO => ObjectType::OBJECT_PROP_INFO(PodIterator::new(self)),
            Type::OBJECT_PROPS => ObjectType::OBJECT_PROPS(PodIterator::new(self)),
            Type::OBJECT_FORMAT => ObjectType::OBJECT_FORMAT(PodIterator::new(self)),
            _ => {
                todo!()
            }
        })
    }
}

pub trait PodPropKeyType<'a>
where
    Self: 'a,
    Self: TryFrom<&'a PodPropRef<'a, Self>, Error = PodError>,
    Self: Debug,
{
}

#[repr(transparent)]
pub struct PodPropRef<'a, T: PodPropKeyType<'a>> {
    raw: spa_sys::spa_pod_prop,
    phantom_type: PhantomData<&'a T>,
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct PodPropFlags: u32 {
        const READONLY = spa_sys::SPA_POD_PROP_FLAG_READONLY;
        const HARDWARE = spa_sys::SPA_POD_PROP_FLAG_HARDWARE;
        const HINT_DICT = spa_sys::SPA_POD_PROP_FLAG_HINT_DICT;
        const MANDATORY = spa_sys::SPA_POD_PROP_FLAG_MANDATORY;
        const DONT_FIXATE = spa_sys::SPA_POD_PROP_FLAG_DONT_FIXATE;
    }
}

impl<'a, T: PodPropKeyType<'a>> RawWrapper for PodPropRef<'a, T> {
    type CType = spa_sys::spa_pod_prop;

    fn as_raw_ptr(&self) -> *mut Self::CType {
        &self.raw as *const _ as *mut _
    }

    fn as_raw(&self) -> &Self::CType {
        &self.raw
    }

    fn from_raw(raw: Self::CType) -> Self {
        Self {
            raw,
            phantom_type: PhantomData::default(),
        }
    }

    unsafe fn mut_from_raw_ptr<'b>(raw: *mut Self::CType) -> &'b mut Self {
        &mut *(raw as *mut PodPropRef<T>)
    }
}

impl<'a, T: PodPropKeyType<'a>> Pod for PodPropRef<'a, T> {
    fn pod_size(&self) -> usize {
        size_of::<PodPropRef<T>>() + self.pod().size() as usize
    }
}

impl<'a, T: PodPropKeyType<'a>> PodPropRef<'a, T> {
    pub fn key(&self) -> u32 {
        self.raw.key
    }

    pub fn flags(&self) -> PodPropFlags {
        PodPropFlags::from_bits_retain(self.raw.flags)
    }

    pub fn pod(&self) -> &PodRef {
        unsafe { PodRef::from_raw_ptr(addr_of!(self.raw.value) as *const spa_sys::spa_pod) }
    }
}

impl<'a, T: PodPropKeyType<'a>> ReadablePod for &'a PodPropRef<'a, T> {
    type Value = T;

    fn value(&self) -> PodResult<Self::Value> {
        (*self).try_into()
    }
}

impl<'a, T: PodPropKeyType<'a>> Debug for &'a PodPropRef<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("PodPropRef")
                .field("key", &self.key())
                .field("flags", &self.flags())
                .field("pod", &self.pod())
                .field("value", &self.value())
                .finish()
        }
    }
}

pub type ObjectPropsIterator<'a, T> = PodIterator<'a, PodObjectRef, PodPropRef<'a, T>>;

#[repr(u32)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum ObjectType<'a> {
    OBJECT_PROP_INFO(ObjectPropsIterator<'a, ObjectPropInfoType<'a>>) = Type::OBJECT_PROP_INFO.raw,
    OBJECT_PROPS(ObjectPropsIterator<'a, ObjectPropType<'a>>) = Type::OBJECT_PROPS.raw,
    OBJECT_FORMAT(ObjectPropsIterator<'a, ObjectFormatType<'a>>) = Type::OBJECT_FORMAT.raw,
}
