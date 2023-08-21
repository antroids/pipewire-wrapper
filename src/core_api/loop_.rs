/*
 * SPDX-License-Identifier: MIT
 */

//! PipeWire [Loop](https://docs.pipewire.org/group__pw__loop.html) bindings.
//!
use std::os::fd::RawFd;
use std::time::Duration;

use pipewire_wrapper_proc_macro::RawWrapper;

use crate::spa;
use crate::spa::loop_::restricted::AsLoopRef;
use crate::spa::loop_::utils::LoopUtilsRef;
use crate::spa::loop_::{
    EventSource, IOSource, IdleSource, LoopControlRef, SignalSource, SourceRef, TimerSource,
};
use crate::spa::system::SystemRef;
use crate::wrapper::*;

pub mod channel;

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

    pub fn utils(&self) -> &LoopUtilsRef<Self> {
        unsafe { LoopUtilsRef::from_raw_ptr(self.raw.utils) }
    }

    pub fn iter(&self, timeout_millis: i32) -> LoopRefIterator {
        LoopRefIterator::new(self, timeout_millis)
    }
}

impl LoopRef {
    pub fn add_io<'l, F>(
        &'l self,
        fd: RawFd,
        mask: u32,
        callback: Box<F>,
    ) -> crate::Result<IOSource<'l, Self>>
    where
        F: FnMut(RawFd, u32),
        F: 'l,
    {
        self.utils().add_io(self, fd, mask, callback)
    }

    pub fn update_io(&self, source: &IOSource<Self>, mask: u32) -> crate::Result<()> {
        self.utils().update_io(source, mask)
    }

    pub fn add_idle<'l, F>(
        &'l self,
        enabled: bool,
        callback: Box<F>,
    ) -> crate::Result<IdleSource<'l, Self>>
    where
        F: FnMut(),
        F: 'l,
    {
        self.utils().add_idle(self, enabled, callback)
    }

    pub fn enable_idle(&self, source: &IdleSource<Self>, enabled: bool) -> crate::Result<()> {
        self.utils().enable_idle(source, enabled)
    }

    pub fn add_event<'l, F>(&'l self, callback: Box<F>) -> crate::Result<EventSource<'l, Self>>
    where
        F: FnMut(u64),
        F: 'l,
    {
        self.utils().add_event(self, callback)
    }

    pub fn signal_event(&self, source: &EventSource<Self>) -> crate::Result<()> {
        self.utils().signal_event(source)
    }

    pub fn add_timer<'l, F>(&'l self, callback: Box<F>) -> crate::Result<TimerSource<'l, Self>>
    where
        F: FnMut(u64),
        F: 'l,
    {
        self.utils().add_timer(self, callback)
    }

    pub fn update_timer(
        &self,
        source: &TimerSource<Self>,
        value: Duration,
        interval: Duration,
        absolute: bool,
    ) -> crate::Result<()> {
        self.utils().update_timer(source, value, interval, absolute)
    }

    pub fn disable_timer(&self, source: &TimerSource<Self>) -> crate::Result<()> {
        self.utils().disable_timer(source)
    }

    pub fn add_signal<'l, F>(
        &'l self,
        signal_number: i32,
        callback: Box<F>,
    ) -> crate::Result<SignalSource<'l, Self>>
    where
        F: FnMut(i32),
        F: 'l,
    {
        self.utils().add_signal(self, signal_number, callback)
    }

    /// Must be called inside loop or when loop is not running
    pub fn destroy_source(&self, source: &SourceRef) -> crate::Result<()> {
        self.utils().destroy_source(source)
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

impl AsLoopRef for LoopRef {
    fn loop_(&self) -> &spa::loop_::LoopRef {
        self.loop_()
    }
}

impl<'l> IOSource<'l, LoopRef> {
    pub fn update(&self, mask: u32) -> crate::Result<()> {
        self.loop_.update_io(self, mask)
    }
}

impl<'l> IdleSource<'l, LoopRef> {
    pub fn enable(&self, enabled: bool) -> crate::Result<()> {
        self.loop_.enable_idle(self, enabled)
    }
}

impl<'l> EventSource<'l, LoopRef> {
    pub fn signal(&self) -> crate::Result<()> {
        self.loop_.signal_event(self)
    }
}

impl<'l> TimerSource<'l, LoopRef> {
    pub fn update(&self, value: Duration, interval: Duration, absolute: bool) -> crate::Result<()> {
        self.loop_.update_timer(self, value, interval, absolute)
    }

    pub fn disable(&self) -> crate::Result<()> {
        self.loop_.disable_timer(self)
    }
}
