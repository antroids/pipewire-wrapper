use core::slice;
use std::ffi::{CStr, CString};
use std::fmt::{Debug, Formatter};
use std::io::{Seek, Write};
use std::marker::PhantomData;
use std::mem::size_of;

use crate::spa::pod::pod_buf::{AllocatedData, PodBuf};
use crate::spa::pod::restricted::{CloneTo, PodHeader, PodRawValue};
use crate::spa::pod::string::PodStringRef;
use crate::spa::pod::{
    BasicTypePod, PodIntRef, PodResult, PodValue, SizedPod, WritePod, POD_ALIGN,
};

pub struct PodIterator<'a, E: SizedPod> {
    size: usize,
    first_element_ptr: *const E,
    current_element_ptr: *const E,
    phantom: PhantomData<&'a E>,
}

impl<'a, E: SizedPod> PodIterator<'a, E> {
    pub fn from_container<C: SizedPod>(container: &'a C) -> Self {
        unsafe {
            let first_element_ptr = (container as *const C).offset(1).cast();
            Self::new(first_element_ptr, container.pod_size() - size_of::<C>())
        }
    }

    pub fn new(first_element_ptr: *const E, size: usize) -> Self {
        Self {
            size,
            first_element_ptr,
            current_element_ptr: first_element_ptr,
            phantom: PhantomData,
        }
    }

    pub fn build() -> PodIteratorBuilder<'a, E> {
        PodIteratorBuilder::new()
    }

    unsafe fn inside(&self, ptr: *const E) -> bool {
        let max_offset_bytes = self.max_offset_bytes();
        let offset_bytes =
            (ptr as *const u8).offset_from(self.first_element_ptr as *const u8) as usize;
        offset_bytes < max_offset_bytes && (offset_bytes + (*ptr).pod_size()) <= max_offset_bytes
    }

    unsafe fn next_element_ptr(&self) -> *const E {
        let ptr = self.current_element_ptr;
        let size = (*ptr).pod_size();
        let next_ptr = (ptr as *const u8).offset(size as isize);
        let aligned = next_ptr
            .offset(next_ptr.align_offset(POD_ALIGN) as isize)
            .cast();
        aligned
    }

    fn max_offset_bytes(&self) -> usize {
        self.size
    }

    pub(crate) unsafe fn as_bytes(&self) -> &[u8] {
        slice::from_raw_parts(self.first_element_ptr as *const u8, self.max_offset_bytes())
    }
}

impl<'a, E: SizedPod + 'a> Iterator for PodIterator<'a, E> {
    type Item = &'a E;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let current_element_ptr = self.current_element_ptr;
            if self.inside(current_element_ptr) {
                self.current_element_ptr = self.next_element_ptr();
                Some(&*current_element_ptr)
            } else {
                None
            }
        }
    }
}

impl<'a, E: SizedPod + 'a> Debug for PodIterator<'_, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodIterator").finish()
    }
}

pub struct PodValueIterator<'a, E: PodRawValue> {
    size: usize,
    element_size: usize,
    first_element_ptr: *const E::RawValue,
    current_element_ptr: *const E::RawValue,
    phantom: PhantomData<&'a ()>,
}

impl<'a, E: PodRawValue> PodValueIterator<'a, E> {
    pub fn new(first_element_ptr: *const E::RawValue, size: usize, element_size: usize) -> Self {
        Self {
            size,
            element_size,
            first_element_ptr,
            current_element_ptr: first_element_ptr,
            phantom: PhantomData,
        }
    }

    unsafe fn inside(&self, ptr: *const E::RawValue) -> bool {
        let max_offset_bytes = self.size;
        let offset_bytes =
            (ptr as *const u8).offset_from(self.first_element_ptr as *const u8) as usize;
        offset_bytes < max_offset_bytes && (offset_bytes + self.element_size) <= max_offset_bytes
    }

    unsafe fn next_element_ptr(&self) -> *const E::RawValue {
        let ptr = self.current_element_ptr;
        let size = self.element_size;
        (ptr as *const u8).offset(size as isize).cast()
    }
}

impl<'a, E: PodRawValue + 'a> Iterator for PodValueIterator<'a, E> {
    type Item = E::Value;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let current_element_ptr = self.current_element_ptr;
            if self.inside(current_element_ptr) {
                self.current_element_ptr = self.next_element_ptr();
                E::parse_raw_value(current_element_ptr, self.element_size).ok()
            } else {
                None
            }
        }
    }
}

