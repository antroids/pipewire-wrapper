/*
 * SPDX-License-Identifier: MIT
 */

//! PipeWire [Loop](https://docs.pipewire.org/group__pw__loop.html) bindings.
//!
use std::os::fd::RawFd;
use std::time::Duration;

use crate::core_api::loop_::restricted::AsLoopRef;
use pipewire_wrapper_proc_macro::RawWrapper;

use crate::spa;
use crate::spa::loop_::utils::LoopUtilsRef;
use crate::spa::loop_::LoopControlRef;
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

pub(crate) mod restricted {
    use crate::core_api::loop_::LoopRef;
    use crate::spa;
    use crate::spa::loop_::utils::LoopUtilsRef;
    use crate::wrapper::RawWrapper;
    use std::fmt::Debug;
    use std::time::Duration;

    pub trait AsLoopRef: Debug + Sized {
        fn loop_(&self) -> &LoopRef;

        fn utils(&self) -> &LoopUtilsRef<Self> {
            unsafe { LoopUtilsRef::from_raw_ptr(self.loop_().raw.utils) }
        }

        fn update_io(&self, source: &spa::loop_::IOSource<Self>, mask: u32) -> crate::Result<()> {
            self.utils().update_io(source, mask)
        }

        fn enable_idle(
            &self,
            source: &spa::loop_::IdleSource<Self>,
            enabled: bool,
        ) -> crate::Result<()> {
            self.utils().enable_idle(source, enabled)
        }

        fn signal_event(&self, source: &spa::loop_::EventSource<Self>) -> crate::Result<()> {
            self.utils().signal_event(source)
        }

        fn update_timer(
            &self,
            source: &spa::loop_::TimerSource<Self>,
            value: Duration,
            interval: Duration,
            absolute: bool,
        ) -> crate::Result<()> {
            self.utils().update_timer(source, value, interval, absolute)
        }

        fn disable_timer(&self, source: &spa::loop_::TimerSource<Self>) -> crate::Result<()> {
            self.utils().disable_timer(source)
        }
    }
}

impl AsLoopRef for LoopRef {
    fn loop_(&self) -> &LoopRef {
        self
    }
}

impl<T: AsLoopRef> spa::loop_::restricted::AsLoopRef for T {
    fn loop_(&self) -> &spa::loop_::LoopRef {
        self.loop_().loop_()
    }
}

pub trait Loop: AsLoopRef + Clone + 'static {
    fn add_io<F>(
        &self,
        fd: RawFd,
        mask: u32,
        callback: F,
    ) -> crate::Result<spa::loop_::IOSource<'static, Self>>
    where
        F: FnMut(RawFd, u32) + 'static,
    {
        self.utils()
            .add_io(self.clone(), fd, mask, Box::new(callback))
    }

    fn add_idle<F>(
        &self,
        enabled: bool,
        callback: F,
    ) -> crate::Result<spa::loop_::IdleSource<'static, Self>>
    where
        F: FnMut() + 'static,
    {
        self.utils()
            .add_idle(self.clone(), enabled, Box::new(callback))
    }

    fn add_event<F>(&self, callback: F) -> crate::Result<spa::loop_::EventSource<'static, Self>>
    where
        F: FnMut(u64) + 'static,
    {
        self.utils().add_event(self.clone(), Box::new(callback))
    }

    fn add_timer<F>(&self, callback: F) -> crate::Result<spa::loop_::TimerSource<'static, Self>>
    where
        F: FnMut(u64) + 'static,
    {
        self.utils().add_timer(self.clone(), Box::new(callback))
    }

    fn add_signal<F>(
        &self,
        signal_number: i32,
        callback: F,
    ) -> crate::Result<spa::loop_::SignalSource<'static, Self>>
    where
        F: FnMut(i32) + 'static,
    {
        self.utils()
            .add_signal(self.clone(), signal_number, Box::new(callback))
    }

    /// Must be called inside loop or when loop is not running
    fn destroy_source(&self, source: &spa::loop_::SourceRef) -> crate::Result<()> {
        self.utils().destroy_source(source)
    }
}

impl<T: AsLoopRef + Clone + 'static> Loop for T {}

impl<T: Loop> spa::loop_::IOSource<'static, T> {
    pub fn update(&self, mask: u32) -> crate::Result<()> {
        self.loop_.update_io(self, mask)
    }
}

impl<T: Loop> spa::loop_::IdleSource<'static, T> {
    pub fn enable(&self, enabled: bool) -> crate::Result<()> {
        self.loop_.enable_idle(self, enabled)
    }
}

impl<T: Loop> spa::loop_::EventSource<'static, T> {
    pub fn signal(&self) -> crate::Result<()> {
        self.loop_.signal_event(self)
    }
}

impl<T: Loop> spa::loop_::TimerSource<'static, T> {
    pub fn update(&self, value: Duration, interval: Duration, absolute: bool) -> crate::Result<()> {
        self.loop_.update_timer(self, value, interval, absolute)
    }

    pub fn disable(&self) -> crate::Result<()> {
        self.loop_.disable_timer(self)
    }
}
