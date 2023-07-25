use std::ffi::{c_char, CStr, CString};
use std::fmt::{Debug, Formatter};
use std::io::{Seek, Write};

use pipewire_proc_macro::RawWrapper;

use crate::spa::pod::pod_buf::PodBuf;
use crate::spa::pod::restricted::{PodHeader, PodRawValue, StaticTypePod};
use crate::spa::pod::{
    BasicTypePod, FromValue, PodError, PodResult, PodValue, SizedPod, WritePod, WriteValue,
    POD_ALIGN,
};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodStringRef {
    #[raw]
    raw: spa_sys::spa_pod_string,
}

impl PodHeader for PodStringRef {
    fn pod_header(&self) -> &spa_sys::spa_pod {
        &self.raw.pod
    }
}

impl StaticTypePod for PodStringRef {
    fn static_type() -> Type {
        Type::STRING
    }
}

impl PodStringRef {
    fn content_size(&self) -> usize {
        self.raw.pod.size as usize
    }

    unsafe fn content_ptr(&self) -> *const c_char {
        (self as *const Self).offset(1).cast()
    }
}

impl<'a> PodRawValue for &'a PodStringRef {
    type RawValue = c_char;

    fn raw_value_ptr(&self) -> *const Self::RawValue {
        unsafe { (&self.raw.pod as *const spa_sys::spa_pod).offset(1).cast() }
    }

    fn parse_raw_value(ptr: *const Self::RawValue, size: usize) -> PodResult<Self::Value> {
        unsafe {
            if *(ptr as *const u8).add((size - 1)) != 0 {
                Err(PodError::StringIsNotNullTerminated)
            } else {
                Ok(CStr::from_ptr(ptr))
            }
        }
    }
}

impl<'a> PodValue for &'a PodStringRef {
    type Value = &'a CStr;
    fn value(&self) -> PodResult<Self::Value> {
        Self::parse_raw_value(self.raw_value_ptr(), self.pod_header().size as usize)
    }
}

impl<'a> WritePod for &'a PodStringRef {
    fn write_pod<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        let string_bytes = value.to_bytes_with_nul();
        let header_size = Self::write_header(
            buffer,
            string_bytes.len() as u32,
            PodStringRef::static_type(),
        )?;
        buffer.write_all(string_bytes)?;
        Ok(header_size + string_bytes.len() + Self::write_align_padding(buffer)?)
    }
}

impl<'a> WriteValue for &'a PodStringRef {
    fn write_raw_value<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        let string_bytes = value.to_bytes_with_nul();
        buffer.write_all(string_bytes)?;
        Ok(string_bytes.len())
    }
}

impl Debug for PodStringRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("PodStringRef")
                .field("pod.type", &self.pod_type())
                .field("pod.size", &self.pod_size())
                .field("value", &self.value())
                .finish()
        }
    }
}

#[test]
fn test_from_value() {
    let string = CString::new("Test string").unwrap();
    let string_wrong = CString::new("Test string wrong").unwrap();
    let allocated_pod = PodStringRef::from_value(&string.as_ref()).unwrap();
    assert_eq!(allocated_pod.as_pod().as_ptr().align_offset(POD_ALIGN), 0);
    assert_eq!(allocated_pod.as_pod().pod_size(), 20);
    assert_eq!(allocated_pod.as_pod().pod_header().size, 12);
    assert_eq!(allocated_pod.as_pod().pod_header().type_, Type::STRING.raw);
    assert_eq!(allocated_pod.as_pod().value().unwrap(), string.as_ref());
    assert_ne!(
        allocated_pod.as_pod().value().unwrap(),
        string_wrong.as_ref()
    );
}
