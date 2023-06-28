use std::fmt::{Debug, Formatter};
use std::mem::size_of;
use std::ptr::addr_of;

use pipewire_macro_impl::enum_wrapper;
use pipewire_proc_macro::RawWrapper;

use crate::spa::type_::pod::bytes::PodBytesRef;
use crate::spa::type_::pod::iterator::PodIterator;
use crate::spa::type_::pod::object::prop::ObjectPropType;
use crate::spa::type_::pod::object::{ObjectPropsIterator, ObjectType, PodPropRef};
use crate::spa::type_::pod::restricted::{PodSubtype, PodValueParser};
use crate::spa::type_::pod::{Pod, PodError, PodRef, PodResult, ReadablePod};
use crate::wrapper::RawWrapper;

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodControlRef {
    #[raw]
    raw: spa_sys::spa_pod_control,
}

impl Pod for PodControlRef {
    fn pod_size(&self) -> usize {
        size_of::<PodControlRef>() + self.raw.value.size as usize
    }
}

impl PodControlRef {
    fn type_(&self) -> Type {
        Type::from_raw(self.raw.type_)
    }

    fn value_pod(&self) -> &PodRef {
        unsafe { PodRef::from_raw_ptr(addr_of!(self.raw.value)) }
    }
}

#[repr(u32)]
#[derive(Debug)]
pub enum ControlType<'a> {
    INVALID = Type::INVALID.raw,
    PROPERTIES(PodIterator<'a, PodControlRef, PodPropRef<'a, ObjectPropType<'a>>>) =
        Type::PROPERTIES.raw,
    MIDI(&'a PodBytesRef) = Type::MIDI.raw,
    OSC(&'a PodBytesRef) = Type::OSC.raw,
}

impl<'a> ReadablePod for &'a PodControlRef {
    type Value = ControlType<'a>;

    fn value(&self) -> PodResult<Self::Value> {
        todo!()
    }
}

impl<'a> PodValueParser<&'a PodControlRef> for &'a PodControlRef {
    fn parse(_size: u32, value: &'a PodControlRef) -> PodResult<<Self as ReadablePod>::Value> {
        match value.type_() {
            Type::INVALID => Ok(ControlType::INVALID),
            Type::PROPERTIES => Ok(ControlType::PROPERTIES(PodIterator::new(value))),
            Type::MIDI => value.value_pod().cast().map(|r| ControlType::MIDI(r)),
            Type::OSC => value.value_pod().cast().map(|r| ControlType::OSC(r)),
            type_ => Err(PodError::UnexpectedControlType(type_.raw)),
        }
    }
}

impl<'a> PodValueParser<*const u8> for &'a PodControlRef {
    fn parse(_size: u32, value: *const u8) -> PodResult<<Self as ReadablePod>::Value> {
        unsafe { Self::parse(_size, PodControlRef::from_raw_ptr(value.cast())) }
    }
}

impl<'a> Debug for &'a PodControlRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodControlRef")
            .field("type", &self.type_())
            .field("offset", &self.raw.offset)
            .field("value", &self.value())
            .finish()
    }
}

enum_wrapper!(
    Type,
    spa_sys::spa_control_type,
    INVALID: spa_sys::SPA_CONTROL_Invalid,
    PROPERTIES: spa_sys::SPA_CONTROL_Properties,
    MIDI: spa_sys::SPA_CONTROL_Midi,
    OSC: spa_sys::SPA_CONTROL_OSC,
);
