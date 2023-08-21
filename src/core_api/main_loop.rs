/*
 * SPDX-License-Identifier: MIT
 */

//! PipeWire [Main Loop](https://docs.pipewire.org/group__pw__main__loop.html) bindings.
//!
use std::fmt::Debug;
use std::ops::Deref;
use std::os::raw;
use std::ptr::NonNull;

use pipewire_wrapper_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::loop_::{LoopRef, LoopRefIterator};
use crate::core_api::properties::Properties;
use crate::core_api::PipeWire;
use crate::spa::dict::DictRef;
use crate::spa::loop_::AsLoopRef;
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

    pipewire: PipeWire,
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
    pub fn new(pipewire: PipeWire, props: &DictRef) -> crate::Result<Self> {
        let main_loop_ptr = unsafe { pw_sys::pw_main_loop_new(props.as_raw_ptr()) };
        let ref_ = new_instance_raw_wrapper(main_loop_ptr)?;
        Ok(Self { ref_, pipewire })
    }
}

impl Default for MainLoop {
    fn default() -> Self {
        MainLoop::new(PipeWire::default(), Properties::default().dict()).unwrap()
    }
}
