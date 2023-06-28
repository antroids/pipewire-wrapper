use std::ptr::addr_of;

use pipewire_proc_macro::RawWrapper;

use crate::spa::type_::pod::control::PodControlRef;
use crate::spa::type_::pod::iterator::PodIterator;
use crate::spa::type_::pod::restricted::{PodSubtype, PodValueParser};
use crate::spa::type_::pod::{BasicType, Pod, PodResult, ReadablePod};
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

impl Pod for PodSequenceRef {
    fn pod_size(&self) -> usize {
        self.upcast().pod_size()
    }
}

impl PodSubtype for PodSequenceRef {
    fn static_type() -> Type {
        Type::SEQUENCE
    }
}

impl<'a> ReadablePod for &'a PodSequenceRef {
    type Value = PodIterator<'a, PodSequenceRef, PodControlRef>;

    fn value(&self) -> PodResult<Self::Value> {
        Ok(PodIterator::new(self))
    }
}
