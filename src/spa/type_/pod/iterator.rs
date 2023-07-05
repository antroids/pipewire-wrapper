use core::slice;
use std::ffi::{CStr, CString};
use std::fmt::{Debug, Formatter};
use std::io::{Seek, Write};
use std::marker::PhantomData;
use std::mem::size_of;

use crate::spa::type_::pod::pod_buf::{AllocatedData, PodBuf};
use crate::spa::type_::pod::restricted::{CloneTo, PodHeader};
use crate::spa::type_::pod::string::PodStringRef;
use crate::spa::type_::pod::{
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
            phantom: PhantomData::default(),
        }
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

pub struct PodValueIterator<'a, E: PodValue> {
    size: usize,
    element_size: usize,
    first_element_ptr: *const E::RawValue,
    current_element_ptr: *const E::RawValue,
    phantom: PhantomData<&'a ()>,
}

impl<'a, E: PodValue> PodValueIterator<'a, E> {
    pub fn new(first_element_ptr: *const E::RawValue, size: usize, element_size: usize) -> Self {
        Self {
            size,
            element_size,
            first_element_ptr,
            current_element_ptr: first_element_ptr,
            phantom: PhantomData::default(),
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

impl<'a, E: PodValue + 'a> Iterator for PodValueIterator<'a, E> {
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

impl<'a, E: PodValue + 'a> Debug for PodValueIterator<'a, E> {
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
    pub fn push_pod(&mut self, pod_value: &E) -> PodResult<usize> {
        pod_value.clone_to(&mut self.buf)
    }
}

impl<'a, E: SizedPod + WritePod> PodIteratorBuilder<'a, E> {
    pub fn push_value(&mut self, pod_value: &E::Value) -> PodResult<usize> {
        E::write_pod(&mut self.buf, pod_value)
    }
}

#[test]
fn test_from_values() {
    let mut builder: PodIteratorBuilder<PodIntRef> = PodIteratorBuilder::new();
    builder.push_value(&123).unwrap();
    builder.push_value(&1).unwrap();
    builder.push_value(&2).unwrap();
    builder.push_value(&3).unwrap();
    builder.push_value(&4).unwrap();
    let allocated_iter = builder.into_pod_iter();

    let v: Vec<i32> = allocated_iter.iter().map(|e| e.value().unwrap()).collect();
    assert_eq!(v, vec![123, 1, 2, 3, 4])
}

#[test]
fn test_from_pods() {
    let mut builder: PodIteratorBuilder<PodStringRef> = PodIteratorBuilder::new();

    let string = CString::new("asd").unwrap();
    let allocated_pod = PodBuf::<PodStringRef>::from_value(&string.as_ref())
        .unwrap()
        .into_pod();
    builder.push_pod(allocated_pod.as_pod()).unwrap();

    let string = CString::new("def").unwrap();
    let allocated_pod = PodBuf::<PodStringRef>::from_value(&string.as_ref())
        .unwrap()
        .into_pod();
    builder.push_pod(allocated_pod.as_pod()).unwrap();

    let allocated_iter = builder.into_pod_iter();

    let v: Vec<&CStr> = allocated_iter.iter().map(|e| e.value().unwrap()).collect();
    assert_eq!(v.get(0).unwrap(), &CString::new("asd").unwrap().as_ref());
    assert_eq!(v.get(1).unwrap(), &CString::new("def").unwrap().as_ref());
}
