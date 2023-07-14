use std::ptr::addr_of;

use pipewire_proc_macro::RawWrapper;

use crate::spa::pod::control::PodControlRef;
use crate::spa::pod::iterator::PodIterator;
use crate::spa::pod::restricted::{PodHeader, StaticTypePod};
use crate::spa::pod::{BasicType, PodResult, PodValue, SizedPod};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper)]
#[repr(transparent)]
struct PodSequenceBodyRef {
    #[raw]
    raw: spa_sys::spa_pod_sequence_body,
}

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct PodSequenceRef {
    #[raw]
    raw: spa_sys::spa_pod_sequence,
}

impl PodSequenceRef {
    fn body(&self) -> &PodSequenceBodyRef {
        unsafe { PodSequenceBodyRef::from_raw_ptr(addr_of!(self.raw.body)) }
    }
}

impl PodHeader for PodSequenceRef {
    fn pod_header(&self) -> &spa_sys::spa_pod {
        &self.raw.pod
    }
}

impl StaticTypePod for PodSequenceRef {
    fn static_type() -> Type {
        Type::SEQUENCE
    }
}

impl<'a> PodValue for &'a PodSequenceRef {
    type Value = PodIterator<'a, PodControlRef>;
    type RawValue = spa_sys::spa_pod_sequence_body;

    fn raw_value_ptr(&self) -> *const Self::RawValue {
        &self.raw.body
    }

    fn parse_raw_value(ptr: *const Self::RawValue, size: usize) -> PodResult<Self::Value> {
        let first_element_ptr = unsafe { ptr.offset(1) as *const PodControlRef };
        Ok(PodIterator::new(first_element_ptr, size))
    }

    fn value(&self) -> PodResult<Self::Value> {
        Self::parse_raw_value(self.raw_value_ptr(), self.pod_header().size as usize)
    }
}
