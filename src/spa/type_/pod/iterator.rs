use core::slice;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::mem::size_of;

use crate::spa::type_::pod::{BasicTypePod, PodValueParser, SizedPod, POD_ALIGN};

pub struct PodIterator<'a, C: SizedPod, E: SizedPod> {
    container: &'a C,
    first_element_ptr: *const E,
    current_element_ptr: *const E,
}

impl<'a, C: SizedPod, E: SizedPod> PodIterator<'a, C, E> {
    pub fn new(container: &'a C) -> Self {
        unsafe {
            let first_element_ptr = (container as *const C).offset(1).cast();
            Self {
                container,
                first_element_ptr,
                current_element_ptr: first_element_ptr,
            }
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
        self.container.pod_size() - size_of::<C>()
    }

    pub(crate) unsafe fn as_bytes(&self) -> &[u8] {
        slice::from_raw_parts(self.first_element_ptr as *const u8, self.max_offset_bytes())
    }
}

impl<'a, C: SizedPod, E: SizedPod + 'a> Iterator for PodIterator<'a, C, E> {
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

impl<'a, C: SizedPod, E: SizedPod + 'a> Debug for PodIterator<'_, C, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodIterator").finish()
    }
}

pub struct PodValueIterator<'a, E: PodValueParser<*const u8>> {
    size: usize,
    element_size: usize,
    first_element_ptr: *const E::Value,
    current_element_ptr: *const E::Value,
    phantom: PhantomData<&'a ()>,
}

impl<'a, E: PodValueParser<*const u8>> PodValueIterator<'a, E> {
    pub fn new(first_element_ptr: *const E::Value, size: usize, element_size: usize) -> Self {
        Self {
            size,
            element_size,
            first_element_ptr,
            current_element_ptr: first_element_ptr,
            phantom: PhantomData::default(),
        }
    }

    unsafe fn inside(&self, ptr: *const E::Value) -> bool {
        let max_offset_bytes = self.size;
        let offset_bytes =
            (ptr as *const u8).offset_from(self.first_element_ptr as *const u8) as usize;
        offset_bytes < max_offset_bytes && (offset_bytes + self.element_size) <= max_offset_bytes
    }

    unsafe fn next_element_ptr(&self) -> *const E::Value {
        let ptr = self.current_element_ptr;
        let size = self.element_size;
        (ptr as *const u8).offset(size as isize).cast()
    }
}

impl<'a, E: PodValueParser<*const u8> + 'a> Iterator for PodValueIterator<'a, E> {
    type Item = E::Value;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let current_element_ptr = self.current_element_ptr;
            if self.inside(current_element_ptr) {
                self.current_element_ptr = self.next_element_ptr();
                E::parse(self.element_size, current_element_ptr.cast()).ok()
            } else {
                None
            }
        }
    }
}

impl<'a, E: PodValueParser<*const u8> + 'a> Debug for PodValueIterator<'a, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodValueParser").finish()
    }
}
