use std::ffi::{c_char, CStr, CString};
use std::fmt::{Debug, Formatter};
use std::io::{Seek, Write};

use pipewire_proc_macro::RawWrapper;

use crate::spa::type_::pod::pod_buf::PodBuf;
use crate::spa::type_::pod::restricted::{PodHeader, StaticTypePod};
use crate::spa::type_::pod::{
    BasicTypePod, PodError, PodResult, PodValueParser, ReadablePod, SizedPod, WritablePod,
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

impl<'a> PodValueParser<*const c_char> for &'a PodStringRef {
    fn parse(
        content_size: usize,
        header_or_value: *const c_char,
    ) -> PodResult<<Self as ReadablePod>::Value> {
        unsafe {
            if *header_or_value.offset((content_size - 1) as isize) != 0 {
                Err(PodError::StringIsNotNullTerminated)
            } else {
                Ok(CStr::from_ptr(header_or_value))
            }
        }
    }
}

impl<'a> PodValueParser<*const u8> for &'a PodStringRef {
    fn parse(
        content_size: usize,
        header_or_value: *const u8,
    ) -> PodResult<<Self as ReadablePod>::Value> {
        Self::parse(content_size, header_or_value as *const c_char)
    }
}

impl<'a> ReadablePod for &'a PodStringRef {
    type Value = &'a CStr;

    fn value(&self) -> PodResult<Self::Value> {
        unsafe { Self::parse(self.content_size(), self.content_ptr()) }
    }
}

impl<'a> WritablePod for &'a PodStringRef {
    fn write_pod<W>(buffer: &mut W, value: &<Self as ReadablePod>::Value) -> PodResult<usize>
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

    fn write_raw_value<W>(buffer: &mut W, value: &<Self as ReadablePod>::Value) -> PodResult<usize>
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
                .field("pod.type", &self.upcast().type_())
                .field("pod.size", &self.upcast().size())
                .field("value", &self.value())
                .finish()
        }
    }
}

#[test]
fn test_from_value() {
    let string = CString::new("Test string").unwrap();
    let string_wrong = CString::new("Test string wrong").unwrap();
    let allocated_pod = PodBuf::<PodStringRef>::from_value(&string.as_ref())
        .unwrap()
        .into_pod();
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
