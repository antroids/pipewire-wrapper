use std::fmt::{Debug, Formatter};

use pipewire_proc_macro::RawWrapper;

use crate::spa::type_::pod::iterator::PodIterator;
use crate::spa::type_::pod::{Pod, PodRef, PodResult, PodSubtype, PodValueParser, ReadablePod};
use crate::spa::type_::Type;

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodStructRef {
    #[raw]
    raw: spa_sys::spa_pod_struct,
}

impl PodSubtype for PodStructRef {
    fn static_type() -> Type {
        Type::STRUCT
    }
}

impl Pod for PodStructRef {
    fn pod_size(&self) -> usize {
        self.upcast().pod_size()
    }
}

impl<'a> ReadablePod for &'a PodStructRef {
    type Value = PodIterator<'a, PodStructRef, PodRef>;

    fn value(&self) -> PodResult<Self::Value> {
        Ok(PodIterator::new(self))
    }
}

impl Debug for PodStructRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodStructRef")
            .field("pod", &self.upcast())
            .field(
                "value",
                &self
                    .value()
                    .map(|v| v.map(|p| p.downcast()).collect::<Vec<_>>()),
            )
            .finish()
    }
}
