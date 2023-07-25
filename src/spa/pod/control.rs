use std::fmt::{Debug, Formatter};
use std::mem::size_of;
use std::ptr::addr_of;

use pipewire_proc_macro::RawWrapper;

use crate::enum_wrapper;
use crate::spa::pod::bytes::PodBytesRef;
use crate::spa::pod::iterator::PodIterator;
use crate::spa::pod::object::prop::ObjectPropType;
use crate::spa::pod::object::{ObjectPropsIterator, ObjectType, PodPropRef};
use crate::spa::pod::restricted::PodHeader;
use crate::spa::pod::{BasicTypePod, PodError, PodRef, PodResult, PodValue, SizedPod};
use crate::wrapper::RawWrapper;

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodControlRef {
    #[raw]
    raw: spa_sys::spa_pod_control,
}

impl Debug for PodControlRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodControlRef")
            .field("type", &self.type_())
            .field("offset", &self.raw.offset)
            .field("value", &self.value())
            .finish()
    }
}

impl SizedPod for PodControlRef {
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

    fn content_size(&self) -> usize {
        self.raw.value.size as usize
    }
}

#[repr(u32)]
#[derive(Debug)]
pub enum ControlType<'a> {
    INVALID = Type::INVALID.raw,
    PROPERTIES(PodIterator<'a, PodPropRef<'a, ObjectPropType<'a>>>) = Type::PROPERTIES.raw,
    MIDI(&'a PodBytesRef) = Type::MIDI.raw,
    OSC(&'a PodBytesRef) = Type::OSC.raw,
}

impl<'a> PodValue for &'a PodControlRef {
    type Value = ControlType<'a>;
    type RawValue = spa_sys::spa_pod_control;

    fn raw_value_ptr(&self) -> *const Self::RawValue {
        &self.raw
    }

    fn parse_raw_value(ptr: *const Self::RawValue, _size: usize) -> PodResult<Self::Value> {
        let control = unsafe { PodControlRef::from_raw_ptr(ptr) };
        match control.type_() {
            Type::INVALID => Ok(ControlType::INVALID),
            Type::PROPERTIES => Ok(ControlType::PROPERTIES(PodIterator::from_container(
                control,
            ))),
            Type::MIDI => control.value_pod().cast().map(|r| ControlType::MIDI(r)),
            Type::OSC => control.value_pod().cast().map(|r| ControlType::OSC(r)),
            type_ => Err(PodError::UnexpectedControlType(type_.raw)),
        }
    }

    fn value(&self) -> PodResult<Self::Value> {
        Self::parse_raw_value(
            &self.raw,
            size_of::<Self::RawValue>() + self.raw.value.size as usize,
        )
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
