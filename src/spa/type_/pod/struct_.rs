use std::fmt::{Debug, Formatter};
use std::io::{Seek, Write};

use pipewire_proc_macro::RawWrapper;

use crate::spa::type_::pod::iterator::PodIterator;
use crate::spa::type_::pod::restricted::{PodHeader, StaticTypePod};
use crate::spa::type_::pod::{
    BasicTypePod, PodRef, PodResult, PodValueParser, ReadablePod, SizedPod, WritablePod,
    WritableValue,
};
use crate::spa::type_::Type;

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodStructRef {
    #[raw]
    raw: spa_sys::spa_pod_struct,
}

impl StaticTypePod for PodStructRef {
    fn static_type() -> Type {
        Type::STRUCT
    }
}

impl PodHeader for PodStructRef {
    fn pod_header(&self) -> &spa_sys::spa_pod {
        &self.raw.pod
    }
}

impl<'a> ReadablePod for &'a PodStructRef {
    type Value = PodIterator<'a, PodStructRef, PodRef>;

    fn value(&self) -> PodResult<Self::Value> {
        Ok(PodIterator::new(self))
    }
}

impl<'a> WritablePod for &'a PodStructRef {
    fn write_pod<W>(buffer: &mut W, value: &<Self as ReadablePod>::Value) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        let iterator_content = unsafe { value.as_bytes() };
        let header_size = Self::write_header(
            buffer,
            iterator_content.len() as u32,
            PodStructRef::static_type(),
        )?;
        buffer.write_all(iterator_content)?;
        Ok(header_size + iterator_content.len() + Self::write_align_padding(buffer)?)
    }
}

impl<'a> WritableValue for &'a PodStructRef {
    fn write_raw_value<W>(buffer: &mut W, value: &<Self as ReadablePod>::Value) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        let iterator_content = unsafe { value.as_bytes() };
        buffer.write_all(iterator_content)?;
        Ok(iterator_content.len())
    }
}

impl Debug for PodStructRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodStructRef")
            .field("pod.type", &self.upcast().type_())
            .field("pod.size", &self.upcast().size())
            .field(
                "value",
                &self
                    .value()
                    .map(|v| v.map(|p| p.downcast()).collect::<Vec<_>>()),
            )
            .finish()
    }
}
