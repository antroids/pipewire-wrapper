use std::io::{ErrorKind, Seek, SeekFrom, Write};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::slice_from_raw_parts;
use std::{io, slice};

use crate::spa::type_::pod::restricted::PodHeader;
use crate::spa::type_::pod::{
    PodBoolRef, PodError, PodLongRef, PodResult, ReadablePod, SizedPod, WritablePod, POD_ALIGN,
};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

type AlignedDataType = u64;
const ZEROED_ALIGNED_DATA: AlignedDataType = 0;
const DATA_ALIGN: u64 = size_of::<AlignedDataType>() as u64;

pub struct PodBuf<'a, T>
where
    T: 'a,
    &'a T: WritablePod,
{
    data: Vec<AlignedDataType>,
    pos: u64,
    phantom: PhantomData<&'a T>,
}

impl<'a, T> PodBuf<'a, T>
where
    &'a T: WritablePod,
{
    pub fn from_value(value: <&'a T as ReadablePod>::Value) -> PodResult<Self> {
        let mut buf = Self::new();
        <&'a T>::write_pod(&mut buf, value)?;
        Ok(buf)
    }

    pub fn into_pod(self) -> AllocatedPod<T> {
        AllocatedPod {
            data: self.data,
            phantom: PhantomData::default(),
        }
    }

    fn new() -> Self {
        Self {
            data: vec![ZEROED_ALIGNED_DATA],
            pos: 0,
            phantom: PhantomData::default(),
        }
    }

    unsafe fn data_bytes_mut(&mut self) -> &mut [u8] {
        slice::from_raw_parts_mut(self.data.as_mut_ptr().cast(), self.data_size())
    }

    unsafe fn data_bytes(&self) -> &[u8] {
        slice::from_raw_parts(self.data.as_ptr().cast(), self.data_size())
    }

    fn data_size(&self) -> usize {
        self.data.len() * DATA_ALIGN as usize
    }

    fn allocate_data_if_needed(&mut self, pos: u64) {
        let data_size = self.data_size() as u64;
        if data_size < pos {
            let required = (pos - data_size + DATA_ALIGN - 1) / DATA_ALIGN;
            for _ in 0..required {
                self.data.push(ZEROED_ALIGNED_DATA);
            }
        }
    }
}

impl<'a, T> PodBuf<'a, T>
where
    &'a T: WritablePod,
    <&'a T as ReadablePod>::Value: Iterator,
    <<&'a T as ReadablePod>::Value as Iterator>::Item: SizedPod,
{
    // todo
}

impl<'a, T> Write for PodBuf<'a, T>
where
    &'a T: WritablePod,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let start_pos = self.pos;
        let end_pos = self
            .pos
            .checked_add(buf.len() as u64)
            .ok_or(std::io::Error::new(
                ErrorKind::InvalidInput,
                "buffer size is too big",
            ))?;
        self.allocate_data_if_needed(end_pos);
        let mut data_bytes = unsafe { self.data_bytes_mut() };
        data_bytes[start_pos as usize..end_pos as usize].copy_from_slice(buf);
        self.pos = end_pos;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a, T> Seek for PodBuf<'a, T>
where
    &'a T: WritablePod,
{
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let (from, offset) = match pos {
            SeekFrom::Start(pos) => {
                self.allocate_data_if_needed(pos);
                self.pos = pos;
                return Ok(pos);
            }
            SeekFrom::End(pos_from_end) => (self.data_size() as u64, pos_from_end),
            SeekFrom::Current(pos_from_current) => (self.pos, pos_from_current),
        };

        if let Some(pos) = from.checked_add_signed(offset) {
            self.seek(SeekFrom::Start(pos))
        } else {
            Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "invalid seek to a negative or overflowing position",
            ))
        }
    }
}

pub struct AllocatedPod<T> {
    data: Vec<AlignedDataType>,
    phantom: PhantomData<T>,
}

impl<T> AllocatedPod<T> {
    pub fn as_pod(&self) -> &T {
        unsafe { (self.data.as_ptr() as *const T).as_ref().unwrap() }
    }

    pub fn as_pod_mut(&mut self) -> &mut T {
        unsafe { (self.data.as_mut_ptr() as *mut T).as_mut().unwrap() }
    }
}

pub struct PodBufFrame<'a, T>
where
    T: 'a,
    T: RawWrapper,
    &'a T: WritablePod,
{
    buf: &'a mut PodBuf<'a, T>,
    start_pos: u64,
}

impl<'a, T> Write for PodBufFrame<'a, T>
where
    T: RawWrapper,
    &'a T: WritablePod,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a, T> Seek for PodBufFrame<'a, T>
where
    T: RawWrapper,
    &'a T: WritablePod,
{
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let (from, offset) = match pos {
            SeekFrom::Start(pos) => return self.buf.seek(SeekFrom::Start(self.start_pos + pos)),
            SeekFrom::End(pos_from_end) => {
                (self.buf.data_size() as u64 - self.start_pos, pos_from_end)
            }
            SeekFrom::Current(pos_from_current) => {
                (self.buf.pos - self.start_pos, pos_from_current)
            }
        };

        if let Some(pos) = from.checked_add_signed(offset) {
            self.seek(SeekFrom::Start(pos))
        } else {
            Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "invalid seek to a negative or overflowing position",
            ))
        }
    }
}

#[test]
fn test_buf_from_value() {
    let allocated_pod = PodBuf::<PodBoolRef>::from_value(true).unwrap().into_pod();
    assert_eq!(allocated_pod.data.as_ptr().align_offset(POD_ALIGN), 0);
    assert_eq!(allocated_pod.as_pod().pod_size(), 12);
    assert_eq!(allocated_pod.as_pod().pod_header().size, 4);
    assert_eq!(allocated_pod.as_pod().pod_header().type_, Type::BOOL.raw);
    assert_eq!(allocated_pod.as_pod().value().unwrap(), true);
    assert_eq!(allocated_pod.as_pod().raw_value(), 1);

    let allocated_pod = PodBuf::<PodLongRef>::from_value(123456789)
        .unwrap()
        .into_pod();
    assert_eq!(allocated_pod.data.as_ptr().align_offset(POD_ALIGN), 0);
    assert_eq!(allocated_pod.as_pod().pod_size(), 16);
    assert_eq!(allocated_pod.as_pod().pod_header().size, 8);
    assert_eq!(allocated_pod.as_pod().pod_header().type_, Type::LONG.raw);
    assert_eq!(allocated_pod.as_pod().value().unwrap(), 123456789i64);
    assert_eq!(allocated_pod.as_pod().raw_value(), 123456789i64);
}
