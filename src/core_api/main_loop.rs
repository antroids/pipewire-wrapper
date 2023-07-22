use std::ffi::CStr;
use std::fmt::{Debug, Formatter};
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::os::fd::RawFd;
use std::os::raw;
use std::pin::Pin;
use std::ptr::{addr_of, addr_of_mut, NonNull};
use std::rc::Rc;
use std::time::Duration;

use pw_sys::pw_main_loop_events;

use pipewire_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::loop_::{LoopRef, LoopRefIterator};
use crate::core_api::properties::Properties;
use crate::core_api::PipeWire;
use crate::spa::dict::DictRef;
use crate::spa::interface::{Hook, HookRef};
use crate::spa::list::ListElement;
use crate::spa::loop_::{
    AsLoopRef, EventSource, IOSource, IdleSource, SignalSource, SourceRef, TimerSource,
};
use crate::wrapper::{RawWrapper, Wrapper};
use crate::{i32_as_void_result, new_instance_raw_wrapper};

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct MainLoopRef {
    #[raw]
    raw: pw_sys::pw_main_loop,
}

#[derive(Wrapper, Debug)]
pub struct MainLoop {
    #[raw_wrapper]
    ref_: NonNull<MainLoopRef>,

    pipewire: std::sync::Arc<PipeWire>,
}

impl MainLoopRef {
    pub fn get_loop(&self) -> &LoopRef {
        unsafe { LoopRef::from_raw_ptr(pw_sys::pw_main_loop_get_loop(self.as_raw_ptr())) }
    }

    pub fn run(&self) -> crate::Result<()> {
        let result = unsafe { pw_sys::pw_main_loop_run(self.as_raw_ptr()) };
        i32_as_void_result(result)
    }

    pub fn quit(&self) -> crate::Result<()> {
        let result = unsafe { pw_sys::pw_main_loop_quit(self.as_raw_ptr()) };
        i32_as_void_result(result)
    }
}

impl MainLoopRef {
    pub fn add_io<'l, F>(
        &'l self,
        fd: RawFd,
        mask: u32,
        callback: Box<F>,
    ) -> crate::Result<IOSource<'l>>
    where
        F: FnMut(RawFd, u32),
        F: 'l,
    {
        self.get_loop().utils().add_io(self, fd, mask, callback)
    }

    pub fn update_io(&self, source: &IOSource, mask: u32) -> crate::Result<()> {
        self.get_loop().utils().update_io(source, mask)
    }

    pub fn add_idle<'l, F>(
        &'l self,
        enabled: bool,
        callback: Box<F>,
    ) -> crate::Result<IdleSource<'l>>
    where
        F: FnMut(),
        F: 'l,
    {
        self.get_loop().utils().add_idle(self, enabled, callback)
    }

    pub fn enable_idle(&self, source: &IdleSource, enabled: bool) -> crate::Result<()> {
        self.get_loop().utils().enable_idle(source, enabled)
    }

    pub fn add_event<'l, F>(&'l self, callback: Box<F>) -> crate::Result<EventSource<'l>>
    where
        F: FnMut(u64),
        F: 'l,
    {
        self.get_loop().utils().add_event(self, callback)
    }

    pub fn signal_event(&self, source: &EventSource) -> crate::Result<()> {
        self.get_loop().utils().signal_event(source)
    }

    pub fn add_timer<'l, F>(&'l self, callback: Box<F>) -> crate::Result<TimerSource<'l>>
    where
        F: FnMut(u64),
        F: 'l,
    {
        self.get_loop().utils().add_timer(self, callback)
    }

    pub fn update_timer(
        &self,
        source: &TimerSource,
        value: Duration,
        interval: Duration,
        absolute: bool,
    ) -> crate::Result<()> {
        self.get_loop()
            .utils()
            .update_timer(source, value, interval, absolute)
    }

    pub fn disable_timer(&self, source: &TimerSource) -> crate::Result<()> {
        self.get_loop().utils().disable_timer(source)
    }

    pub fn add_signal<'l, F>(
        &'l self,
        signal_number: i32,
        callback: Box<F>,
    ) -> crate::Result<SignalSource<'l>>
    where
        F: FnMut(i32),
        F: 'l,
    {
        self.get_loop()
            .utils()
            .add_signal(self, signal_number, callback)
    }

    /// Must be called inside loop or when loop is not running
    pub fn destroy_source(&self, source: &SourceRef) -> crate::Result<()> {
        self.get_loop().utils().destroy_source(source)
    }

    // todo add_listener
}

impl MainLoopRef {
    pub fn iter(&self, timeout_millis: i32) -> LoopRefIterator {
        self.get_loop().iter(timeout_millis)
    }
}

impl Drop for MainLoop {
    fn drop(&mut self) {
        unsafe { pw_sys::pw_main_loop_destroy(self.as_raw()) }
    }
}

impl AsLoopRef for MainLoop {
    fn loop_(&self) -> &crate::spa::loop_::LoopRef {
        self.get_loop().loop_()
    }
}

impl AsLoopRef for MainLoopRef {
    fn loop_(&self) -> &crate::spa::loop_::LoopRef {
        self.get_loop().loop_()
    }
}

impl MainLoop {
    pub fn new(pipewire: std::sync::Arc<PipeWire>, props: &DictRef) -> crate::Result<Self> {
        let main_loop_ptr = unsafe { pw_sys::pw_main_loop_new(props.as_raw_ptr()) };
        let ref_ = new_instance_raw_wrapper(main_loop_ptr)?;
        Ok(Self { ref_, pipewire })
    }
}

impl Default for MainLoop {
    fn default() -> Self {
        MainLoop::new(
            std::sync::Arc::new(PipeWire::default()),
            Properties::default().dict(),
        )
        .unwrap()
    }
}
