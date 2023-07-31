/*
 * SPDX-License-Identifier: MIT
 */
use std::fmt::{Debug, Formatter};
use std::io::{ErrorKind, Seek, SeekFrom, Write};
use std::marker::PhantomData;
use std::mem::size_of;
use std::slice;

use crate::spa::pod::restricted::{BasicTypePod, CloneTo, PodHeader};
use crate::spa::pod::{
    PodBoolRef, PodError, PodLongRef, PodRef, PodResult, PodValue, SizedPod, WritePod, POD_ALIGN,
};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

type AlignedDataType = u64;
const ZEROED_ALIGNED_DATA: AlignedDataType = 0;
const DATA_ALIGN: u64 = size_of::<AlignedDataType>() as u64;

pub struct PodBuf<T> {
    data: Vec<AlignedDataType>,
    pos: u64,
    phantom: PhantomData<T>,
}

impl<'a, T> PodBuf<T>
where
    &'a T: WritePod,
    T: 'a,
{
    pub fn from_value(value: &<&'a T as PodValue>::Value) -> PodResult<Self> {
        let mut buf = Self::new();
        <&'a T>::write_pod(&mut buf, value)?;
        Ok(buf)
    }
}

impl<'a, T> PodBuf<T>
where
    &'a T: CloneTo,
    T: 'a,
{
    pub fn from_pod(pod: &'a T) -> PodResult<Self> {
        let mut buf = Self::new();
        pod.clone_to(&mut buf)?;
        Ok(buf)
    }
}

impl<T> PodBuf<T>
where
    T: WritePod,
{
    pub fn from_primitive_value(value: T::Value) -> PodResult<Self> {
        let mut buf = Self::new();
        T::write_pod(&mut buf, &value)?;
        Ok(buf)
    }
}

impl<T> PodBuf<T> {
    pub fn into_pod(self) -> AllocPod<T> {
        AllocPod {
            data: self.data,
            phantom: PhantomData,
        }
    }

    pub(crate) fn new() -> Self {
        Self {
            data: vec![ZEROED_ALIGNED_DATA],
            pos: 0,
            phantom: PhantomData,
        }
    }

    pub(crate) fn with_size(size: usize) -> Self {
        let mut buf = Self::new();
        buf.allocate_data_if_needed(size as u64);
        buf
    }

    /// Append all data from [AllocPod] to the buffer.
    ///
    /// # Note
    ///
    /// Buffer position must be aligned.
    /// The data after current position will be corrupted.
    pub(crate) unsafe fn append_alloc_pod(&mut self, mut alloc_pod: AllocPod<T>) -> PodResult<()> {
        let aligned_size = self.pos / DATA_ALIGN;
        if (aligned_size * DATA_ALIGN - self.pos) == 0 {
            self.data.truncate(aligned_size as usize);
            self.data.append(&mut alloc_pod.data);
            self.pos = self.data_size() as u64;
            Ok(())
        } else {
            Err(PodError::PodIsNotAligned)
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

impl<T> Write for PodBuf<T> {
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

impl<T> Seek for PodBuf<T> {
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

pub struct AllocPod<T> {
    data: Vec<AlignedDataType>,
    phantom: PhantomData<T>,
}

impl<T> Clone for AllocPod<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            phantom: PhantomData,
        }
    }
}

impl<T> AllocPod<T> {
    pub fn as_pod(&self) -> &T {
        unsafe { self.as_ptr().as_ref().unwrap() }
    }

    pub fn as_pod_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.as_mut_ptr()) }
    }

    pub(crate) fn as_ptr(&self) -> *const T {
        self.data.as_ptr() as *const _ as *const T
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr() as *mut _ as *mut T
    }

    pub fn size(&self) -> usize {
        self.data.len() * size_of::<AlignedDataType>()
    }
}

impl<'a, T> AllocPod<T>
where
    &'a T: WritePod,
    T: 'a,
{
    pub fn from_value(value: &<&'a T as PodValue>::Value) -> PodResult<Self> {
        Ok(PodBuf::from_value(value)?.into_pod())
    }
}

impl<'a, T> AllocPod<T>
where
    &'a T: CloneTo,
    T: 'a,
{
    pub fn from_pod(pod: &'a T) -> PodResult<Self> {
        Ok(PodBuf::from_pod(pod)?.into_pod())
    }
}

impl<T> AllocPod<T>
where
    T: WritePod,
{
    pub fn from_primitive_value(value: T::Value) -> PodResult<Self> {
        Ok(PodBuf::from_primitive_value(value)?.into_pod())
    }
}

impl<T: Debug> Debug for AllocPod<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AllocatedData")
            .field("data", &self.data)
            .finish()
    }
}

impl<'a, T> TryFrom<&'a PodRef> for AllocPod<T>
where
    T: 'a,
    &'a T: WritePod,
    T: BasicTypePod,
{
    type Error = PodError;

    fn try_from(value: &'a PodRef) -> Result<Self, Self::Error> {
        let pod: &T = value.cast()?;
        Ok(PodBuf::from_value(&pod.value()?)?.into_pod())
    }
}

#[test]
fn test_buf_from_value() {
    let allocated_pod = PodBuf::<PodBoolRef>::from_primitive_value(true)
        .unwrap()
        .into_pod();
    assert_eq!(allocated_pod.data.as_ptr().align_offset(POD_ALIGN), 0);
    assert_eq!(allocated_pod.as_pod().pod_size(), 12);
    assert_eq!(allocated_pod.as_pod().pod_header().size, 4);
    assert_eq!(allocated_pod.as_pod().pod_header().type_, Type::BOOL.raw);
    assert!(allocated_pod.as_pod().value().unwrap());
    assert_eq!(allocated_pod.as_pod().raw_value(), 1);

    let allocated_pod = PodBuf::<PodLongRef>::from_primitive_value(123456789)
        .unwrap()
        .into_pod();
    assert_eq!(allocated_pod.data.as_ptr().align_offset(POD_ALIGN), 0);
    assert_eq!(allocated_pod.as_pod().pod_size(), 16);
    assert_eq!(allocated_pod.as_pod().pod_header().size, 8);
    assert_eq!(allocated_pod.as_pod().pod_header().type_, Type::LONG.raw);
    assert_eq!(allocated_pod.as_pod().value().unwrap(), 123456789i64);
    assert_eq!(allocated_pod.as_pod().raw_value(), 123456789i64);
}
