use std::fmt::{Debug, Formatter};
use std::io::{Seek, Write};
use std::slice;

use spa_sys::spa_pod;

use pipewire_proc_macro::RawWrapper;

use crate::spa::pod::pod_buf::PodBuf;
use crate::spa::pod::restricted::{PodHeader, StaticTypePod};
use crate::spa::pod::{
    BasicTypePod, PodResult, PodValue, SizedPod, WritePod, WriteValue, POD_ALIGN,
};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodBitmapRef {
    #[raw]
    raw: spa_sys::spa_pod_bitmap,
}

impl PodBitmapRef {
    fn content_size(&self) -> usize {
        self.raw.pod.size as usize
    }

    unsafe fn content_ptr(&self) -> *const u8 {
        (self as *const Self).offset(1).cast()
    }
}

impl StaticTypePod for PodBitmapRef {
    fn static_type() -> Type {
        Type::BITMAP
    }
}

impl PodHeader for PodBitmapRef {
    fn pod_header(&self) -> &spa_pod {
        &self.raw.pod
    }
}

impl<'a> PodValue for &'a PodBitmapRef {
    type Value = &'a [u8];
    type RawValue = u8;

    fn raw_value_ptr(&self) -> *const Self::RawValue {
        unsafe { (&self.raw.pod as *const spa_sys::spa_pod).offset(1).cast() }
    }

    fn parse_raw_value(ptr: *const Self::RawValue, size: usize) -> PodResult<Self::Value> {
        unsafe { Ok(slice::from_raw_parts(ptr, size)) }
    }

    fn value(&self) -> PodResult<Self::Value> {
        Self::parse_raw_value(self.raw_value_ptr(), self.pod_header().size as usize)
    }
}

impl<'a> WritePod for &'a PodBitmapRef {
    fn write_pod<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        Ok(
            Self::write_header(buffer, value.len() as u32, PodBitmapRef::static_type())?
                + Self::write_raw_value(buffer, value)?
                + Self::write_align_padding(buffer)?,
        )
    }
}

impl<'a> WriteValue for &'a PodBitmapRef {
    fn write_raw_value<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        buffer.write_all(value)?;
        Ok(value.len())
    }
}

impl Debug for PodBitmapRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("PodBitmapRef")
                .field("pod.type", &self.upcast().type_())
                .field("pod.size", &self.upcast().size())
                .field("value", &self.value())
                .finish()
        }
    }
}

#[test]
fn test_from_value() {
    let bytes = [1u8, 2u8, 3u8];
    let bytes_wrong = [1u8, 1u8, 1u8];
    let allocated_pod = PodBuf::<PodBitmapRef>::from_value(&bytes.as_ref())
        .unwrap()
        .into_pod();
    assert_eq!(allocated_pod.as_pod().as_ptr().align_offset(POD_ALIGN), 0);
    assert_eq!(allocated_pod.as_pod().pod_size(), 11);
    assert_eq!(allocated_pod.as_pod().pod_header().size, 3);
    assert_eq!(allocated_pod.as_pod().pod_header().type_, Type::BITMAP.raw);
    assert_eq!(allocated_pod.as_pod().value().unwrap(), bytes.as_ref());
    assert_ne!(
        allocated_pod.as_pod().value().unwrap(),
        bytes_wrong.as_ref()
    );
}
