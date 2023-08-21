/*
 * SPDX-License-Identifier: MIT
 */
use std::fmt::{Debug, Formatter};
use std::os::fd::RawFd;
use std::ptr::NonNull;

use spa_sys::spa_source;

use pipewire_wrapper_proc_macro::{spa_interface, RawWrapper, Wrapper};

use crate::error::Error;
use crate::spa::loop_::restricted::AsLoopRef;
use crate::spa_interface_call;
use crate::wrapper::{RawWrapper, SpaInterface, Wrapper};
use crate::{i32_as_result, i32_as_void_result};

pub mod utils;

#[derive(RawWrapper, Debug)]
#[spa_interface(methods=spa_sys::spa_loop_methods)]
#[repr(transparent)]
pub struct LoopRef {
    #[raw]
    raw: spa_sys::spa_loop,
}

#[derive(RawWrapper, Debug)]
#[spa_interface(methods=spa_sys::spa_loop_control_methods)]
#[repr(transparent)]
pub struct LoopControlRef {
    #[raw]
    raw: spa_sys::spa_loop_control,
}

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct SourceRef {
    #[raw]
    raw: spa_source,
}

#[derive(Wrapper)]
pub struct IOSource<'l, T: AsLoopRef> {
    #[raw_wrapper]
    ref_: NonNull<SourceRef>,
    pub(crate) loop_: &'l T,
    callback: Box<dyn FnMut(RawFd, u32) + 'l>,
}

#[derive(Wrapper)]
pub struct IdleSource<'l, T: AsLoopRef> {
    #[raw_wrapper]
    ref_: NonNull<SourceRef>,
    pub(crate) loop_: &'l T,
    callback: Box<dyn FnMut() + 'l>,
}

#[derive(Wrapper)]
pub struct EventSource<'l, T: AsLoopRef> {
    #[raw_wrapper]
    ref_: NonNull<SourceRef>,
    pub(crate) loop_: &'l T,
    callback: Box<dyn FnMut(u64) + 'l>,
}

#[derive(Wrapper)]
pub struct TimerSource<'l, T: AsLoopRef> {
    #[raw_wrapper]
    ref_: NonNull<SourceRef>,
    pub(crate) loop_: &'l T,
    callback: Box<dyn FnMut(u64) + 'l>,
}

#[derive(Wrapper)]
pub struct SignalSource<'l, T: AsLoopRef> {
    #[raw_wrapper]
    ref_: NonNull<SourceRef>,
    pub(crate) loop_: &'l T,
    callback: Box<dyn FnMut(i32) + 'l>,
}

pub(crate) mod restricted {
    use std::fmt::Debug;

    use crate::spa::loop_::LoopRef;

    pub trait AsLoopRef: Debug {
        fn loop_(&self) -> &LoopRef;
    }
}

impl LoopRef {
    pub fn add_source(&self, source: &SourceRef) -> crate::Result<RawFd> {
        let result = spa_interface_call!(self, add_source, source.as_raw_ptr())?;
        i32_as_result(result, result)
    }

    pub fn update_source(&self, source: &SourceRef) -> crate::Result<()> {
        let result = spa_interface_call!(self, update_source, source.as_raw_ptr())?;
        i32_as_void_result(result)
    }

    pub fn remove_source(&self, source: &SourceRef) -> crate::Result<()> {
        let result = spa_interface_call!(self, remove_source, source.as_raw_ptr())?;
        i32_as_void_result(result)
    }

    //todo invoke
}

impl LoopControlRef {
    pub fn get_fd(&self) -> crate::Result<RawFd> {
        spa_interface_call!(self, get_fd)
    }

    //todo pub fn add_hook

    pub fn enter(&self) -> crate::Result<()> {
        spa_interface_call!(self, enter)
    }

    pub fn leave(&self) -> crate::Result<()> {
        spa_interface_call!(self, leave)
    }

    pub fn iterate(&self, timeout: i32) -> crate::Result<i32> {
        spa_interface_call!(self, iterate, timeout)
    }
}

impl<T: AsLoopRef> Drop for IOSource<'_, T> {
    fn drop(&mut self) {
        self.loop_.loop_().remove_source(self.as_ref());
    }
}

impl<T: AsLoopRef> Drop for IdleSource<'_, T> {
    fn drop(&mut self) {
        self.loop_.loop_().remove_source(self.as_ref());
    }
}

impl<T: AsLoopRef> Drop for EventSource<'_, T> {
    fn drop(&mut self) {
        self.loop_.loop_().remove_source(self.as_ref());
    }
}

impl<T: AsLoopRef> Drop for TimerSource<'_, T> {
    fn drop(&mut self) {
        self.loop_.loop_().remove_source(self.as_ref());
    }
}

impl<T: AsLoopRef> Drop for SignalSource<'_, T> {
    fn drop(&mut self) {
        self.loop_.loop_().remove_source(self.as_ref());
    }
}

impl<T: AsLoopRef> Debug for IOSource<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IOSource")
            .field("ref_", &self.ref_)
            .field("loop_", &self.loop_)
            .finish()
    }
}

impl<T: AsLoopRef> Debug for IdleSource<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IdleSource")
            .field("ref_", &self.ref_)
            .field("loop_", &self.loop_)
            .finish()
    }
}

impl<T: AsLoopRef> Debug for EventSource<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventSource")
            .field("ref_", &self.ref_)
            .field("loop_", &self.loop_)
            .finish()
    }
}

impl<T: AsLoopRef> Debug for TimerSource<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TimerSource")
            .field("ref_", &self.ref_)
            .field("loop_", &self.loop_)
            .finish()
    }
}

impl<T: AsLoopRef> Debug for SignalSource<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalSource")
            .field("ref_", &self.ref_)
            .field("loop_", &self.loop_)
            .finish()
    }
}