impl<'a, E: PodRawValue + 'a> Debug for PodValueIterator<'a, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodValueIterator").finish()
    }
}

#[repr(transparent)]
pub struct AllocatedPodIterator<E: SizedPod> {
    data: AllocatedData<E>,
}

impl<E: SizedPod> AllocatedPodIterator<E> {
    pub fn iter(&self) -> PodIterator<E> {
        PodIterator::new(self.data.as_ptr(), self.data.size())
    }
}

impl<'a, E> AllocatedPodIterator<E>
where
    E: 'a,
    &'a E: WritePod,
    E: SizedPod,
{
    pub fn from_values(
        values: impl IntoIterator<Item = &'a <&'a E as PodValue>::Value>,
    ) -> PodResult<Self> {
        Ok(PodIteratorBuilder::from_values(values)?.into_pod_iter())
    }
}

pub struct PodIteratorBuilder<'a, E: SizedPod> {
    buf: PodBuf<'a, E>,
}

impl<'a, E: SizedPod> PodIteratorBuilder<'a, E> {
    pub fn new() -> Self {
        Self { buf: PodBuf::new() }
    }

    pub fn into_pod_iter(self) -> AllocatedPodIterator<E> {
        AllocatedPodIterator {
            data: self.buf.into_pod(),
        }
    }
}

impl<'a, E: SizedPod + PodHeader> PodIteratorBuilder<'a, E> {
    pub fn push_pod(mut self, pod_value: &E) -> PodResult<Self> {
        pod_value.clone_to(&mut self.buf)?;
        Ok(self)
    }
}

impl<'a, E> PodIteratorBuilder<'a, E>
where
    &'a E: WritePod,
    E: SizedPod,
{
    pub fn push_value(mut self, pod_value: &<&'a E as PodValue>::Value) -> PodResult<Self> {
        <&'a E as WritePod>::write_pod(&mut self.buf, pod_value)?;
        Ok(self)
    }

    pub fn from_values(
        values: impl IntoIterator<Item = &'a <&'a E as PodValue>::Value>,
    ) -> PodResult<Self> {
        let mut builder = Self::new();
        for v in values {
            builder = builder.push_value(v)?;
        }
        Ok(builder)
    }
}

#[test]
fn test_from_values() {
    let mut builder: PodIteratorBuilder<PodIntRef> = PodIteratorBuilder::new()
        .push_value(&123)
        .unwrap()
        .push_value(&1)
        .unwrap()
        .push_value(&2)
        .unwrap()
        .push_value(&3)
        .unwrap()
        .push_value(&4)
        .unwrap();
    let allocated_iter = builder.into_pod_iter();

    let v: Vec<i32> = allocated_iter.iter().map(|e| e.value().unwrap()).collect();
    assert_eq!(v, vec![123, 1, 2, 3, 4]);

    let mut builder: PodIteratorBuilder<PodIntRef> =
        PodIteratorBuilder::from_values([&123, &1, &2, &3, &4]).unwrap();
    let allocated_iter = builder.into_pod_iter();

    let v: Vec<i32> = allocated_iter.iter().map(|e| e.value().unwrap()).collect();
    assert_eq!(v, vec![123, 1, 2, 3, 4]);
}

#[test]
fn test_from_pods() {
    let mut builder: PodIteratorBuilder<PodStringRef> = PodIteratorBuilder::new();

    let string = CString::new("asd").unwrap();
    let allocated_pod = PodBuf::<PodStringRef>::from_value(&string.as_ref())
        .unwrap()
        .into_pod();
    builder = builder.push_pod(allocated_pod.as_pod()).unwrap();

    let string = CString::new("def").unwrap();
    let allocated_pod = PodBuf::<PodStringRef>::from_value(&string.as_ref())
        .unwrap()
        .into_pod();
    builder = builder.push_pod(allocated_pod.as_pod()).unwrap();

    let allocated_iter = builder.into_pod_iter();

    let v: Vec<&CStr> = allocated_iter.iter().map(|e| e.value().unwrap()).collect();
    assert_eq!(v.get(0).unwrap(), &CString::new("asd").unwrap().as_ref());
    assert_eq!(v.get(1).unwrap(), &CString::new("def").unwrap().as_ref());
}
