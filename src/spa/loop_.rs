use std::fmt::{Debug, Formatter};
use std::os::fd::RawFd;
use std::ptr::NonNull;
use std::rc::Rc;
use std::time::Duration;

use spa_sys::{spa_interface, spa_source, spa_source_io_func_t};

use pipewire_macro_impl::spa_interface_call;
use pipewire_proc_macro::{spa_interface, RawWrapper, Wrapper};

use crate::core_api::main_loop::MainLoop;
use crate::error::Error;
use crate::spa::interface::InterfaceRef;
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
    raw: spa_sys::spa_source,
}

#[derive(Wrapper)]
pub struct IOSource<'l> {
    #[raw_wrapper]
    ref_: NonNull<SourceRef>,
    loop_: &'l dyn AsLoopRef,
    callback: Box<dyn FnMut(RawFd, u32) + 'l>,
}

#[derive(Wrapper)]
pub struct IdleSource<'l> {
    #[raw_wrapper]
    ref_: NonNull<SourceRef>,
    loop_: &'l dyn AsLoopRef,
    callback: Box<dyn FnMut() + 'l>,
}

#[derive(Wrapper)]
pub struct EventSource<'l> {
    #[raw_wrapper]
    ref_: NonNull<SourceRef>,
    loop_: &'l dyn AsLoopRef,
    callback: Box<dyn FnMut(u64) + 'l>,
}

#[derive(Wrapper)]
pub struct TimerSource<'l> {
    #[raw_wrapper]
    ref_: NonNull<SourceRef>,
    loop_: &'l dyn AsLoopRef,
    callback: Box<dyn FnMut(u64) + 'l>,
}

#[derive(Wrapper)]
pub struct SignalSource<'l> {
    #[raw_wrapper]
    ref_: NonNull<SourceRef>,
    loop_: &'l dyn AsLoopRef,
    callback: Box<dyn FnMut(i32) + 'l>,
}

pub trait AsLoopRef: Debug {
    fn loop_(&self) -> &LoopRef;
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

impl Drop for IOSource<'_> {
    fn drop(&mut self) {
        self.loop_.loop_().remove_source(self.as_ref().into());
    }
}

impl Drop for IdleSource<'_> {
    fn drop(&mut self) {
        self.loop_.loop_().remove_source(self.as_ref().into());
    }
}

impl Drop for EventSource<'_> {
    fn drop(&mut self) {
        self.loop_.loop_().remove_source(self.as_ref().into());
    }
}

impl Drop for TimerSource<'_> {
    fn drop(&mut self) {
        self.loop_.loop_().remove_source(self.as_ref().into());
    }
}

impl Drop for SignalSource<'_> {
    fn drop(&mut self) {
        self.loop_.loop_().remove_source(self.as_ref().into());
    }
}

impl Debug for IOSource<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IOSource")
            .field("ref_", &self.ref_)
            .field("loop_", &self.loop_)
            .finish()
    }
}

impl Debug for IdleSource<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IdleSource")
            .field("ref_", &self.ref_)
            .field("loop_", &self.loop_)
            .finish()
    }
}

impl Debug for EventSource<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventSource")
            .field("ref_", &self.ref_)
            .field("loop_", &self.loop_)
            .finish()
    }
}

impl Debug for TimerSource<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TimerSource")
            .field("ref_", &self.ref_)
            .field("loop_", &self.loop_)
            .finish()
    }
}

impl Debug for SignalSource<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalSource")
            .field("ref_", &self.ref_)
            .field("loop_", &self.loop_)
            .finish()
    }
}
