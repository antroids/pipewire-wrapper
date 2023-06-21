use pipewire_proc_macro::{RawWrapper, spa_interface};

use crate::spa;
use crate::spa::loop_::LoopControlRef;
use crate::spa::loop_::utils::LoopUtilsRef;
use crate::spa::system::SystemRef;
use crate::wrapper::*;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct LoopRef {
    #[raw]
    raw: pw_sys::pw_loop,
}

impl LoopRef {
    pub fn system(&self) -> &SystemRef {
        unsafe { SystemRef::from_raw_ptr(self.raw.system) }
    }

    pub fn loop_(&self) -> &spa::loop_::LoopRef {
        unsafe { spa::loop_::LoopRef::from_raw_ptr(self.raw.loop_) }
    }

    pub fn control(&self) -> &LoopControlRef {
        unsafe { LoopControlRef::from_raw_ptr(self.raw.control) }
    }

    pub fn utils(&self) -> &LoopUtilsRef {
        unsafe { LoopUtilsRef::from_raw_ptr(self.raw.utils) }
    }

    pub fn iter(&self, timeout_millis: i32) -> LoopRefIterator {
        LoopRefIterator::new(self, timeout_millis)
    }
}

pub struct LoopRefIterator<'a> {
    loop_ref: &'a LoopRef,
    timeout_millis: i32,
}

impl<'a> LoopRefIterator<'a> {
    fn new(loop_ref: &'a LoopRef, timeout_millis: i32) -> Self {
        unsafe {
            loop_ref.control().enter();
        }
        LoopRefIterator {
            timeout_millis,
            loop_ref,
        }
    }

    pub fn set_timeout(&mut self, timeout_millis: i32) {
        self.timeout_millis = timeout_millis;
    }
}

impl Iterator for LoopRefIterator<'_> {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        let timeout_millis = self.timeout_millis;
        let result = unsafe { self.loop_ref.control().iterate(timeout_millis) };
        if let Ok(result) = result {
            if result >= 0 {
                return Some(result);
            }
        }
        None
    }
}

impl Drop for LoopRefIterator<'_> {
    fn drop(&mut self) {
        unsafe {
            self.loop_ref.control().leave();
        }
    }
}
